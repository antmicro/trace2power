# trace2power

Copyright (c) 2024-2025 [Antmicro](https://antmicro.com)

**trace2power** is a tool that can read VCD and FST signal traces and extract accumulated power
activity data for use with power analysis tools.

The tool can export the data into two distinct formats:
* **tcl** - a Tcl procedure containing calls for setting power activity data in
  [OpenSTA](https://github.com/parallaxsw/OpenSTA). This was the original intended usage and it
  allows significantly faster trace processing than loading VCDs directly to OpenSTA using
  `read_vcd`.
* **saif** - A "Backwards SAIF" file (IEEE 1801-2018 (Annex I.3)). This format should be compatible
  with more tools.

**trace2power** can also optimize out irrelevant signals or scopes. Those optimizations
will usually require providing a netlst file generated with
[Yosys](https://github.com/YosysHQ/yosys).

## Building

Note that `trace2power` requires rustc 1.80.1 or newer.

This is a standard cargo package. Cargo comes bundled with the standard Rust installation:
https://www.rust-lang.org/tools/install.

You can build the app and install it using
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
for a description of a flow used to test trace2power.

