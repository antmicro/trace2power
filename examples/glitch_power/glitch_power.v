// Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

`timescale 1ns/1ns

module glitch(
  input wire a,
  input wire b,
  output wire out
);
  wire x, y;

  assign #1 x = a;
  assign #2 y = b;

  // Different path delays cause `out` to be updated possibly
  // many times during one clock cycle.
  assign out = x | y;
endmodule

module glitch_power;

  reg clk = 0;
  reg a = 0;
  reg b = 0;
  wire out;

  glitch glitch(.*);

  always #2.5 clk = ~clk;

  initial begin
    $dumpfile("glitch_power.vcd");
    $dumpvars();

    #5 a = 1; b = 0;

    #5 a = 0; b = 1;

    #5 $finish;
  end
endmodule
