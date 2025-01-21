use chrono::Utc;
use std::collections::HashMap;
use indoc::indoc;
use wellen::{GetItem, TimescaleUnit, Scope, Var, SignalRef};
use wellen::simple::Waveform;
use rayon::prelude::*;
use crate::stats::{calc_stats, SignalStats};

struct DisplayTimescaleUnit(TimescaleUnit);

impl std::fmt::Display for DisplayTimescaleUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TimescaleUnit::*;
        let s = match self.0 {
            FemtoSeconds => "fs",
            PicoSeconds => "ps",
            NanoSeconds => "ns",
            MicroSeconds => "us",
            MilliSeconds => "ms",
            Seconds => "s",
            Unknown => panic!("Unknown time unit")
        };
        f.write_str(s)
    }
}

fn write_indent<W>(out: &mut W, indent: usize) -> std::io::Result<()> where W: std::io::Write {
    for _ in 0..indent {
        write!(out, "  ")?;
    }
    Ok(())
}

fn export_net<'w, W>(
    out: &mut W,
    waveform: &'w Waveform,
    port: &'w Var,
    stats: &HashMap<String, Vec<SignalStats>>,
    indent: usize
) -> std::io::Result<()>
    where W: std::io::Write
{
    let my_stats = &stats[&port.full_name(waveform.hierarchy())];
    for (idx, stat) in my_stats.iter().enumerate() {
        let name = if my_stats.len() > 1 {
            format!("{}[{}]", port.name(waveform.hierarchy()), idx)
        } else {
            port.name(waveform.hierarchy()).into()
        }.replace('\\', "\\\\");
        write_indent(out, indent)?;
        write!(
            out,
            "({} (T0 {}) (T1 {}) (TX {}) (TZ {}) (TC {}) (IG {}))\n",
            name,
            stat.low_time,
            stat.high_time,
            stat.x_time,
            stat.z_time,
            stat.clean_trans_count,
            stat.glitch_trans_count
        )?;
    }

    Ok(())
}


fn export_scope<'w, W>(
    out: &mut W,
    waveform: &'w Waveform,
    scope: &'w Scope,
    stats: &HashMap<String, Vec<SignalStats>>,
    indent: usize
) -> std::io::Result<()>
    where W: std::io::Write
{
    let name = scope.name(waveform.hierarchy());
    let mut instance_empty = true;
    let begin_instance = |out: &mut W| {
        write_indent(out, indent)?;
        write!(out, "(INSTANCE {name}\n")?;
        std::io::Result::<()>::Ok(())
    };
    let begin_net = |out: &mut W| {
        write_indent(out, indent + 1)?;
        write!(out, "(NET\n")?;
        std::io::Result::<()>::Ok(())
    };
    for var_ref in scope.vars(waveform.hierarchy()) {
        if instance_empty {
            begin_instance(out)?;
            begin_net(out)?;
            instance_empty = false;
        }
        let var = waveform.hierarchy().get(var_ref);
        export_net(out, waveform, var, &stats, indent + 2)?;
    }
    if !instance_empty {
        write_indent(out, indent + 1)?;
        write!(out, ")\n")?;
    }
    for scope_ref in scope.scopes(waveform.hierarchy()) {
        if instance_empty {
            begin_instance(out)?;
            instance_empty = false;
        }
        let scope = waveform.hierarchy().get(scope_ref);
        export_scope(out, waveform, scope, stats, indent + 1)?;
    }
    if instance_empty {
        return Ok(()); // Write only
    }
    write_indent(out, indent)?;
    write!(out, ")\n")
}

pub fn export<W>(
    waveform: &Waveform,
    all_sig_refs: Vec<SignalRef>,
    all_names: Vec<String>,
    _clk_period: f64,
    mut out: W
) -> std::io::Result<()>
    where W: std::io::Write
{
    let time_end = *waveform.time_table().last().unwrap();
    let timescale = waveform.hierarchy().timescale().unwrap();

    let var_stats: HashMap<String, Vec<SignalStats>> = all_sig_refs.par_iter()
        .map(|sig_ref| waveform.get_signal(*sig_ref).unwrap())
        .zip(all_names)
        .map(|(sig, fname)| (fname.clone(), calc_stats(sig, fname, time_end)))
        .collect();

    let now = Utc::now();

    write!(
        out,
        indoc!("
            (SAIFILE
              (SAIFVERSION \"2.0\")
              (DIRECTION \"backward\")
              (DESIGN )
              (DATE \"{}\")
              (PROGRAM_NAME \"{}\")
              (VERSION \"{}\")
              (DIVIDER / )
              (TIMESCALE {}{})
              (DURATION {})
        "),
        now.format("%a %b %-d %T %Y"),
        clap::crate_name!(),
        clap::crate_version!(),
        timescale.factor, DisplayTimescaleUnit(timescale.unit),
        time_end
    )?;

    for scope_ref in waveform.hierarchy().scopes() {
        let scope = waveform.hierarchy().get(scope_ref);
        export_scope(&mut out, waveform, scope, &var_stats, 1)?;
    }

    write!(out, ")\n")?;

    Ok(())
}
