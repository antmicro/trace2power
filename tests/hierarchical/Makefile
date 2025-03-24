# Copyright (c) 2025 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

EXAMPLE_FILES = $(EXAMPLES_DIRECTORY)/hierarchical/*
EXAMPLE_SCRIPT = ./hierarchical.sh

TCL_OUTPUT_FILE = out.tcl
TCL_GOLDEN_FILE = hierarchical.tcl
SAIF_OUTPUT_FILE = out.saif
SAIF_GOLDEN_FILE = hierarchical.saif

all: hierarchical

hierarchical:
	cp $(EXAMPLE_FILES) .
	$(EXAMPLE_SCRIPT)
	sort -bd $(TCL_OUTPUT_FILE) | diff - $(TCL_GOLDEN_FILE)
	diff $(SAIF_OUTPUT_FILE) $(SAIF_GOLDEN_FILE)
