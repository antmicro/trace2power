#!/bin/bash
# Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
# SPDX-License-Identifier: Apache-2.0

for f in $(ls $1); do
    if [ -f "$2/$f" ]; then
        diff "$1/$f" "$2/$f"
    fi
done
