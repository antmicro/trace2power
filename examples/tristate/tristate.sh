#!/bin/bash
# Copyright (c) 2025 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

set -e

# Build simulation files
iverilog -g2012 -ltristate.v tristate_tb.sv -otristate_tb

# Run simulation and generate a VCD trace file
./tristate_tb

# Process the VCD file to SAIF
trace2power \
    --clk-freq 500000000 \
    --output-format saif \
    --limit-scope tristate_tb \
    --ignore-date \
    --output out.saif \
    tristate.vcd

# Process the VCD file to TCL
trace2power \
    --clk-freq 500000000 \
    --output-format tcl \
    --limit-scope tristate_tb \
    --output out.tcl \
    tristate.vcd
