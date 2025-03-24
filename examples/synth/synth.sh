#!/bin/bash
# Copyright (c) 2025 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

set -e

# Process the VCD file to SAIF
trace2power \
    --clk-freq 500000000 \
    --output-format saif \
    --netlist counter.json \
    --remove-virtual-pins \
    --limit-scope counter_tb.counter0 \
    --ignore-date \
    --output out.saif \
    counter.vcd

# Process the VCD file to TCL
trace2power \
    --clk-freq 500000000 \
    --output-format tcl \
    --netlist counter.json \
    --limit-scope counter_tb.counter0 \
    --remove-virtual-pins \
    --output out.tcl \
    counter.vcd
