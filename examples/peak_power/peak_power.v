// Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

`timescale 1ns/1ns

module peak_power;
  reg clk;
  wire pulse2, pulse4, pulse8, pulse16;

  counter counter0(clk, pulse2, pulse4, pulse8, pulse16);
  always #1 clk = ~clk;

  initial begin
    $dumpfile("peak_power.vcd");
    $dumpvars(0, counter0);

    clk = 0;

    #20 $finish;
  end
endmodule
