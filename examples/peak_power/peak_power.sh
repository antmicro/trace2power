#!/bin/bash
# Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

# Build simulation files
iverilog -lcounter.v peak_power.v -opeak_power

# Run simulation and generate a VCD trace file
./peak_power

# Process the VCD file into per clock cycle TCL power activities
# --per-clock-cycle causes trace2power to divide the trace file activity
# for each clock cycle period by given clock frequency
mkdir -p out_tcl
trace2power \
    --clk-freq 500000000 \
    --output-format tcl \
    --limit-scope peak_power \
    --per-clock-cycle \
    --output out_tcl/ \
    peak_power.vcd

mkdir -p out_saif
trace2power \
    --clk-freq 500000000 \
    --output-format saif \
    --limit-scope peak_power \
    --ignore-date \
    --ignore-version \
    --per-clock-cycle \
    --output out_saif/ \
    peak_power.vcd
