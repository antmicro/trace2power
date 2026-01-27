// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use trace2power::process;
use trace2power::Args;

fn main() {
    let args = Args::from_cli();
    process(args);
}
