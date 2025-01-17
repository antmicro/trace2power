This example is simple counter module that outputs a single pulse on port `pulse[2|4|8|16]` every 2, 4, 8 or 16 cycles respectively.
This corresponds to duty cycles of 0.5, 0.25, 0.125 and 0.0625.

Running the example requires Icarus Verilog. To run the example:

```
./counter.sh
```

This should produce the following output (order of pins may be different):

```
proc set_pin_activity_and_duty {} {
  set_power_activity -pins "counter_tb.counter0.clk" -activity 1 -duty 0.5
  set_power_activity -pins "counter_tb.counter0.rst" -activity 0.000998003992015968 -duty 0.001996007984031936
  set_power_activity -pins "counter_tb.counter0.cnt[0]" -activity 0.03093812375249501 -duty 0.4880239520958084
  set_power_activity -pins "counter_tb.counter0.cnt[1]" -activity 0.06187624750499002 -duty 0.49500998003992014
  set_power_activity -pins "counter_tb.counter0.cnt[2]" -activity 0.124750499001996 -duty 0.49600798403193613
  set_power_activity -pins "counter_tb.counter0.cnt[3]" -activity 0.249500998003992 -duty 0.499001996007984
  set_power_activity -pins "counter_tb.counter0.cnt[4] counter_tb.counter0.pulse2" -activity 0.499001996007984 -duty 0.499001996007984
  set_power_activity -pins "counter_tb.counter0.pulse4" -activity 0.249500998003992 -duty 0.249500998003992
  set_power_activity -pins "counter_tb.counter0.pulse8" -activity 0.12375249500998003 -duty 0.12375249500998003
  set_power_activity -pins "counter_tb.counter0.pulse16" -activity 0.06187624750499002 -duty 0.06187624750499002
}
```

This is a TCL script that has a single function `set_pin_activity_and_duty` that is meant to be called by your OpenSTA TCL script.
This function sets up power activity via `set_power_activity` for each pin that was calculated from the supplied trace.
As per OpenSTA documentation arguments used for `set_power_activity` are:
- `-pins` list hierarchical pin names
- `-activity` means "number of transitions per clock cycle"
- `-duty` means "probability the signal is high", or simply duty cycle.

Each pin is assigned an activity and duty cycle.
For example, `pulse4` has activity and duty cycle of roughly 0.25, which corresponds to it being high 25% of the time and being toggled in 25% of all cycles on average. This matches our expectations as `pulse4` is supposed to be high every 4 cycles.
Note that activity values larger than 1 are permitted, e.g. if a signal toggles more than once during a clock cycle on average.

