// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use trace2power::process_single_iteration_trace;
use trace2power::process_trace_iterations;
use trace2power::Cli;
use trace2power::Context;

fn main() {
    let args = Cli::from_cli();
    let ctx = Context::build_from_args(&args);
    if ctx.num_of_iterations > 1 {
        process_trace_iterations(&ctx, args.output);
    } else {
        process_single_iteration_trace(&ctx, args.output);
    }
}
