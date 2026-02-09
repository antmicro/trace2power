proc set_pin_activity_and_duty {} {
  set_power_activity -pins "counter0/rst" -activity 0.001996007984031936 -duty 0.001996007984031936
  set_power_activity -pins "counter0/pulse16" -activity 0.12375249500998003 -duty 0.06187624750499002
  set_power_activity -pins "counter0/pulse8" -activity 0.24750499001996007 -duty 0.12375249500998003
  set_power_activity -pins "counter0/pulse4" -activity 0.499001996007984 -duty 0.249500998003992
  set_power_activity -pins "counter0/cnt[4]" -activity 0.06187624750499002 -duty 0.4880239520958084
  set_power_activity -pins "counter0/cnt[3]" -activity 0.12375249500998003 -duty 0.49500998003992014
  set_power_activity -pins "counter0/cnt[2]" -activity 0.249500998003992 -duty 0.49600798403193613
  set_power_activity -pins "counter0/cnt[1]" -activity 0.499001996007984 -duty 0.499001996007984
  set_power_activity -pins "counter0/cnt[0] counter0/pulse2" -activity 0.998003992015968 -duty 0.499001996007984
  set_power_activity -pins "counter0/clk" -activity 2 -duty 0.5
}
