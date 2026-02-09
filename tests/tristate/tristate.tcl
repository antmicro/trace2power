proc set_pin_activity_and_duty {} {
  set_power_activity -pins "tristate0/out" -activity 0.5 -duty 0.2375
  set_power_activity -pins "tristate0/z_state" -activity 0.4875 -duty 0.25
  set_power_activity -pins "tristate0/dir" -activity 0.475 -duty 0.5
  set_power_activity -pins "tristate0/ctrl" -activity 0.975 -duty 0.5
  set_power_activity -pins "tristate0/clk" -activity 2 -duty 0.5
}
