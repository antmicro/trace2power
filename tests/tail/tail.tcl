proc set_pin_activity_and_duty {} {
  set_power_activity -pins "dut/d dut/o" -activity 0.14285714285714285 -duty 0.7142857142857143
  set_power_activity -pins "dut/c" -activity 0.14285714285714285 -duty 0.7857142857142857
  set_power_activity -pins "dut/b" -activity 0.14285714285714285 -duty 0.8571428571428571
  set_power_activity -pins "dut/a" -activity 0.14285714285714285 -duty 0.9285714285714286
}
