// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::iter;
use std::path::PathBuf;
use std::{io::BufWriter, str::FromStr};
use std::collections::HashMap;

use clap::Parser;
use stats::PackedStats;
use wellen::{self, simple::Waveform, GetItem, Hierarchy, ScopeRef, Var, VarRef};
use rayon::prelude::*;

pub mod util;
pub mod stats;
pub mod netlist;
mod exporters;

use netlist::Netlist;
use util::VarRefsIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HashVarRef(VarRef);

impl std::hash::Hash for HashVarRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.index().hash(state);
    }
}

/// trace2power - Extract acccumulated power activity data from VCD/FST
#[derive(Parser)]
struct Cli {
    /// Trace file
    input_file: std::path::PathBuf,
    /// Clock frequency (in Hz)
    #[arg(short, long, value_parser = clap::value_parser!(f64))]
    clk_freq: f64,
    /// Format to extract data into
    #[arg(short = 'f', long, default_value = "tcl")]
    output_format: OutputFormat,
    /// Scope in which signals should be looked for. By default it's the global hierarchy scope.
    #[arg(long, short)]
    limit_scope: Option<String>,
    /// Yosys JSON netlist of DUT. Can be used to identify ports of primitives when exporting data.
    /// Allows skipping unnecessary or unwanted signals
    #[arg(short, long)]
    netlist: Option<std::path::PathBuf>,
    /// Name of the top module (DUT)
    #[arg(short, long)]
    top: Option<String>,
    /// Scope at which the DUT is located. The loaded netlist will be rooted at this point.
    #[arg(short = 'T', long)]
    top_scope: Option<String>,
    /// Export only nets from blackboxes (undefined modules) in provided netlist. Those are assumed
    /// to be post-synthesis primitives
    #[arg(short, long)]
    blackboxes_only: bool,
    /// Remove nets that are in blackboxes and have suspicious names: "VGND", "VNB", "VPB", "VPWR".
    #[arg(long)]
    remove_virtual_pins: bool,
    /// Write the output to a specified file instead of stdout.
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
    /// Time span of single stats accumulation. By default it accumulates from the entire trace file.
    #[arg(short, long, value_parser = clap::value_parser!(u64))]
    span: Option<u64>
}

fn indexed_name(mut name: String, variable: &Var) -> String {
    if let Some(idx) = variable.index() {
        name += format!("[{}]", idx.lsb()).as_str();
    }
    name
}

fn get_scope_by_full_name(hier: &Hierarchy, scope_str: &str) -> Option<ScopeRef> {
    hier.lookup_scope(scope_str.split('.').collect::<Vec<_>>().as_slice())
}

/// Represents a pointin hierarchy - either a scope or top-level hierarchy as they are distinct for
/// some reason
#[derive(Copy, Clone)]
enum LookupPoint {
    Top,
    Scope(ScopeRef)
}

#[derive(Copy, Clone)]
enum OutputFormat {
    Tcl,
    Saif
}

impl clap::ValueEnum for OutputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Tcl, Self::Saif]
    }
    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        use clap::builder::PossibleValue;
        match self {
            Self::Tcl => Some(PossibleValue::new("tcl")),
            Self::Saif => Some(PossibleValue::new("saif")),
        }
    }
}

impl FromStr for OutputFormat {
    type Err = std::io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tcl" => Ok(Self::Tcl),
            "saif" => Ok(Self::Saif),
            other @ _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Format {} is not a valid output format forthis program", other)
            )),
        }
    }
}

struct Context {
    wave: Waveform,
    clk_period: f64,
    stats: HashMap<HashVarRef, Vec<PackedStats>>,
    num_of_iterations: u64,
    lookup_point: LookupPoint,
    output_fmt: OutputFormat,
    scope_prefix_length: usize,
    netlist: Option<Netlist>,
    top: String,
    top_scope: Option<ScopeRef>,
    blackboxes_only: bool,
    remove_virtual_pins: bool
}

