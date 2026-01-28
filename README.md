# trace2power

Copyright (c) 2024-2026 [Antmicro](https://antmicro.com)

**trace2power** reads VCD and FST signal traces and extracts accumulated power
activity data for use with power analysis tools.

The tool can export data into two distinct formats:
* **tcl** - a Tcl procedure containing calls for setting power activity data in
  [OpenSTA](https://github.com/parallaxsw/OpenSTA). This was the original intended usage and it
  allows significantly faster trace processing than loading VCDs directly to OpenSTA using
  `read_vcd`.
* **saif** - A "Backwards SAIF" file (IEEE 1801-2018 (Annex I.3)). This format should be compatible
  with more tools.

**trace2power** can also optimize out irrelevant signals or scopes. Those optimizations
will usually require providing a netlist file generated with
[Yosys](https://github.com/YosysHQ/yosys).

## Installing

`trace2power` requires [Rust](https://www.rust-lang.org/tools/install) 1.80.1 or newer.

You can install it using Cargo:

```bash
cargo install trace2power
```

## Building

If you need a version that is not published on [crates.io](https://crates.io/), you can also clone the repository and build the app using:
```bash
cargo install --path .
```
or just build the binary:
```bash
cargo build --release
```
The binary will be located at `target/release/trace2power`

## Usage

```bash
trace2power [OPTIONS] --clk-freq <CLK_FREQ> <INPUT_FILE>
```

Run `trace2power --help` for detailed descriptions of available options.

## Examples

Check out [the examples README](examples/README.md) for instructions for running test examples and
for a description of a flow used to test `trace2power`.

