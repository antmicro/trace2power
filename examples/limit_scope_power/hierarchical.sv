// Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

module add(
  input logic clk,
  input logic rst_n,
  input logic [3:0] a,
  input logic [3:0] b,
  output logic [3:0] result
);
  logic [3:0] sum;

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
  input logic [3:0] a,
  input logic [3:0] b,
  output logic [3:0] result
);
  logic [3:0] sum;

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
  input logic [3:0] a,
  input logic [3:0] b,
  input logic [3:0] c,
  input logic [3:0] d,
  output logic [3:0] result
);
  logic [3:0] add_res;
  logic [3:0] mul_res;

  add adder1(.result(add_res), .*);
  mul multiplier1(.a(c),.b(d), .clk(clk), .rst_n(rst_n), .result(mul_res));
  assign result = add_res * mul_res;
endmodule
