// Copyright (c) 2024-2025 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use itertools::izip;
use rayon::prelude::*;
use std::collections::HashMap;
use wellen;

#[derive(Debug, Clone)]
struct SignalStats {
    name: String,
    trans_count_doubled: u32,
    high_time: u32,
}

#[derive(Parser)]
struct Cli {
    input_file: std::path::PathBuf,
    #[arg(short, long, value_parser = clap::value_parser!(f64))]
    clk_freq: f64,
}

fn val_at(
    ti: wellen::TimeTableIdx,
    sig: &wellen::Signal,
) -> (wellen::SignalValue, wellen::TimeTableIdx) {
    let offset = sig.get_offset(ti).unwrap();
    (sig.get_value_at(&offset, 0), sig.get_time_idx_at(&offset))
}

fn calc_stats(sig: &wellen::Signal, name: String, time_end: wellen::Time) -> Vec<SignalStats> {
    let n = sig.time_indices().len();
    if n == 0 || !name.contains("root.dram_ctrl.") {
        return vec![];
    }

    let (mut prev_val, mut prev_ts) = val_at(sig.get_first_time_idx().unwrap(), sig);
    let bit_len = prev_val.bits().unwrap();
    let mut ss = Vec::<SignalStats>::with_capacity(bit_len as usize);
    let name_tcl = name
        .clone()
        .replace("root.dram_ctrl.", "")
        .replace(".", "/");
    for i in 0..bit_len {
        ss.push(SignalStats {
            name: name_tcl.clone()
                + (if bit_len > 1 {
                    format!("[{}]", i)
                } else {
                    "".into()
                })
                .as_ref(),
            trans_count_doubled: 0,
            high_time: 0,
        })
    }

    for time_idx in sig.time_indices().iter() {
        let (val, ts) = val_at(*time_idx, sig);
        let val_str = val.to_bit_string().unwrap();
        let prev_val_str = prev_val.to_bit_string().unwrap();
        for (c, prev_c, i) in izip!(val_str.chars(), prev_val_str.chars(), 0..) {
            if prev_c != c {
                ss[i].trans_count_doubled +=
                    if c == 'x' || c == 'z' || prev_c == 'x' || prev_c == 'z' {
                        1
                    } else {
                        2
                    };
            }
            if prev_c == '1' {
                ss[i].high_time += ts - prev_ts;
            }
        }
        prev_ts = ts;
        prev_val = val;
    }

    for (prev_c, i) in izip!(prev_val.to_bit_string().unwrap().chars(), 0..) {
        if prev_c == '1' {
            ss[i].high_time += (time_end - (prev_ts as u64)) as u32;
        }
    }

    return ss;
}

const LOAD_OPTS: wellen::LoadOptions = wellen::LoadOptions {
    multi_thread: true,
    remove_scopes_with_empty_name: false,
};

fn process_trace(args: Cli) -> wellen::Result<()> {
    let mut wave =
        wellen::simple::read_with_options(args.input_file.to_str().unwrap(), &LOAD_OPTS)?;
    let (all_sig_refs, all_names): (Vec<_>, Vec<_>) = wave
        .hierarchy()
        .iter_vars()
        .map(|var| (var.signal_ref(), var.full_name(wave.hierarchy())))
        .collect();
    wave.load_signals_multi_threaded(&all_sig_refs[..]);
    let time_end = *wave.time_table().last().unwrap();
    let timescale = wave.hierarchy().timescale().unwrap();
    let timescale_norm =
        (timescale.factor as f64) * (10.0_f64).powf(timescale.unit.to_exponent().unwrap() as f64);

    let stats: Vec<_> = all_sig_refs
        .par_iter()
        .map(|sig_ref| wave.get_signal(*sig_ref).unwrap())
        .zip(all_names)
        .flat_map(|(sig, name)| calc_stats(sig, name, time_end))
        .collect();

    let clk_period = 1.0_f64 / args.clk_freq;
    let mut grouped_stats = HashMap::new();

    for stat in stats.iter() {
        grouped_stats
            .entry((stat.high_time, stat.trans_count_doubled))
            .or_insert_with(|| vec![])
            .push(stat.name.replace("$", "\\$"));
    }

    println!("proc set_pin_activity_and_duty {{}} {{");
    for ((high_time, trans_count_doubled), sig_names) in grouped_stats.iter_mut() {
        let duty = (*high_time as f64) / (time_end as f64);
        let activity = ((*trans_count_doubled as f64) / 2.0_f64)
            / ((time_end as f64) * timescale_norm / clk_period);
        println!(
            "  set_power_activity -pins \"{}\" -activity {} -duty {}",
            sig_names.join(" "),
            activity,
            duty
        );
    }
    println!("}}");

    Ok(())
}

fn main() {
    let args = Cli::parse();
    if let Err(e) = process_trace(args) {
        eprintln!("Failed to read waveform: {e}!");
    }
}
