# Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

EXAMPLE_FILES = $(EXAMPLES_DIRECTORY)/counter/*
EXAMPLE_SCRIPT = ./counter.sh

TCL_OUTPUT_FILE = out.tcl
TCL_GOLDEN_FILE = counter.tcl
SAIF_OUTPUT_FILE = out.saif
SAIF_GOLDEN_FILE = counter.saif

all: counter

counter:
	cp $(EXAMPLE_FILES) .
	$(EXAMPLE_SCRIPT)
	sort -bd $(TCL_OUTPUT_FILE) | diff - $(TCL_GOLDEN_FILE)
	diff $(SAIF_OUTPUT_FILE) $(SAIF_GOLDEN_FILE)
