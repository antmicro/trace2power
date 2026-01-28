// Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

`timescale 1ns/1ns

module tail();
  logic a, b, c, d, o;

  big_and dut(a, b, c, d, o);

  task automatic set(logic a_, logic b_, logic c_, logic d_);
    a = a_;
    b = b_;
    c = c_;
    d = d_;
  endtask

  initial begin
    $dumpfile("tail.vcd");
    $dumpvars(0, dut);

    set(0, 0, 0, 0);

    #1 set(1, 0, 0, 0); // o = 0
    #1 set(1, 1, 0, 0); // o = 0
    #1 set(1, 1, 1, 0); // o = 0
    #1 set(1, 1, 1, 1); // o = 1

    #10 $finish;
  end

endmodule
