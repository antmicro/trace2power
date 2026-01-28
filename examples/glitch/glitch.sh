#!/bin/bash
# Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

# Build simulation files
iverilog -lglitch.v glitch_tb.v -oglitch_tb.out

# Run simulation and generate VCD trace file
./glitch_tb.out

# Process VCD file into per clock cycle TCL glitch power activity.
# --only-glitches causes trace2power to only export activities of signals where TC >= 2.
# --clock-name CLOCK_NAME allows trace2power to ignore clock signal activity.
mkdir -p out_tcl
trace2power \
    --clk-freq 200000000 \
    --output-format tcl \
    --limit-scope glitch_tb \
    --output out_tcl \
    --per-clock-cycle \
    --only-glitches \
    --clock-name clk \
    glitch_tb.vcd

mkdir -p out_saif
trace2power \
    --clk-freq 200000000 \
    --output-format saif \
    --limit-scope glitch_tb \
    --output out_saif \
    --per-clock-cycle \
    --ignore-date \
    --ignore-version \
    --only-glitches \
    --clock-name clk \
    glitch_tb.vcd
