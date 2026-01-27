// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use pretty_assertions::assert_eq;
use std::fs;
use std::io::Read;
use tempfile::NamedTempFile;

use std::path::PathBuf;
use trace2power::process;
use trace2power::Args;
use trace2power::OutputFormat;

struct Common {
    args: Args,
    output_file: NamedTempFile,
}

impl Common {
    fn new() -> Self {
        let output_file = NamedTempFile::new().expect("Failed to allocate temp file");
        Common {
            args: common_args(output_file.path().to_path_buf()),
            output_file,
        }
    }
}
fn common_args(output_file: PathBuf) -> Args {
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
        output: Some(output_file),
        ignore_date: false,
        ignore_version: false,
        per_clock_cycle: false,
        only_glitches: false,
        export_empty: false,
    }
}

#[test]
fn test_synth_saif() {
    let mut common = Common::new();
    common.args.output_format = OutputFormat::Saif;
    common.args.ignore_date = true;
    common.args.ignore_version = true;

    process(common.args);

    let golden = include_str!("synth/synth.saif");
    let mut actual = String::new();
    common
        .output_file
        .read_to_string(&mut actual)
        .expect("Actual file should exist");
    assert_eq!(actual, golden);
}

#[test]
fn test_synth_tcl() {
    let mut common = Common::new();

    process(common.args);

    let golden = include_str!("synth/synth.tcl");
    let mut actual = String::new();
    common
        .output_file
        .read_to_string(&mut actual)
        .expect("Actual file should exist");
    assert_eq!(sort_tcl(actual), sort_tcl(String::from(golden)));
}

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
