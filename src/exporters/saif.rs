use chrono::Utc;
use std::collections::HashMap;
use indoc::indoc;
use wellen::{GetItem, TimescaleUnit, Scope, VarRef};
use crate::{indexed_name, Context};
use crate::stats::{PackedStats, SignalStats};

use crate::HashVarRef;
use super::{TraceVisitorAgent, TraceVisit, TraceVisitCtx};

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

/// Holds SAIF exporter's state corresponding to a given scope
struct ScopeCtx {
    name_escaped: String,
    instance_empty: bool,
}

struct SaifAgent<'a> {
    stats: &'a HashMap<HashVarRef, Vec<PackedStats>>,
    span_index: usize,
    scope_ctx: Vec<ScopeCtx>,
    indent: usize
}

impl<'a> SaifAgent<'a> {
    fn new(stats: &'a HashMap<HashVarRef, Vec<PackedStats>>, span_index: usize, indent: usize) -> Self {
        Self {
            stats,
            span_index,
            scope_ctx: Vec::new(),
            indent,
        }
    }
}

impl<'a> SaifAgent<'a> {
    fn get_ctx<'s>(&'s self) -> &'s ScopeCtx { self.scope_ctx.last().unwrap() }
    fn get_ctx_mut<'s>(&'s mut self) -> &'s mut ScopeCtx { self.scope_ctx.last_mut().unwrap() }
    fn get_parent_ctx_mut<'s>(&'s mut self) -> Option<&'s mut ScopeCtx> {
        let len = self.scope_ctx.len();
        if len >= 2 {
            Some(&mut self.scope_ctx[len - 2])
        } else {
            None
        }
    }

    fn write_indent<W>(&self, out: &mut W) -> std::io::Result<()> where W: std::io::Write {
        for _ in 0..self.indent {
            write!(out, "  ")?;
        }
        Ok(())
    }

    fn begin_scope<W>(&mut self, out: &mut W, params: &str) -> std::io::Result<()>
        where W: std::io::Write
    {
        self.write_indent(out)?;
        self.indent += 1;
        write!(out, "({params}\n")
    }

    fn end_scope<W>(&mut self, out: &mut W) -> std::io::Result<()> where W: std::io::Write {
        self.indent -= 1;
        self.write_indent(out)?;
        write!(out, ")\n")
    }

    fn write_net_stat<W, S>(&self, ctx: &mut TraceVisitCtx<W>, name: S, stat: &SignalStats)
        -> Result<(), std::io::Error>
    where
        W: std::io::Write,
        S: Into<String>
    {
        self.write_indent(ctx.out)?;
        write!(
            ctx.out,
            "({} (T0 {}) (T1 {}) (TX {}) (TZ {}) (TC {}) (IG {}))\n",
            name.into().replace('\\', "\\\\"),
            stat.low_time,
            stat.high_time,
            stat.x_time,
            stat.z_time,
            stat.clean_trans_count,
            stat.glitch_trans_count
        )
    }

}

impl<'w, W> TraceVisitorAgent<'w, W> for SaifAgent<'w> where W: std::io::Write {
    type Error = std::io::Error;

    fn enter_net(&mut self, ctx: &mut TraceVisitCtx<W>, var_ref: VarRef)
        -> Result<(), Self::Error>
    {
        let net = ctx.waveform.hierarchy().get(var_ref);

        if self.get_ctx().instance_empty {
            let scope_str = format!("INSTANCE {}", self.get_ctx().name_escaped);
            self.begin_scope(ctx.out, scope_str.as_str())?;
            self.begin_scope(ctx.out, "NET")?;
            self.get_ctx_mut().instance_empty = false;
        }

        let my_stats = &self.stats[&HashVarRef(var_ref)][self.span_index];
        match my_stats {
            PackedStats::OneBit(stat) => {
                let name = indexed_name(net.name(ctx.waveform.hierarchy()).into(), net);
                if stat.clean_trans_count > 1 {
                    println!("Glitch detected: signal name {:?}, clock index: {}", name.clone(), self.span_index);
                }
                self.write_net_stat(ctx, name, stat)?;
            }
            PackedStats::Vector(stats) => for (idx, stat) in stats.iter().enumerate() {
                let name = format!("{}[{}]", net.name(ctx.waveform.hierarchy()), idx);
                if stat.clean_trans_count > 1 {
                    println!("Glitch detected: signal name {:?}, clock index: {}", name.clone(), self.span_index);
                }
                self.write_net_stat(ctx, name, stat)?;
            }
        }

        Ok(())
    }

    fn end_nets(&mut self, ctx: &mut TraceVisitCtx<W>) -> Result<(), Self::Error> {
        if !self.get_ctx().instance_empty {
            self.end_scope(ctx.out) // End NETS scope
        } else {
            Ok(())
        }
    }

    fn enter_scope(&mut self, ctx: &mut TraceVisitCtx<W>, scope: &'w Scope)
        -> Result<(), Self::Error>
    {
        self.scope_ctx.push(ScopeCtx {
            name_escaped: scope.name(ctx.waveform.hierarchy())
                .replace('[', "\\[")
                .replace(']', "\\]"),
            instance_empty: true
        });

        // TODO: Scope export should be deferred until it's determined there's at least one
        // net down the hierarchy tha should be exported. The reason for that is to avoid exporting
        // hierarhies of scopes that contain no relevant nets.
        // This is an overcomplicated hack that deffers it only by one level of nesting.

        // Begin parent's scope if it was empty
        if let Some(ScopeCtx { ref instance_empty, name_escaped }) = self.get_parent_ctx_mut() {
            if *instance_empty {
                let scope_str = format!("INSTANCE {}", name_escaped);
                self.begin_scope(ctx.out, scope_str.as_str())?;
            }
        }

        Ok(())
    }

    fn exit_scope(&mut self, ctx: &mut TraceVisitCtx<W>, _scope: &'w Scope)
            -> Result<(), Self::Error> {
        self.end_scope(ctx.out)
    }
}

pub fn export<W>(
    ctx: &Context,
    mut out: W,
    iteration: usize
) -> std::io::Result<()>
    where W: std::io::Write
{
    let hier = ctx.wave.hierarchy();
    let time_end = *ctx.wave.time_table().last().unwrap();
    let timescale = hier.timescale().unwrap();

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
        Utc::now().format("%a %b %-d %T %Y"),
        clap::crate_name!(),
        clap::crate_version!(),
        timescale.factor, DisplayTimescaleUnit(timescale.unit),
        time_end / ctx.num_of_iterations
    )?;

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

    let mut agent = SaifAgent::new(&ctx.stats, iteration, 1);
    agent.visit_hierarchy(ctx.lookup_point, &mut visitor_ctx)?;

    write!(out, ")\n")?;

    Ok(())
}
