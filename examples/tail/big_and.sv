// Copyright (c) 2024-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

module big_and(
  input logic a,
  input logic b,
  input logic c,
  input logic d,
  output wire o
);
  and and0(o, a, b, c, d);
endmodule
