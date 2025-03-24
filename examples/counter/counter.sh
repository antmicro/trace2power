#!/bin/bash
# Copyright (c) 2025 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

set -e

# Build simulation files
iverilog -lcounter.v counter_tb.v -ocounter_tb

# Run simulation and generate a VCD trace file
./counter_tb

# Process the VCD file to SAIF
trace2power \
    --clk-freq 500000000 \
    --output-format saif \
    --limit-scope counter_tb \
    --ignore-date \
    --output out.saif \
    counter.vcd

# Process the VCD file to TCL
trace2power \
    --clk-freq 500000000 \
    --output-format tcl \
    --limit-scope counter_tb \
    --output out.tcl \
    counter.vcd
