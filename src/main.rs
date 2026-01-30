// Copyright (c) 2024-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use trace2power::Args;
use trace2power::process;

fn main() {
    let args = Args::from_cli();
    process(args);
}
