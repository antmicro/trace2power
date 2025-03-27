module glitch(
    // inputs
    input clk_s,
    input a,
    input b,
    input c,
    //outputs
    output out
    );

    wire x, y, z;
    
    assign #1 x = a & b;
    assign #1 y = ~b;
    assign #1 z = y & c;
    assign #1 out = x | z;

endmodule