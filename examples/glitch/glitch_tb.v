// Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

`timescale 1ns/1ns

module glitch_tb;

  reg clk = 0;
  reg a, b, c;
  wire out;

  glitch glitch0(.*);

  always #2.5 clk = ~clk;

  reg [3:0] i;

  initial
  begin
      $dumpfile("glitch_tb.vcd");
      $dumpvars(0, glitch_tb);

      for (i = 0; i < 4; i = i + 1)
      begin
          {a, c} = i;
          #5 b = 1;
          #5 b = 0;
          #5 ;
      end

      $finish;
  end
endmodule
