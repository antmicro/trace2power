use itertools::izip;
use wellen::{Signal, SignalValue, TimeTableIdx};

#[derive(Debug, Clone)]
pub struct SignalStats {
    pub name: String,
    pub trans_count_doubled: u32,
    pub high_time: u32,
}

fn val_at(ti: TimeTableIdx, sig: &Signal) -> (SignalValue, TimeTableIdx) {
    let offset = sig.get_offset(ti).unwrap();
    (sig.get_value_at(&offset, 0), sig.get_time_idx_at(&offset))
}

pub fn calc_stats(sig: &Signal, name: String, time_end: wellen::Time) -> Vec<SignalStats> {
    let n = sig.time_indices().len();
    if n == 0 {
        return vec![];
    }

    let (mut prev_val, mut prev_ts) = val_at(sig.get_first_time_idx().unwrap(), sig);
    let bit_len = prev_val.bits().unwrap();
    let mut ss = Vec::<SignalStats>::with_capacity(bit_len as usize);
    for i in 0..bit_len {
        ss.push(SignalStats {
            name: name.clone()
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
