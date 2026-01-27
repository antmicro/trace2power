// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use trace2power::process_single_iteration_trace;
use trace2power::process_trace_iterations;
use trace2power::Cli;
use trace2power::Context;
use trace2power::OutputFormat;

#[test]
fn test_synth() {
    let input_file = PathBuf::from(r"tests/synth/counter.vcd");
    let clk_freq = 500000000.0;
    let clock_name = Option::None;
    let output_format = OutputFormat::Saif;
    let limit_scope = Option::Some(String::from("counter_tb.counter0"));
    let netlist = Option::Some(PathBuf::from(r"tests/synth/counter.json"));
    let top = Option::None;
    let top_scope = Option::None;
    let blackboxes_only = false;
    let remove_virtual_pins = true;
    let output = PathBuf::from(r"tests/synth/out.saif");
    std::fs::File::create(&output).expect("Created file should be valid");
    let ignore_date = true;
    let ignore_version = true;
    let per_clock_cycle = false;
    let only_glitches = false;
    let export_empty = false;
    let args = Cli::new(
        input_file,
        clk_freq,
        clock_name,
        output_format,
        limit_scope,
        netlist,
        top,
        top_scope,
        blackboxes_only,
        remove_virtual_pins,
        Option::Some(output),
        ignore_date,
        ignore_version,
        per_clock_cycle,
        only_glitches,
        export_empty,
    );
    let ctx = Context::build_from_args(&args);
    if ctx.num_of_iterations > 1 {
        process_trace_iterations(&ctx, args.output);
    } else {
        process_single_iteration_trace(&ctx, args.output);
    }
}
