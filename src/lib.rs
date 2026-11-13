// Copyright (c) 2024-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::ops::Range;
use std::str::FromStr;
use std::{collections::HashMap, io};
use std::{fs, hash, path};

use clap::Parser;
use rayon::prelude::*;
use stats::PackedStats;
use wellen::{self, Hierarchy, ScopeRef, SignalRef, Var, VarRef, simple::Waveform};

mod exporters;
pub mod netlist;
pub mod stats;
pub mod util;

use netlist::Netlist;
use util::VarRefsIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HashVarRef(VarRef);

impl hash::Hash for HashVarRef {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.index().hash(state);
    }
}

type StatsMap = HashMap<HashVarRef, Vec<PackedStats>>;

struct StatInput {
    var_ref: VarRef,
    signal_ref: SignalRef,
    calculate_power: bool,
}

/// trace2power - Extract acccumulated power activity data from VCD/FST
#[derive(Parser)]
pub struct Args {
    /// Trace file
    pub input_file: path::PathBuf,
    /// Clock frequency (in Hz)
    #[arg(short, long, value_parser = clap::value_parser!(f64))]
    pub clk_freq: f64,
    /// Clock signal name
    #[arg(long)]
    pub clock_name: Option<String>,
    /// Format to extract data into
    #[arg(short = 'f', long, default_value = "tcl")]
    pub output_format: OutputFormat,
    /// Scope in which signals should be looked for. By default it's the global hierarchy scope.
    #[arg(long, short)]
    pub limit_scope: Option<String>,
    /// Scope in which power will be calculated. By default it's equal to `limit_scope`.
    /// Must be a subset of `limit_scope`.
    #[arg(long)]
    pub limit_scope_power: Option<String>,
    /// Yosys JSON netlist of DUT. Can be used to identify ports of primitives when exporting data.
    /// Allows skipping unnecessary or unwanted signals
    #[arg(short, long)]
    pub netlist: Option<path::PathBuf>,
    /// Name of the top module (DUT)
    #[arg(short, long)]
    pub top: Option<String>,
    /// Scope at which the DUT is located. The loaded netlist will be rooted at this point.
    #[arg(short = 'T', long)]
    pub top_scope: Option<String>,
    /// Export only nets from blackboxes (undefined modules) in provided netlist. Those are assumed
    /// to be post-synthesis primitives
    #[arg(short, long)]
    pub blackboxes_only: bool,
    /// Remove nets that are in blackboxes and have suspicious names: "VGND", "VNB", "VPB", "VPWR".
    #[arg(long)]
    pub remove_virtual_pins: bool,
    /// Write the output to a specified file instead of stdout.
    /// In case of per clock cycle output, it must be a directory.
    #[arg(short, long)]
    pub output: Option<path::PathBuf>,
    /// Ignore exporting current date.
    #[arg(long)]
    pub ignore_date: bool,
    /// Ignore exporting current version.
    #[arg(long)]
    pub ignore_version: bool,
    /// Accumulate stats for each clock cycle separately. Output path is required to be a directory.
    #[arg(long)]
    pub per_clock_cycle: bool,
    /// Write stats only for glitches
    #[arg(long)]
    pub only_glitches: bool,
    /// Export without accumulation
    #[arg(long)]
    pub export_empty: bool,
    /// Set activity for input ports in TCL mode
    #[arg(long)]
    pub input_ports_activity: bool,
    /// Controls how many clock iterations are processed at one iteration in case
    /// of per clock cycle outputs. May increase simultaneous memory usage.
    #[arg(long, default_value_t = 32, value_parser = clap::value_parser!(u64).range(1..))]
    pub stats_chunk_size: u64,
}

impl Args {
    pub fn from_cli() -> Self {
        Args::parse()
    }
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
    Scope(ScopeRef),
}

#[derive(Copy, Clone)]
pub enum OutputFormat {
    Tcl,
    Saif,
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
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tcl" => Ok(Self::Tcl),
            "saif" => Ok(Self::Saif),
            other @ _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Format {} is not a valid output format forthis program",
                    other
                ),
            )),
        }
    }
}

struct Context {
    wave: Waveform,
    clk_period: f64,
    stat_inputs: Vec<StatInput>,
    clk_signal: Option<SignalRef>,
    only_glitches: bool,
    pub num_of_iterations: u64,
    lookup_point: LookupPoint,
    output_fmt: OutputFormat,
    scope_prefix_length: usize,
    netlist: Option<Netlist>,
    top: String,
    top_scope: Option<ScopeRef>,
    blackboxes_only: bool,
    remove_virtual_pins: bool,
    ignore_date: bool,
    ignore_version: bool,
    export_empty: bool,
    power_scope_prefix: String,
    input_ports_activity: bool,
    stats_chunk_size: u64,
}

