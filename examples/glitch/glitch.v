// Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

`timescale 1ns/1ns

module glitch(
  input wire a,
  input wire b,
  input wire c,
  output wire out
);
  wire x, y, z;

  assign #1 x = a & b;
  assign #1 y = ~b;
  assign #1 z = y & c;
  assign #1 out = x | z;
endmodule
