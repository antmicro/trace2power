use std::fmt::Debug;

use itertools::izip;
use wellen::{simple::Waveform, Signal, SignalValue, TimeTableIdx, SignalRef};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct SignalStats {
    //pub name: String,
    pub trans_count_doubled: u32,
    pub clean_trans_count: u32,
    pub glitch_trans_count: u32,
    pub high_time: u32,
    pub low_time: u32,
    pub x_time: u32,
    pub z_time: u32,
}

impl Default for SignalStats {
    fn default() -> Self {
        Self {
            trans_count_doubled: 0,
            clean_trans_count: 0,
            glitch_trans_count: 0,
            high_time: 0,
            low_time: 0,
            x_time: 0,
            z_time: 0,
        }
    }
}

impl SignalStats {
    fn modify_time_stat_of_value<'s, F>(&'s mut self, val: char, f: F) where F: FnOnce(u32) -> u32 {
        match val {
            '1' => self.high_time = f(self.high_time),
            '0' => self.low_time = f(self.low_time),
            'x' => self.x_time = f(self.x_time),
            'z' => self.z_time = f(self.z_time),
            _ => panic!("Invalid value"),
        }
    }
}

impl SignalStats {
    fn is_glitch<'s>(&'s mut self) -> bool {
        self.clean_trans_count >= 2 || self.glitch_trans_count >= 2 || self.trans_count_doubled > 2
    }
}

impl SignalStats {
    fn clear<'s>(&'s mut self) {
        self.trans_count_doubled = 0;
        self.clean_trans_count = 0;
        self.glitch_trans_count = 0;
        self.high_time = 0;
        self.low_time = 0;
        self.x_time = 0;
        self.z_time = 0;
    }
}

fn val_at(ti: TimeTableIdx, sig: &Signal) -> SignalValue {
    let offset = sig.get_offset(ti).unwrap();
    return sig.get_value_at(&offset, 0)
}

fn time_value_at(wave: &Waveform, ti: TimeTableIdx) -> u64 {
    let time_stamp = wave.time_table()[ti as usize];
    return time_stamp;
}

pub fn calc_stats_for_each_time_span(
    wave: &Waveform,
    glitches_only: bool,
    clk_signal: Option<SignalRef>,
    sig_ref: SignalRef,
    num_of_iterations: u64) -> Vec<PackedStats>
{
    let mut stats = Vec::with_capacity(num_of_iterations as usize);
    let time_span = (*wave.time_table().last().unwrap()) / num_of_iterations;

    stats.extend((0..num_of_iterations).into_par_iter().map(|index| {
        let first_time_stamp = index * time_span;
        let last_time_stamp = (index + 1) * time_span;
        return calc_stats(wave, glitches_only, clk_signal, sig_ref, first_time_stamp, last_time_stamp);
    }).collect::<Vec<PackedStats>>());

    return stats;
}

pub fn calc_stats(
    wave: &Waveform,
    glitches_only: bool,
    clk_signal: Option<SignalRef>,
    sig_ref: SignalRef,
    first_time_stamp: wellen::Time,
    last_time_stamp: wellen::Time) -> PackedStats
{
    let sig = wave.get_signal(sig_ref).unwrap();

    let n = sig.time_indices().len();
    if n == 0 {
        return PackedStats::Vector(Vec::new());
    }

    let mut prev_val = val_at(sig.get_first_time_idx().unwrap(), sig);
    
    let bits = prev_val.bits();
    
    // Check if bits are valid, otherwise value is a real number
    if bits == None {
        // TODO: add function handling real numbers
        return PackedStats::Vector(Vec::new());
    }

    let bit_len = bits.unwrap();

    let mut ss = Vec::<SignalStats>::with_capacity(bit_len as usize);
    // TODO: Consider rev on range
    for _ in 0..bit_len {
        ss.push(Default::default())
    }
    
    let mut current_value_entry_index: usize = 1;

    // Fast forward to relevant starting time stamp
    while current_value_entry_index < sig.time_indices().len() {
        let time_idx = sig.time_indices()[current_value_entry_index];
        
        if time_value_at(wave, time_idx) > first_time_stamp {
            break;
        }
        
        prev_val = val_at(time_idx, sig);
        
        current_value_entry_index += 1;
    }

    // For high time calculations to start only from specified first time stamp
    let mut prev_ts = first_time_stamp;
    
    // Accumulate statistics over desired time span
    while current_value_entry_index < sig.time_indices().len() {
        let time_idx = sig.time_indices()[current_value_entry_index];
        let val = val_at(time_idx, sig);
        let ts = time_value_at(wave, time_idx);
        current_value_entry_index += 1;

        if ts > last_time_stamp {
            break;
        }

        let val_str = val.to_bit_string().unwrap();
        let prev_val_str = prev_val.to_bit_string().unwrap();
        for (c, prev_c, i) in izip!(val_str.chars(), prev_val_str.chars(), 0..) {
            match (prev_c, c) {
                ('0', '1') | ('1', '0') => {
                    ss[i].clean_trans_count += 1;
                    ss[i].trans_count_doubled += 2;
                }
                (other @ _, 'x') | ('x', other @ _) => if other != 'x' {
                    ss[i].trans_count_doubled += 1;
                    ss[i].glitch_trans_count += 1;
                }
                (other @ _, 'z') | ('z', other @ _) => if other != 'z' {
                    ss[i].trans_count_doubled += 1;
                    if other == '0' {
                        ss[i].clean_trans_count += 1;
                    }
                },
                _ => if prev_c != c {
                    panic!("Unknown transition {prev_c} -> {c}")
                }
            }

            ss[i].modify_time_stat_of_value(prev_c, |v| v + (ts - prev_ts) as u32);
        }
        prev_ts = ts;
        prev_val = val;
    }

    for (prev_c, i) in izip!(prev_val.to_bit_string().unwrap().chars(), 0..) {
        ss[i].modify_time_stat_of_value(prev_c, |v| v + (last_time_stamp - (prev_ts as u64)) as u32);
    }

    if glitches_only {
        for stat in ss.iter_mut() {
            if !stat.is_glitch() || sig_ref == clk_signal.unwrap() {
                stat.clear();
            }
        }
    }

    // TODO: Figure out how the indexing direction is denoted
    ss.reverse();

    return if ss.len() == 1 {
        PackedStats::OneBit(ss.into_iter().next().unwrap())
    } else {
        PackedStats::Vector(ss)
    }
}

pub enum PackedStats {
    OneBit(SignalStats),
    Vector(Vec<SignalStats>)
}
