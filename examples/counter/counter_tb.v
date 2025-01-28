// Copyright (c) 2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

`timescale 1ns/1ns

module counter_tb;
  reg clk, rst;
  wire pulse2, pulse4, pulse8, pulse16;

  counter counter0(clk, rst, pulse2, pulse4, pulse8, pulse16);
  always #1 clk = ~clk;

  initial begin
    $dumpfile("counter.vcd");
    $dumpvars(0, counter0);

    clk = 0;
    rst = 1;
    #2
    rst = 0;

    #1000 $finish;
  end
endmodule
