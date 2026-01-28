// Copyright (c) 2024-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

module tristate(
  input wire clk,
  input wire dir, ctrl,
  output reg out,
  inout wire z_state
);
  always @(posedge clk) begin
    out <= dir ? ctrl : z_state;
  end

  assign z_state = dir ? ctrl : 1'bz;
endmodule
