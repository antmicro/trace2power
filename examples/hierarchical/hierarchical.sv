// Copyright (c) 2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

module add(
  input logic clk,
  input logic rst_n,
  input logic [31:0] a,
  input logic [31:0] b,
  output logic [31:0] result
);
  logic [31:0] sum;

  assign r = a + b;

  always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n)
      result <= 16'b0;
    else
      result <= r;
  end
endmodule

module mul(
  input logic clk,
  input logic rst_n,
  input logic [31:0] a,
  input logic [31:0] b,
  output logic [31:0] result
);
  logic [31:0] sum;

  assign r = a * b;

  always_ff @(posedge clk or negedge rst_n) begin
    if (!rst_n)
      result <= 16'b0;
    else
      result <= r;
  end
endmodule

module hierarchical(
  input logic clk,
  input logic rst_n,
  input logic [31:0] a,
  input logic [31:0] b,
  input logic [31:0] c,
  output logic [31:0] result
);
  logic [31:0] add_res;

  add adder1(.result(add_res), .*);
  mul multiplier1(.a(add_res), .b(c), .*);
endmodule
