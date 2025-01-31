# trace2power examples

This directory contains example projects for testing trace2power.

## The flow

All examples are testing with the following flow:
* Synthesize the DUT design using
  [OpenROAD flow scripts](https://github.com/The-OpenROAD-Project/OpenROAD-flow-scripts)
* Export a JSON netlist from the post-symthesis verilog using
  [Yosys](https://github.com/YosysHQ/yosys)
* Simulate a testbench containing the synthesised DUT using Icarus Verilog. The simulation will
  produce a trace file (VCD).
* Run `trace2power` and export SAIF and TCL files.
* Run [OpenSTA](https://github.com/parallaxsw/OpenSTA) and perform power analysis based on
  * no power activity input (null)
  * `read_vcd` input (vcd)
  * `read_saif input (saif)
  * call to `set_pin_activity_and_duty` procedure (tcl)

  For the testing purposes the output report for saif and tcl inputs must match the report from
  the vcd input.

The flow is implemented in `examples/test.py` and the script used for setting up OpenSTA is
`examples/sta.tcl`.

## Running the flow

There are four examples available:
* counter
* tristate
* hierarchical
* tail

First, set the following environmental variables:
* `ORFS` - Must point to a directory containing
  [`OpenROAD-flow-scripts`](https://github.com/The-OpenROAD-Project/OpenROAD-flow-scripts).
  You must build included copy of Yosys.
* `TRACE2POWER` - path to `trace2power` executable.
* `OPENSTA` (optional) - Path to [OpenSTA](https://github.com/parallaxsw/OpenSTA) executable.
  If not set, it will be assumed it's built as a part of `ORFS`.
* `IVERILOG` (optional) - Path to Icarus Verilog executable. Defaults to `iverilog`.
* `YOSYS` (optional) - Path to yosys executable. Defaults to `${ORFS}/tools/yosys/yosys`

To run complete flows for all examples:
```bash
python examples/test.py
```
To run selected examples (eg. counter and tristate)
```bash
python examples/test.py counter tristate
```

You can add `-s <phase>`/`--start-from <phase>` to specify a phase of the flowfrom which to start.
There are four available starting points:
* `synth` - start from synthesis (full flow)
* `sim` - start from simulation (assume we are already after synthesis)
* `trace2power` - start from running `trace2power` (assume we are already after synthesis and
  simulation)
* `sta` - start from running OpenSTA (assume all previous steps were already done)

You can also add `-f <fmt>`/`--formats <fmt>` options to override the selected formats for the flow:
* `tcl`
* `sta`

You can skip specific formats for specific examples with `--skip <example:format>`, eg.
```bash
python examples/test.py --skip hierarchical:tcl
```

At the end of execution a summary will be presented, eg.:
```
SUMMARY:
counter:
* ✔ tcl: PASS
* ✔ saif: PASS
tristate:
* ✔ tcl: PASS
* ✔ saif: PASS
hierarchical:
* ✔ tcl: PASS
* ✔ saif: PASS
tail:
* ✔ saif: PASS
* ✘ tcl: SKIPPED
```

All generated files wil be available under `examples/<project name>/out` directories. It's safe to
delete them.