impl Context {
    fn build_from_args(args: &Cli) -> Self {
        const LOAD_OPTS: wellen::LoadOptions = wellen::LoadOptions {
            multi_thread: true,
            remove_scopes_with_empty_name: false,
        };

        let mut wave =
            wellen::simple::read_with_options(args.input_file.to_str().unwrap(), &LOAD_OPTS)
                .unwrap();

        let lookup_point = match &args.limit_scope {
            None => LookupPoint::Top,
            Some(scope_str) => LookupPoint::Scope(
                get_scope_by_full_name(wave.hierarchy(), scope_str).expect("Requested scope not found")
            ),
        };

        let lookup_scope_name_prefix = match lookup_point {
            LookupPoint::Top => "".to_string(),
            LookupPoint::Scope(scope_ref) => {
                let scope = wave.hierarchy().get(scope_ref);
                scope.full_name(wave.hierarchy()).to_string() + "."
            }
        };

        let (all_vars, all_signals): (Vec<_>, Vec<_>) = match lookup_point {
            LookupPoint::Top => wave.hierarchy().var_refs_iter()
                .map(|var_ref| (var_ref, wave.hierarchy().get(var_ref).signal_ref()))
                .unzip(),
            LookupPoint::Scope(_) => wave.hierarchy().var_refs_iter()
                .map(|var_ref| (var_ref, wave.hierarchy().get(var_ref)))
                .filter(|(_, var)| {
                    let fname = indexed_name(var.full_name(wave.hierarchy().into()), var);
                    fname.starts_with(&lookup_scope_name_prefix)
                })
                .map(|(var_ref, var)| (var_ref, var.signal_ref()))
                .unzip()
        };

        wave.load_signals_multi_threaded(&all_signals);

        let last_time_stamp = *wave.time_table().last().unwrap();
        let accumulation_span = args.span.unwrap_or_else(|| last_time_stamp);
        let num_of_iterations = last_time_stamp / accumulation_span;

        // TODO: A massive optimization that can be done here is to calculate stats only
        // for exported signals instead of all nets
        // It's easy to do with the current implementation of DFS (see src/exporter/mod.rs).
        // However it's single-threaded and parallelizing it efficiently is non-trivial.
        let stats: HashMap<HashVarRef, Vec<stats::PackedStats>> = all_vars.par_iter()
            .zip(all_signals)
            .map(|(var_ref, sig_ref)| (*var_ref, wave.get_signal(sig_ref).unwrap()))
            .map(|(var_ref, sig)| (HashVarRef(var_ref), stats::calc_stats_temp(&wave, sig, num_of_iterations)))
            .collect();

        let clk_period = 1.0_f64 / args.clk_freq;

        let top_scope = args.top_scope.as_ref()
            .map(|s| get_scope_by_full_name(wave.hierarchy(), s)
                .unwrap_or_else(|| panic!("Couldn't find top scope `{}`", s))
        );

        Self {
            wave,
            clk_period,
            stats,
            num_of_iterations,
            lookup_point,
            output_fmt: args.output_format,
            scope_prefix_length: lookup_scope_name_prefix.len(),
            netlist: args.netlist.as_ref().map(|path| {
                let f = std::fs::File::open(path).expect("Couldn't open the netlist file");
                let reader = std::io::BufReader::new(f);
                serde_json::from_reader::<_, Netlist>(reader)
                    .expect("Couldn't parse the netlist file")
            }),
            top: args.top.clone().unwrap_or_else(String::new),
            top_scope,
            blackboxes_only: args.blackboxes_only,
            remove_virtual_pins: args.remove_virtual_pins
        }
    }
}

fn process_trace<W>(ctx: &Context, out: W, iteration: usize) where W: std::io::Write {
    match &ctx.output_fmt {
        OutputFormat::Tcl => exporters::tcl::export(&ctx, out, iteration),
        OutputFormat::Saif => exporters::saif::export(&ctx, out, iteration),
    }.unwrap()
}

fn main() {
    let args = Cli::parse();
    let ctx = Context::build_from_args(&args);
    match args.output {
        None => process_trace(&ctx, std::io::stdout(), 0),
        Some(path) => {
            for iteration in 0..ctx.num_of_iterations {
                let mut file_path = path.clone();
                file_path.push(iteration.to_string() + ".tcl");
                println!("{:?}", file_path);
                let f = std::fs::File::create(file_path).unwrap();
                let writer = std::io::BufWriter::new(f);
                process_trace(&ctx, writer, iteration as usize);
            }
        }
    }
}
