// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use pretty_assertions::assert_eq;
use std::fs;
use std::path::PathBuf;
use trace2power::process;
use trace2power::Args;
use trace2power::OutputFormat;

#[test]
fn test_synth() {
    let input_file = PathBuf::from(r"tests/synth/counter.vcd");
    let clk_freq = 500000000.0;
    let clock_name = None;
    let output_format = OutputFormat::Saif;
    let limit_scope = Some(String::from("counter_tb.counter0"));
    let netlist = Some(PathBuf::from(r"tests/synth/counter.json"));
    let top = None;
    let top_scope = None;
    let blackboxes_only = false;
    let remove_virtual_pins = true;
    let output = PathBuf::from(r"tests/synth/out.saif");
    std::fs::File::create(&output).expect("Created file should be valid");
    let ignore_date = true;
    let ignore_version = true;
    let per_clock_cycle = false;
    let only_glitches = false;
    let export_empty = false;
    let args = Args::new(
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
        Some(output.clone()),
        ignore_date,
        ignore_version,
        per_clock_cycle,
        only_glitches,
        export_empty,
    );

    process(args);

    let golden = fs::read_to_string(r"tests/synth/synth.saif").expect("Golden file should exist");
    let actual = fs::read_to_string(output.to_str().expect("Actual file should exist")).unwrap();
    assert_eq!(golden, actual);
}
