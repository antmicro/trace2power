// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;
use std::collections::HashMap;

use clap::Parser;
use stats::PackedStats;
use wellen::{self, simple::Waveform, GetItem, Hierarchy, Scope, ScopeRef, SignalRef, Var, VarRef};
use rayon::prelude::*;

pub mod stats;
pub mod netlist;
mod exporters;

use netlist::Netlist;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HashVarRef(VarRef);

impl std::hash::Hash for HashVarRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.index().hash(state);
    }
}

#[derive(Parser)]
struct Cli {
    input_file: std::path::PathBuf,
    #[arg(short, long, value_parser = clap::value_parser!(f64))]
    clk_freq: f64,
    #[arg(long, default_value = "tcl")]
    output_format: String,
    #[arg(long, short)]
    limit_scope: Option<String>,
    #[arg(long)]
    netlist: Option<String>,
    #[arg(long)]
    top: Option<String>,
    #[arg(long)]
    top_scope: Option<String>,
    #[arg(short, long)]
    blackboxes_only: bool,
}

const LOAD_OPTS: wellen::LoadOptions = wellen::LoadOptions {
    multi_thread: true,
    remove_scopes_with_empty_name: false,
};

fn indexed_name(mut name: String, variable: &Var) -> String {
    if let Some(idx) = variable.index() {
        name += format!("[{}]", idx.lsb()).as_str();
    }
    name
}

fn get_scope(hier: &Hierarchy, scope_str: &str) -> Option<ScopeRef> {
    hier.lookup_scope(scope_str.split('.').collect::<Vec<_>>().as_slice())
}

#[derive(Copy, Clone)]
enum LookupPoint {
    Top,
    Scope(ScopeRef)
}