impl Context {
    pub fn build_from_args(args: &Args) -> Self {
        const LOAD_OPTS: wellen::LoadOptions = wellen::LoadOptions {
            multi_thread: true,
            remove_scopes_with_empty_name: false,
        };

        let mut wave = wellen::simple::read_with_options(
            args.input_file
                .to_str()
                .expect("Arguments should contain a path to input trace file"),
            &LOAD_OPTS,
        )
        .expect("Waveform parsing should end successfully");

        let wave_hierarchy = wave.hierarchy();

        let clk_period = 1.0_f64 / args.clk_freq;
        let timescale = wave_hierarchy
            .timescale()
            .expect("Trace file should contain a timescale");
        let timescale_norm = (timescale.factor as f64)
            * (10.0_f64).powf(
                timescale
                    .unit
                    .to_exponent()
                    .expect("Waveform should contain time unit") as f64,
            );

        let lookup_point = match &args.limit_scope {
            None => LookupPoint::Top,
            Some(scope_str) => LookupPoint::Scope(
                get_scope_by_full_name(wave_hierarchy, scope_str)
                    .expect("Requested scope not found"),
            ),
        };

        let lookup_scope_name_prefix = match lookup_point {
            LookupPoint::Top => "".to_string(),
            LookupPoint::Scope(scope_ref) => {
                let scope = &wave_hierarchy[scope_ref];
                scope.full_name(wave_hierarchy).to_string() + "."
            }
        };

        let (all_vars, all_signals): (Vec<_>, Vec<_>) = match lookup_point {
            LookupPoint::Top => wave_hierarchy
                .var_refs_iter()
                .map(|var_ref| (var_ref, wave_hierarchy[var_ref].signal_ref()))
                .unzip(),
            LookupPoint::Scope(_) => wave_hierarchy
                .var_refs_iter()
                .map(|var_ref| (var_ref, &wave_hierarchy[var_ref]))
                .filter(|(_, var)| {
                    let fname = indexed_name(var.full_name(wave_hierarchy.into()), var);
                    fname.starts_with(&lookup_scope_name_prefix)
                })
                .map(|(var_ref, var)| (var_ref, var.signal_ref()))
                .unzip(),
        };

        let all_signals_power: BTreeSet<_> = match &args.limit_scope_power {
            None => wave_hierarchy
                .var_refs_iter()
                .map(|var_ref| &wave_hierarchy[var_ref])
                .map(|var| indexed_name(var.full_name(wave_hierarchy.into()), var))
                .collect::<BTreeSet<_>>(),
            Some(scope_str) => wave_hierarchy
                .var_refs_iter()
                .map(|var_ref| &wave_hierarchy[var_ref])
                .filter(|var| {
                    let fname = indexed_name(var.full_name(wave_hierarchy.into()), var);
                    fname.starts_with(scope_str)
                })
                .map(|var| indexed_name(var.full_name(wave_hierarchy.into()), &var))
                .collect::<BTreeSet<_>>(),
        };

        let clk_signal: Option<SignalRef> = match &args.clock_name {
            None => None,
            Some(clock_name) => {
                let mut found: Option<SignalRef> = None;

                for var_ref in wave_hierarchy.var_refs_iter() {
                    let net = &wave_hierarchy[var_ref];
                    let sig_ref = net.signal_ref();
                    if net.name(wave_hierarchy) == clock_name {
                        found = Some(sig_ref)
                    }
                }

                found
            }
        };

        // TODO load signals that are under a power scope
        wave.load_signals_multi_threaded(&all_signals);

        let last_time_stamp = *wave
            .time_table()
            .last()
            .expect("Given waveform shouldn't be empty");
        let num_of_iterations = if args.per_clock_cycle {
            (last_time_stamp as f64 * timescale_norm / clk_period) as u64
        } else {
            1
        };

        let stat_inputs = all_vars
            .into_iter()
            .zip(all_signals)
            .map(|(var_ref, signal_ref)| {
                let var = &wave.hierarchy()[var_ref];
                let fname = indexed_name(var.full_name(wave.hierarchy().into()), var);
                StatInput {
                    var_ref,
                    signal_ref,
                    calculate_power: args.limit_scope_power.is_none()
                        || all_signals_power.contains(&fname),
                }
            })
            .collect();

        let top_scope = args.top_scope.as_ref().map(|s| {
            get_scope_by_full_name(wave.hierarchy(), s)
                .unwrap_or_else(|| panic!("Couldn't find top scope `{}`", s))
        });

        Self {
            wave,
            clk_period,
            stat_inputs,
            clk_signal,
            only_glitches: args.only_glitches,
            num_of_iterations,
            lookup_point,
            output_fmt: args.output_format,
            scope_prefix_length: lookup_scope_name_prefix.len(),
            netlist: args.netlist.as_ref().map(|path| {
                let f = fs::File::open(path).expect("Couldn't open the netlist file");
                let reader = io::BufReader::new(f);
                serde_json::from_reader::<_, Netlist>(reader)
                    .expect("Couldn't parse the netlist file")
            }),
            top: args.top.clone().unwrap_or_else(String::new),
            top_scope,
            blackboxes_only: args.blackboxes_only,
            remove_virtual_pins: args.remove_virtual_pins,
            ignore_date: args.ignore_date,
            ignore_version: args.ignore_version,
            export_empty: args.export_empty,
            power_scope_prefix: args
                .limit_scope_power
                .clone()
                .unwrap_or_else(|| lookup_scope_name_prefix),
            input_ports_activity: args.input_ports_activity,
            stats_chunk_size: args.stats_chunk_size,
        }
    }
}

