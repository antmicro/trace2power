#!/bin/bash
# Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

set -e

# Build simulation files
iverilog -g2012 -lhierarchical.sv hierarchical_tb.sv -ohierarchical_tb

# Run simulation and generate a VCD trace file
./hierarchical_tb

# Process the VCD file to SAIF
trace2power \
    --clk-freq 500000000 \
    --output-format saif \
    --limit-scope hierarchical_tb \
    --ignore-date \
    --ignore-version \
    --output out.saif \
    hierarchical.vcd

# Process the VCD file to TCL
trace2power \
    --clk-freq 500000000 \
    --output-format tcl \
    --limit-scope hierarchical_tb \
    --output out.tcl \
    hierarchical.vcd
