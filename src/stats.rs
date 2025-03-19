use itertools::izip;
use wellen::{Signal, simple::Waveform, SignalValue, TimeTableIdx};

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

fn val_at(ti: TimeTableIdx, sig: &Signal) -> SignalValue {
    let offset = sig.get_offset(ti).unwrap();
    return sig.get_value_at(&offset, 0)
}

fn time_value_at(wave: &Waveform, ti: TimeTableIdx) -> u64 {
    let time_stamp = wave.time_table()[ti as usize];
    return time_stamp;
}

pub fn calc_stats(wave: &Waveform, sig: &Signal, span_index: u32, first_time_stamp: wellen::Time, last_time_stamp: wellen::Time) -> PackedStats {
    let n = sig.time_indices().len();
    if n == 0 {
        return PackedStats::Vector(Vec::new());
    }

    let mut prev_val = val_at(sig.get_first_time_idx().unwrap(), sig);
    let mut prev_ts = time_value_at(wave, sig.get_first_time_idx().unwrap());

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
    
    for time_idx in sig.time_indices().iter() {
        let val = val_at(*time_idx, sig);
        let ts = time_value_at(wave, *time_idx);
     
        if ts < first_time_stamp {
            prev_ts = ts;
            prev_val = val;
            continue;
        }
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
