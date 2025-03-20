use std::{collections::HashMap, hash::Hash};
use wellen::{VarRef, GetItem};

use crate::stats::{PackedStats, SignalStats};
use crate::{HashVarRef, LookupPoint};

use super::{TraceVisitorAgent, TraceVisit, TraceVisitCtx};


/// Minimal stats, used for Tcl export, hashable to allow grouping
#[derive(Hash, Eq, PartialEq)]
struct TclStat {
    high_time: u32,
    trans_count_doubled: u32
}

impl From<&SignalStats> for TclStat {
    fn from(value: &SignalStats) -> Self {
        Self {
            high_time: value.high_time,
            trans_count_doubled: value.trans_count_doubled
        }
    }
}

struct TclAgent<'a> {
    stats: &'a HashMap<HashVarRef, Vec<PackedStats>>,
    span_index: usize,
    grouped_stats: HashMap<TclStat, Vec<String>>,
    scope: Vec<String>,
}

impl<'a> TclAgent<'a> {
    fn new(stats: &'a HashMap<HashVarRef, Vec<PackedStats>>, span_index: usize) -> Self {
        Self {
            stats,
            span_index,
            grouped_stats: HashMap::new(),
            scope: Vec::new()
        }
    }
}

impl<'w, W> TraceVisitorAgent<'w, W> for TclAgent<'w> where W: std::io::Write {
    type Error = std::io::Error;

    fn enter_net(&mut self, ctx: &mut TraceVisitCtx<W>, var_ref: VarRef)
        -> Result<(), Self::Error>
    {
        let net = ctx.waveform.hierarchy().get(var_ref);
        let scope_str = self.scope.join(".");

        let stats = &self.stats[&HashVarRef(var_ref)];

        let fname = format!(
            "{}/{}",
            scope_str,
            net.name(ctx.waveform.hierarchy())
        );

        match &stats[self.span_index] {
            PackedStats::OneBit{duration: _, stats} => {
                self.grouped_stats.entry(TclStat::from(stats))
                    .or_insert_with(|| vec![])
                    .push(fname);
            },
            PackedStats::Vector{duration: _, stats} => for (idx, stat) in stats.iter().enumerate() {
                self.grouped_stats.entry(TclStat::from(stat))
                    .or_insert_with(|| vec![])
                    .push(format!("{}[{}]", fname, idx));
            }
        }

        Ok(())
    }

    fn enter_scope(&mut self, ctx: &mut TraceVisitCtx<W>, scope: &'w wellen::Scope)
            -> Result<(), Self::Error> {
        self.scope.push(scope.name(ctx.waveform.hierarchy()).into());
        Ok(())
    }

    fn exit_scope(&mut self, _ctx: &mut TraceVisitCtx<W>, _scope: &'w wellen::Scope)
            -> Result<(), Self::Error> {
        self.scope.pop();
        Ok(())
    }
}

pub fn export<W>(
    ctx: &crate::Context,
    mut out: W,
    iteration: usize
) -> std::io::Result<()>
    where W: std::io::Write
{
    let hier = ctx.wave.hierarchy();
    let time_end = *ctx.wave.time_table().last().unwrap();

    let netlist_root = match ctx.top_scope {
        Some(scope_ref) => hier.get(scope_ref).full_name(hier)
            .split('.')
            .map(String::from)
            .collect::<Vec<_>>(),
        None => Vec::new()
    };

    let mut visitor_ctx = TraceVisitCtx {
        out: &mut out,
        waveform: &ctx.wave,
        netlist_root,
        top_module: &ctx.top,
        netlist: ctx.netlist.as_ref(),
        netlist_prefix: Vec::new(),
        blackboxes_only: ctx.blackboxes_only,
        remove_virtual_pins: ctx.remove_virtual_pins,
    };

    let mut agent = TclAgent::new(&ctx.stats, iteration);
    if let LookupPoint::Scope(scope_ref) = ctx.lookup_point {
        let scope_name = ctx.wave.hierarchy().get(scope_ref).full_name(ctx.wave.hierarchy());
        let mut scope: Vec<_> = scope_name.split('.').map(ToString::to_string).collect();
        scope.pop();
        agent.scope = scope;
    }
    agent.visit_hierarchy(ctx.lookup_point, &mut visitor_ctx)?;

    let timescale = ctx.wave.hierarchy().timescale().unwrap();
    let timescale_norm =
        (timescale.factor as f64) * (10.0_f64).powf(timescale.unit.to_exponent().unwrap() as f64);

    writeln!(out, "proc set_pin_activity_and_duty {{}} {{")?;
    for (stats, pins) in agent.grouped_stats {
        let duty = (stats.high_time as f64) / ((time_end / ctx.num_of_iterations) as f64);
        let activity = ((stats.trans_count_doubled as f64) / 2.0_f64)
            / (((time_end / ctx.num_of_iterations) as f64) * timescale_norm / ctx.clk_period);

        writeln!(
            out,
            "  set_power_activity -pins \"{}\" -activity {} -duty {}",
            itertools::Itertools::intersperse(
                pins.into_iter()
                    .map(|n| n[ctx.scope_prefix_length..].replace('\\', "").replace('$', "\\$")),
                " ".into()
            ).collect::<String>(),
            activity,
            duty
        )?;
    }
    writeln!(out, "}}")?;

    Ok(())
}
