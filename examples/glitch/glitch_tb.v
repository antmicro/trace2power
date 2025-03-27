`timescale 1ns/1ps

module glitch_tb;

    parameter clk_period = 5.0;
    parameter clk_period2 = clk_period / 2.0;

    integer k;

    reg clk;
    reg a, b, c;
    wire out;

    glitch glitch0 (
        .clk_s(clk),
        .a(a),
        .b(b),
        .c(c),
        .out(out)
    );

    always #clk_period2 clk = ~clk;

    initial
    begin
        clk = 0;
    end

    initial
    begin
        for (k = 0; k < 4; k=k+1)
        begin
            {a, c} = k;
            #clk_period b = 1;
            #clk_period b = 0;
            #clk_period ;
        end

        $finish;
    end

    initial
    begin
        $dumpfile("glitch_tb.vcd");
        $dumpvars(0, glitch_tb);
    end

endmodule