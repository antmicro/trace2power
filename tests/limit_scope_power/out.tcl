proc set_pin_activity_and_duty {} {
  set_power_activity -pins "dut/adder1/result[0]" -activity 0.020833333333333332 -duty 0
  set_power_activity -input_ports "dut/adder1/result[0]" -activity 0.020833333333333332 -duty 0
  set_power_activity -pins "dut/a[0] dut/a[1] dut/a[2] dut/a[3] dut/b[0] dut/b[1] dut/b[2] dut/b[3] dut/c[0] dut/c[1] dut/c[2] dut/c[3] dut/clk dut/d[0] dut/d[1] dut/d[2] dut/d[3] dut/rst_n dut/result[0] dut/result[1] dut/result[2] dut/result[3] dut/mul_res[0] dut/mul_res[1] dut/mul_res[2] dut/mul_res[3] dut/add_res[0] dut/add_res[1] dut/add_res[2] dut/add_res[3] dut/adder1/result[1] dut/adder1/result[2] dut/adder1/result[3] dut/multiplier1/a[0] dut/multiplier1/a[1] dut/multiplier1/a[2] dut/multiplier1/a[3] dut/multiplier1/b[0] dut/multiplier1/b[1] dut/multiplier1/b[2] dut/multiplier1/b[3] dut/multiplier1/clk dut/multiplier1/rst_n dut/multiplier1/r dut/multiplier1/result[0] dut/multiplier1/result[1] dut/multiplier1/result[2] dut/multiplier1/result[3]" -activity 0 -duty 0
  set_power_activity -input_ports "dut/a[0] dut/a[1] dut/a[2] dut/a[3] dut/b[0] dut/b[1] dut/b[2] dut/b[3] dut/c[0] dut/c[1] dut/c[2] dut/c[3] dut/clk dut/d[0] dut/d[1] dut/d[2] dut/d[3] dut/rst_n dut/result[0] dut/result[1] dut/result[2] dut/result[3] dut/mul_res[0] dut/mul_res[1] dut/mul_res[2] dut/mul_res[3] dut/add_res[0] dut/add_res[1] dut/add_res[2] dut/add_res[3] dut/adder1/result[1] dut/adder1/result[2] dut/adder1/result[3] dut/multiplier1/a[0] dut/multiplier1/a[1] dut/multiplier1/a[2] dut/multiplier1/a[3] dut/multiplier1/b[0] dut/multiplier1/b[1] dut/multiplier1/b[2] dut/multiplier1/b[3] dut/multiplier1/clk dut/multiplier1/rst_n dut/multiplier1/r dut/multiplier1/result[0] dut/multiplier1/result[1] dut/multiplier1/result[2] dut/multiplier1/result[3]" -activity 0 -duty 0
  set_power_activity -pins "dut/adder1/clk" -activity 2 -duty 0.5
  set_power_activity -input_ports "dut/adder1/clk" -activity 2 -duty 0.5
  set_power_activity -pins "dut/adder1/rst_n" -activity 0.020833333333333332 -duty 0.6770833333333334
  set_power_activity -input_ports "dut/adder1/rst_n" -activity 0.020833333333333332 -duty 0.6770833333333334
  set_power_activity -pins "dut/adder1/a[0] dut/adder1/b[0]" -activity 0.010416666666666666 -duty 0.6666666666666666
  set_power_activity -input_ports "dut/adder1/a[0] dut/adder1/b[0]" -activity 0.010416666666666666 -duty 0.6666666666666666
  set_power_activity -pins "dut/adder1/a[1] dut/adder1/a[2] dut/adder1/a[3] dut/adder1/b[1] dut/adder1/b[2] dut/adder1/b[3] dut/adder1/r" -activity 0.010416666666666666 -duty 0
  set_power_activity -input_ports "dut/adder1/a[1] dut/adder1/a[2] dut/adder1/a[3] dut/adder1/b[1] dut/adder1/b[2] dut/adder1/b[3] dut/adder1/r" -activity 0.010416666666666666 -duty 0
}
