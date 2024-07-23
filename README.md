# trace2power

Copyright (c) 2024-2025 [Antmicro](https://antmicro.com)

This repository contains the `trace2power` tool that is capable of converting traces (in VCD or FST format) to a TCL script that can be used with [OpenSTA](https://github.com/parallaxsw/OpenSTA) to set pin, input port activity and duty cycle for power analysis.
The tool replicates what OpenSTA does when loading a VCD file using the `read_power_activities` command but does so order of magnitude faster.

## Building

Note that `trace2power` requires rustc 1.80.1 or newer.

```
cargo build --release
```

## Usage

```
cargo run --release -- --clk-freq <clock frequency in Hz> <trace file>
```