pub fn process(args: Args) {
    println!("Collecting...");
    let ctx = Context::build_from_args(&args);
    println!("Exporting...");
    if ctx.num_of_iterations > 1 {
        process_trace_iterations(&ctx, args.output);
    } else {
        process_single_iteration_trace(&ctx, args.output);
    }
}

fn process_trace(ctx: &Context, stats: &StatsMap, out: impl io::Write, iteration: usize) {
    match &ctx.output_fmt {
        OutputFormat::Tcl => exporters::tcl::export(ctx, stats, out, iteration),
        OutputFormat::Saif => exporters::saif::export(ctx, stats, out, iteration),
    }
    .expect("Output format should be either 'tcl' or 'saif'")
}

fn process_trace_iterations(ctx: &Context, mut output_path: Option<path::PathBuf>) {
    // TODO: multithreading can also be introduced here to process each iteration in parallel
    for chunk_start in (0..ctx.num_of_iterations).step_by(ctx.stats_chunk_size as usize) {
        let chunk_end = (chunk_start + ctx.stats_chunk_size).min(ctx.num_of_iterations);
        let stats = calculate_stats_chunk(ctx, chunk_start..chunk_end);

        for iteration in chunk_start..chunk_end {
            let local_iteration = (iteration - chunk_start) as usize;
            if let Some(path) = output_path.as_mut() {
                path.push(format!("{:05}", iteration));
                let f = fs::File::create(&path).expect("Created file should be valid");
                let writer = io::BufWriter::new(f);
                process_trace(ctx, &stats, writer, local_iteration);
                path.pop();
            } else {
                println!("{1} Iteration {:05} {1}", iteration, str::repeat("-", 10));
                process_trace(ctx, &stats, io::stdout(), local_iteration);
            }
        }
    }
}

fn calculate_stats_chunk(ctx: &Context, iterations: Range<u64>) -> StatsMap {
    let num_of_iterations = ctx.num_of_iterations;
    let chunk_len = (iterations.end - iterations.start) as usize;

    ctx.stat_inputs
        .par_iter()
        .map(|input| {
            let stats = if input.calculate_power {
                stats::calc_stats_for_time_span_range(
                    &ctx.wave,
                    ctx.only_glitches,
                    ctx.clk_signal,
                    input.signal_ref,
                    num_of_iterations,
                    iterations.clone(),
                )
            } else {
                vec![stats::empty_stats(&ctx.wave, input.signal_ref); chunk_len]
            };
            (HashVarRef(input.var_ref), stats)
        })
        .collect()
}

fn process_single_iteration_trace(ctx: &Context, output_path: Option<path::PathBuf>) {
    let stats = calculate_stats_chunk(ctx, 0..1);
    match output_path {
        None => process_trace(ctx, &stats, io::stdout(), 0),
        Some(ref path) => {
            let f = fs::File::create(path).expect("Created file should be valid");
            let writer = io::BufWriter::new(f);
            process_trace(ctx, &stats, writer, 0);
        }
    }
}
