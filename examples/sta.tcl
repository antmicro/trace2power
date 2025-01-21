set out_dir $::env(PROJECT_DIR)/out

read_liberty $::env(ORFS)/flow/platforms/sky130hd/lib/sky130_fd_sc_hd__tt_025C_1v80.lib
read_verilog $out_dir/$::env(DESIGN)_synth.v
link_design $::env(DESIGN_TOP)
read_sdc $::env(PROJECT_DIR)/constraints.sdc
if { [info exists ::env(POWER_ACTIVITY_FMT)] } {
    set power_activity_fmt $::env(POWER_ACTIVITY_FMT)
} else {
    set power_activity_fmt "vcd"
}

if { $power_activity_fmt == "vcd" } {
    read_vcd -scope $::env(DESIGN_SCOPE) $out_dir/$::env(DESIGN).vcd
} elseif { $power_activity_fmt == "saif" } {
    read_saif -scope $::env(DESIGN_SCOPE) $out_dir/$::env(DESIGN).saif
} elseif { $power_activity_fmt == "tcl" } {
    # TODO: Fix -pins -> -input_ports ?
    source $out_dir/$::env(DESIGN).tcl
    set_pin_activity_and_duty
} elseif { $power_activity_fmt == "null" } {
} else {
    echo "INCORRECT ACTIVITY FORMAT"
}

file mkdir $out_dir
report_power > $::env(PROJECT_DIR)/out/power_report_$power_activity_fmt.txt
