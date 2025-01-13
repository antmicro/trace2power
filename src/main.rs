// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use itertools::izip;
use rayon::prelude::*;
use std::collections::HashMap;
use wellen;

pub mod stats;
mod exporters;

#[derive(Parser)]
struct Cli {
    input_file: std::path::PathBuf,
    #[arg(short, long, value_parser = clap::value_parser!(f64))]
    clk_freq: f64,
}

const LOAD_OPTS: wellen::LoadOptions = wellen::LoadOptions {
    multi_thread: true,
    remove_scopes_with_empty_name: false,
};

fn process_trace(args: Cli) {
    let mut wave =
        wellen::simple::read_with_options(args.input_file.to_str().unwrap(), &LOAD_OPTS)
            .unwrap();
    let (all_sig_refs, all_names): (Vec<_>, Vec<_>) = wave
        .hierarchy()
        .iter_vars()
        .map(|var| (var.signal_ref(), var.full_name(wave.hierarchy())))
        .collect();
    wave.load_signals_multi_threaded(&all_sig_refs[..]);
    let time_end = *wave.time_table().last().unwrap();

    let stats: Vec<_> = all_sig_refs
        .par_iter()
        .map(|sig_ref| wave.get_signal(*sig_ref).unwrap())
        .zip(all_names)
        .flat_map(|(sig, name)| stats::calc_stats(sig, name, time_end))
        .collect();

    let clk_period = 1.0_f64 / args.clk_freq;

    exporters::tcl::export(&wave, &stats, clk_period, std::io::stdout()).unwrap();
}

fn main() {
    let args = Cli::parse();
    process_trace(args);
}
