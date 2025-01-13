use std::collections::HashMap;
use wellen::simple::Waveform;
use crate::stats::SignalStats;

pub fn export<W>(
    waveform: &Waveform,
    stats: &Vec<SignalStats>,
    clk_period: f64,
    mut out: W
) -> std::io::Result<()>
    where W: std::io::Write
{
    let mut grouped_stats = HashMap::new();
    for stat in stats.iter() {
        grouped_stats
            .entry((stat.high_time, stat.trans_count_doubled))
            .or_insert_with(|| vec![])
            .push(stat.name.replace("$", "\\$"));
    }

    let time_end = *waveform.time_table().last().unwrap();
    let timescale = waveform.hierarchy().timescale().unwrap();
    let timescale_norm =
        (timescale.factor as f64) * (10.0_f64).powf(timescale.unit.to_exponent().unwrap() as f64);

    writeln!(out, "proc set_pin_activity_and_duty {{}} {{")?;
    for ((high_time, trans_count_doubled), sig_names) in grouped_stats.iter_mut() {
        let duty = (*high_time as f64) / (time_end as f64);
        let activity = ((*trans_count_doubled as f64) / 2.0_f64)
            / ((time_end as f64) * timescale_norm / clk_period);
        writeln!(
            out,
            "  set_power_activity -pins \"{}\" -activity {} -duty {}",
            sig_names.join(" "),
            activity,
            duty
        )?;
    }
    writeln!(out, "}}")?;

    Ok(())
}
