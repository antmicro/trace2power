# Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

EXAMPLE_FILES = $(EXAMPLES_DIRECTORY)/tristate/*
EXAMPLE_SCRIPT = ./tristate.sh

TCL_OUTPUT_FILE = out.tcl
TCL_GOLDEN_FILE = tristate.tcl
SAIF_OUTPUT_FILE = out.saif
SAIF_GOLDEN_FILE = tristate.saif

all: tristate

tristate:
	cp $(EXAMPLE_FILES) .
	$(EXAMPLE_SCRIPT)
	diff $(TCL_OUTPUT_FILE) $(TCL_GOLDEN_FILE)
	diff $(SAIF_OUTPUT_FILE) $(SAIF_GOLDEN_FILE)
