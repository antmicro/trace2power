use std::collections::HashMap;
use rayon::prelude::*;
use itertools::*;

pub fn export<W>(
    ctx: crate::Context,
    mut out: W
) -> std::io::Result<()>
    where W: std::io::Write
{
    let time_end = *ctx.wave.time_table().last().unwrap();

    let stats: Vec<_> = ctx.all_sig_refs.par_iter()
        .map(|sig_ref| ctx.wave.get_signal(*sig_ref).unwrap())
        .zip(ctx.all_names)
        .flat_map(|(sig, name)| crate::stats::calc_stats(sig, name, time_end))
        .collect();

    let grouped_stats = stats.iter()
        .fold(HashMap::new(), |mut m, stat| {
            m.entry((stat.high_time, stat.trans_count_doubled))
            .or_insert_with(|| vec![])
            .push(stat.name.replace("$", "\\$"));
            m
        });

    let timescale = ctx.wave.hierarchy().timescale().unwrap();
    let timescale_norm =
        (timescale.factor as f64) * (10.0_f64).powf(timescale.unit.to_exponent().unwrap() as f64);

    writeln!(out, "proc set_pin_activity_and_duty {{}} {{")?;
    for ((high_time, trans_count_doubled), sig_names) in grouped_stats {
        let duty = (high_time as f64) / (time_end as f64);
        let activity = ((trans_count_doubled as f64) / 2.0_f64)
            / ((time_end as f64) * timescale_norm / ctx.clk_period);
        writeln!(
            out,
            "  set_power_activity -pins \"{}\" -activity {} -duty {}",
            sig_names.into_iter()
                .map(|n| n[ctx.scope_prefix_length..].to_string())
                .intersperse(" ".into())
                .collect::<String>(),
            activity,
            duty
        )?;
    }
    writeln!(out, "}}")?;

    Ok(())
}
