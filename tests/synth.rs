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
    input_file: PathBuf,
    clk_freq: f64,
    clock_name: Option<String>,
    limit_scope: Option<String>,
    netlist: Option<std::path::PathBuf>,
    top: Option<String>,
    top_scope: Option<String>,
    blackboxes_only: bool,
    remove_virtual_pins: bool,
    output: Option<std::path::PathBuf>,
    output_file: NamedTempFile,
    per_clock_cycle: bool,
    only_glitches: bool,
    export_empty: bool,
}

impl Common {
    fn new() -> Self {
        let output_file = NamedTempFile::new().expect("Failed to allocate temp file");
        Self {
            input_file: PathBuf::from(r"tests/synth/counter.vcd"),
            clk_freq: 500000000.0,
            clock_name: None,
            limit_scope: Some(String::from("counter_tb.counter0")),
            netlist: Some(PathBuf::from(r"tests/synth/counter.json")),
            top: None,
            top_scope: None,
            blackboxes_only: false,
            remove_virtual_pins: true,
            output: Some(output_file.path().to_path_buf()),
            output_file,
            per_clock_cycle: false,
            only_glitches: false,
            export_empty: false,
        }
    }
}

#[test]
fn test_synth_saif() {
    let mut common = Common::new();
    let output_format = OutputFormat::Saif;
    let ignore_date = true;
    let ignore_version = true;
    let args = Args::new(
        common.input_file,
        common.clk_freq,
        common.clock_name,
        output_format,
        common.limit_scope,
        common.netlist,
        common.top,
        common.top_scope,
        common.blackboxes_only,
        common.remove_virtual_pins,
        common.output,
        ignore_date,
        ignore_version,
        common.per_clock_cycle,
        common.only_glitches,
        common.export_empty,
    );

    process(args);
    let golden = fs::read_to_string(r"tests/synth/synth.saif").expect("Golden file should exist");
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
    let output_format = OutputFormat::Tcl;
    let ignore_date = false;
    let ignore_version = false;
    let args = Args::new(
        common.input_file,
        common.clk_freq,
        common.clock_name,
        output_format,
        common.limit_scope,
        common.netlist,
        common.top,
        common.top_scope,
        common.blackboxes_only,
        common.remove_virtual_pins,
        common.output,
        ignore_date,
        ignore_version,
        common.per_clock_cycle,
        common.only_glitches,
        common.export_empty,
    );

    process(args);

    let mut golden =
        fs::read_to_string(r"tests/synth/synth.tcl").expect("Golden file should exist");
    let mut actual = String::new();
    common
        .output_file
        .read_to_string(&mut actual)
        .expect("Actual file should exist");
    assert_eq!(sort_tcl(&mut actual), sort_tcl(&mut golden));
}

fn sort_tcl(input: &mut String) -> String {
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
