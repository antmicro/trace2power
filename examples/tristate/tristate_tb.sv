`timescale 1ns/1ns

module tristate_tb();
  logic clk, dir, ctrl, out, z_state;

  tristate tristate0(
    .clk(clk),
    .dir(dir),
    .ctrl(ctrl),
    .out(out),
    .z_state(z_state)
  );

  initial clk = 0;
  always #1 clk = ~clk;

  task automatic await_clk_high();
    wait (clk == 1);
  endtask

  task automatic await_clk_low();
    wait (clk == 0);
  endtask

  task automatic cycle(logic dir_, logic ctrl_);
    dir = dir_;
    ctrl = ctrl_;
    await_clk_high();
    await_clk_low();
  endtask

  initial begin
    $dumpfile("tristate.vcd");
    $dumpvars(1, tristate0);

    repeat (10) begin
      cycle(0, 0);
      cycle(0, 1);
      cycle(1, 0);
      cycle(1, 1);
    end

    $finish;
  end
endmodule
