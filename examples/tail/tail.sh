#!/bin/bash
# Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

set -e

# Build simulation files
iverilog -g2012 -lbig_and.sv tail.sv -otail

# Run simulation and generate a VCD trace file
./tail

# Process the VCD file to SAIF
trace2power \
    --clk-freq 500000000 \
    --output-format saif \
    --limit-scope tail \
    --ignore-date \
    --ignore-version \
    --output out.saif \
    tail.vcd

# Process the VCD file to TCL
trace2power \
    --clk-freq 500000000 \
    --output-format tcl \
    --limit-scope tail \
    --output out.tcl \
    tail.vcd
