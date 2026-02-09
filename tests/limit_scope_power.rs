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
        input_file: PathBuf::from(r"tests/limit_scope_power/hierarchical.vcd"),
        clk_freq: 500000000.0,
        clock_name: None,
        output_format: OutputFormat::Tcl,
        limit_scope: Some(String::from("hierarchical_tb")),
        netlist: None,
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
        limit_scope_power: Some(String::from("hierarchical_tb.dut.adder1")),
        input_ports_activity: true,
    }
}

#[test]
fn test_limit_scope_power_tcl() {
    let mut output_file = NamedTempFile::new().expect("Failed to allocate temp file");
    let args = common_args(&output_file);

    process(args);

    let golden = include_str!("limit_scope_power/limit_scope_power.tcl");
    let mut actual = String::new();
    output_file
        .read_to_string(&mut actual)
        .expect("Actual file should exist");
    assert_eq!(actual, String::from(golden));
}