enum OutputFormat {
    Tcl,
    Saif
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

// Wellen has no good iterator over ALL VarRefs, so I made one. The generics are here to deal with
// hidden iterator types.
struct VarRefIterator<'w, HIter, SIter, F, S>
where
    HIter: Iterator<Item = VarRef> + 'w,
    SIter: Iterator<Item = ScopeRef> + 'w,
    F: Fn(&'w Scope) -> HIter,
    S: Fn(&'w Scope) -> SIter
{
    hierarchy: &'w Hierarchy,
    get_iter_of_scope: F,
    get_siter_of_scope: S,
    scopes: Vec<&'w Scope>,
    iter: HIter,
    siter: SIter,
}

impl<'w, HIter, SIter, F, S> Iterator for VarRefIterator<'w, HIter, SIter, F, S>
where
    HIter: Iterator<Item = VarRef> + 'w,
    SIter: Iterator<Item = ScopeRef> + 'w,
    F: Fn(&'w Scope) -> HIter,
    S: Fn(&'w Scope) -> SIter
{
    type Item = VarRef;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(v) => Some(v),
            None => {
                if let Some(scope) = self.scopes.pop() {
                    self.iter = (self.get_iter_of_scope)(scope);
                    self.siter = (self.get_siter_of_scope)(scope);
                    self.scopes.extend(self.siter.by_ref().map(|s| self.hierarchy.get(s)));
                    self.next()
                } else {
                    None
                }
            }
        }
    }
}

fn var_refs_iter<'h>(hierarchy: &'h Hierarchy) -> impl Iterator<Item = VarRef> + 'h {
    VarRefIterator {
        hierarchy: &hierarchy,
        scopes: hierarchy.scopes().map(|s| hierarchy.get(s)).collect(),
        get_iter_of_scope: |s: &Scope| s.vars(hierarchy),
        get_siter_of_scope: |s: &Scope| s.scopes(hierarchy),
        iter: hierarchy.vars(),
        siter: hierarchy.scopes(),
    }
}

struct Context {
    wave: Waveform,
    clk_period: f64,
    stats: HashMap<HashVarRef, PackedStats>,
    lookup_point: LookupPoint,
    output_fmt: OutputFormat,
    scope_prefix_length: usize,
    netlist: Option<Netlist>,
    top: String,
    top_scope: Option<ScopeRef>,
    blackboxes_only: bool
}

impl Context {
    fn build_from_args(args: &Cli) -> Self {
        let output_fmt = OutputFormat::from_str(&args.output_format).unwrap();

        let mut wave =
            wellen::simple::read_with_options(args.input_file.to_str().unwrap(), &LOAD_OPTS)
                .unwrap();

        let lookup_point = match &args.limit_scope {
            None => LookupPoint::Top,
            Some(scope_str) => LookupPoint::Scope(
                get_scope(wave.hierarchy(), scope_str).expect("Requested scope not found")
            ),
        };


        if let (LookupPoint::Scope(_), OutputFormat::Saif) = (&lookup_point, &output_fmt) {
            panic!("Scoped lookup for SAIF is WIP");
        }

        let lookup_scope_name_prefix = match lookup_point {
            LookupPoint::Top => "".to_string(),
            LookupPoint::Scope(scope_ref) => {
                let scope = wave.hierarchy().get(scope_ref);
                scope.full_name(wave.hierarchy()).to_string() + "."
            }
        };

        let (all_vars, all_signals): (Vec<_>, Vec<_>) = match lookup_point {
            LookupPoint::Top => var_refs_iter(wave.hierarchy())
                .map(|var_ref| (var_ref, wave.hierarchy().get(var_ref).signal_ref()))
                .unzip(),
            LookupPoint::Scope(_) => var_refs_iter(wave.hierarchy())
                .map(|var_ref| (var_ref, wave.hierarchy().get(var_ref)))
                .filter(|(_, var)| {
                    let fname = indexed_name(var.full_name(wave.hierarchy().into()), var);
                    fname.starts_with(&lookup_scope_name_prefix)
                })
                .map(|(var_ref, var)| (var_ref, var.signal_ref()))
                .unzip()
        };

        wave.load_signals_multi_threaded(&all_signals);

        let time_end = *wave.time_table().last().unwrap();

        // TODO: A massive optimization that can be done here is to calculate stats only
        // for exported signals instead of all nets
        // It's easy to do with the current implementation of DFS (see src/exporter/mod.rs).
        // However it's single-threaded and parallelizing it efficiently is non-trivial.
        let stats: HashMap<HashVarRef, stats::PackedStats> = all_vars.par_iter()
            .zip(all_signals)
            .map(|(var_ref, sig_ref)| (*var_ref, wave.get_signal(sig_ref).unwrap()))
            .map(|(var_ref, sig)| (HashVarRef(var_ref), stats::calc_stats(sig, time_end)))
            .collect();

        let clk_period = 1.0_f64 / args.clk_freq;

        let top_scope = args.top_scope.as_ref()
            .map(|path| path.split('.').collect::<Vec<_>>())
            .map(|scope| wave.hierarchy().lookup_scope(&scope)
                .unwrap_or_else(|| panic!("Couldn't find top scope `{}`", scope.join(".")))
            );

        Self {
            wave,
            clk_period,
            stats: stats,
            lookup_point,
            output_fmt,
            scope_prefix_length: lookup_scope_name_prefix.len(),
            netlist: args.netlist.as_ref().map(|path| {
                let f = std::fs::File::open(path).expect("Couldn't open the netlist file");
                let reader = std::io::BufReader::new(f);
                serde_json::from_reader::<_, Netlist>(reader)
                    .expect("Couldn't parse the netlist file")
            }),
            top: args.top.clone().unwrap_or_else(String::new),
            top_scope,
            blackboxes_only: args.blackboxes_only
        }
    }
}

fn process_trace(ctx: Context) {
    let out = std::io::stdout();
    match &ctx.output_fmt {
        OutputFormat::Tcl => exporters::tcl::export(ctx, out),
        OutputFormat::Saif => exporters::saif::export(ctx, out),
    }.unwrap()
}

fn main() {
    let ctx = Context::build_from_args(&Cli::parse());
    process_trace(ctx);
}
