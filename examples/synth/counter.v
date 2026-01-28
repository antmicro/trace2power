// Copyright (c) 2025-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

(* src = "/ci/examples/counter/counter.v:4.1-28.10" *)
(* top =  1  *)
(* hdlname = "counter" *)
module counter(clk, rst, pulse2, pulse4, pulse8, pulse16);
  wire _00_;
  wire _01_;
  wire _02_;
  wire _03_;
  wire _04_;
  wire _05_;
  input clk;
  wire clk;
  wire \cnt[1] ;
  wire \cnt[2] ;
  wire \cnt[3] ;
  output pulse16;
  wire pulse16;
  output pulse2;
  wire pulse2;
  output pulse4;
  wire pulse4;
  output pulse8;
  wire pulse8;
  input rst;
  wire rst;
  sky130_fd_sc_hd__inv_1 _06_ (
    .A(pulse2),
    .Y(_00_)
  );
  sky130_fd_sc_hd__nand3_1 _07_ (
    .A(pulse2),
    .B(\cnt[2] ),
    .C(\cnt[1] ),
    .Y(_03_)
  );
  sky130_fd_sc_hd__xnor2_1 _08_ (
    .A(\cnt[3] ),
    .B(_03_),
    .Y(_01_)
  );
  sky130_fd_sc_hd__inv_1 _09_ (
    .A(rst),
    .Y(_02_)
  );
  sky130_fd_sc_hd__and3_1 _10_ (
    .A(\cnt[3] ),
    .B(\cnt[2] ),
    .C(pulse4),
    .X(pulse16)
  );
  sky130_fd_sc_hd__ha_1 _11_ (
    .A(\cnt[1] ),
    .B(pulse2),
    .COUT(pulse4),
    .SUM(_04_)
  );
  sky130_fd_sc_hd__ha_1 _12_ (
    .A(\cnt[2] ),
    .B(pulse4),
    .COUT(pulse8),
    .SUM(_05_)
  );
  sky130_fd_sc_hd__dfrtp_1 \cnt[0]$_DFF_PP0_  (
    .CLK(clk),
    .D(_00_),
    .Q(pulse2),
    .RESET_B(_02_)
  );
  sky130_fd_sc_hd__dfrtp_1 \cnt[1]$_DFF_PP0_  (
    .CLK(clk),
    .D(_04_),
    .Q(\cnt[1] ),
    .RESET_B(_02_)
  );
  sky130_fd_sc_hd__dfrtp_1 \cnt[2]$_DFF_PP0_  (
    .CLK(clk),
    .D(_05_),
    .Q(\cnt[2] ),
    .RESET_B(_02_)
  );
  sky130_fd_sc_hd__dfrtp_1 \cnt[3]$_DFF_PP0_  (
    .CLK(clk),
    .D(_01_),
    .Q(\cnt[3] ),
    .RESET_B(_02_)
  );
endmodule
