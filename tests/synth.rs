// Copyright (c) 2024-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use std::io::Read;
use tempfile::NamedTempFile;

use libtrace2power::Args;
use libtrace2power::OutputFormat;
use libtrace2power::process;
use std::path::PathBuf;

fn common_args(output_file: &NamedTempFile) -> Args {
    Args {
        input_file: PathBuf::from(r"tests/synth/counter.vcd"),
        clk_freq: 500000000.0,
        clock_name: None,
        output_format: OutputFormat::Tcl,
        limit_scope: Some(String::from("counter_tb.counter0")),
        netlist: Some(PathBuf::from(r"tests/synth/counter.json")),
        top: None,
        top_scope: None,
        blackboxes_only: false,
        remove_virtual_pins: true,
        output: Some(output_file.path().to_path_buf()),
        ignore_date: false,
        ignore_version: false,
        per_clock_cycle: false,
        only_glitches: false,
        export_empty: false,
        limit_scope_power: None,
        input_ports_activity: false,
    }
}

#[test]
fn test_synth_saif() {
    let mut output_file = NamedTempFile::new().expect("Failed to allocate temp file");
    let mut args = common_args(&output_file);
    args.output_format = OutputFormat::Saif;
    args.ignore_date = true;
    args.ignore_version = true;

    process(args);

    let golden = include_str!("synth/synth.saif");
    let mut actual = String::new();
    output_file
        .read_to_string(&mut actual)
        .expect("Actual file should exist");
    assert_eq!(actual, golden);
}

#[test]
fn test_synth_tcl() {
    let mut output_file = NamedTempFile::new().expect("Failed to allocate temp file");
    let args = common_args(&output_file);

    process(args);

    let golden = include_str!("synth/synth.tcl");
    let mut actual = String::new();
    output_file
        .read_to_string(&mut actual)
        .expect("Actual file should exist");
    assert_eq!(sort_tcl(actual), sort_tcl(String::from(golden)));
}

// TODO remove once grouped_stats would be iterated in a deterministic way
fn sort_tcl(input: String) -> String {
    let mut lines: Vec<&str> = input.lines().collect();

    fn key(s: &str) -> Vec<u8> {
        s.bytes()
            // ignore leading blanks
            .skip_while(|&c| c == b' ' || c == b'\t')
            // dictionary order
            .filter(|&c| c.is_ascii_alphanumeric() || c == b' ' || c == b'\t')
            .collect()
    }

    lines.sort_by(|a, b| key(a).cmp(&key(b)));
    lines.join("\n")
}
