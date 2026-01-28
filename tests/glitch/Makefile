# Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

EXAMPLE_FILES = $(EXAMPLES_DIRECTORY)/glitch/*
EXAMPLE_SCRIPT = ./glitch.sh

TCL_CHECK_SCRIPT = ./compare_tcl.sh
TCL_OUTPUT_FILES = out_tcl
TCL_GOLDEN_FILES = tcl

SAIF_OUTPUT_FILES = out_saif
SAIF_GOLDEN_FILES = saif

all: glitch

.PHONY: glitch
glitch:
	cp $(EXAMPLE_FILES) .
	$(EXAMPLE_SCRIPT)
	$(TCL_CHECK_SCRIPT) $(TCL_OUTPUT_FILES) $(TCL_GOLDEN_FILES)
	diff $(SAIF_OUTPUT_FILES) $(SAIF_GOLDEN_FILES)
