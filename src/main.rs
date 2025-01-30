// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use clap::Parser;
use wellen::{self, GetItem, Var, simple::Waveform, Hierarchy, ScopeRef, SignalRef};

pub mod stats;
pub mod netlist;
mod exporters;

use netlist::Netlist;

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

struct Context {
    wave: Waveform,
    clk_period: f64,
    all_sig_refs: Vec<SignalRef>,
    all_names: Vec<String>,
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

        // TODO: This can likely be made more performant if we ditch the iterators
        let lookup_scope_name_prefix = match lookup_point {
            LookupPoint::Top => "".to_string(),
            LookupPoint::Scope(scope_ref) => {
                let scope = wave.hierarchy().get(scope_ref);
                scope.full_name(wave.hierarchy()).to_string() + "."
            }
        };
        let (all_sig_refs, all_names): (Vec<_>, Vec<_>) = wave
            .hierarchy()
            .iter_vars()
            .map(|var| {
                (var.signal_ref(), indexed_name(var.full_name(wave.hierarchy().into()), var))
            })
            .filter(|(_, fname)| {
                match lookup_point {
                    LookupPoint::Top => true,
                    LookupPoint::Scope(_) => fname.starts_with(&lookup_scope_name_prefix)
                }
            })
            .collect();

        wave.load_signals_multi_threaded(&all_sig_refs[..]);

        let clk_period = 1.0_f64 / args.clk_freq;

        let top_scope = args.top_scope.as_ref()
            .map(|path| path.split('.').collect::<Vec<_>>())
            .map(|scope| wave.hierarchy().lookup_scope(&scope)
                .unwrap_or_else(|| panic!("Couldn't find top scope `{}`", scope.join(".")))
            );

        Self {
            wave,
            clk_period,
            all_sig_refs,
            all_names,
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
