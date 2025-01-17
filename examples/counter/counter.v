// Copyright (c) 2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

module counter(
  input wire clk,
  input wire rst,
  output reg pulse2,
  output reg pulse4,
  output reg pulse8,
  output reg pulse16
);
  reg[4:0] cnt;

  always @* begin
    pulse2 = cnt[0] == 1'd1;
    pulse4 = cnt[1:0] == 2'd3;
    pulse8 = cnt[2:0] == 3'd7;
    pulse16 = cnt[3:0] == 4'd15;
  end

  always @(posedge clk or posedge rst) begin
    if (rst) begin
      cnt <= 0;
    end else begin
      cnt <= cnt + 1;
    end
  end
endmodule
