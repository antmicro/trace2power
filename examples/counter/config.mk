export PLATFORM               = sky130hd

export DESIGN_NICKNAME        = counter
export DESIGN_NAME            = counter

export VERILOG_FILES         = $(PROJECT_DIR)/counter.v
export SDC_FILE              = $(PROJECT_DIR)/constraints.sdc

export CORE_UTILIZATION       = 40
export CORE_ASPECT_RATIO      = 1
export CORE_MARGIN            = 2
export PLACE_DENSITY_LB_ADDON = 0.20

export ENABLE_DPO = 0

export TNS_END_PERCENT        = 100
