// Copyright (c) 2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

`timescale 1ns/1ns

module hierarchical_tb();
  logic clk, rst_n;
  logic [31:0] a, b, c, result;

  hierarchical dut(.*);

  initial clk = 1'b0;
  always #1 clk = ~clk;

  task automatic await_clk_high();
    wait (clk == 1);
  endtask

  task automatic await_clk_low();
    wait (clk == 0);
  endtask

  task automatic reset(int hold = 1);
    await_clk_low();
    rst_n = 1'b0;
    repeat(hold) begin
      await_clk_low();
      await_clk_high();
    end
    rst_n = 1'b1;
    await_clk_low();
  endtask

  task automatic calculate(logic [31:0] a_, logic [31:0] b_, logic [31:0] c_);
    await_clk_low();
    a = a_;
    b = b_;
    c = c_;
    await_clk_high();
    await_clk_low();
  endtask

  initial begin
    static int seed = 200;
    static logic rval = $random(seed) % 10000;

    $dumpfile("hierarchical.vcd");
    $dumpvars(0, dut);

    reset(16);
    repeat(32)
      calculate(rval, rval, rval);
    //calculate(0, 0, 0);
    //calculate(2, 2, 2);
    //calculate(200, 4, 15);
    //calculate(7898, 91, 10202);
    //calculate(898, 29, 0);
    //calculate(73911, 111111, 19);
    $finish;
  end
endmodule
