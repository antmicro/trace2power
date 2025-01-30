use chrono::Utc;
use std::collections::HashMap;
use indoc::indoc;
use wellen::{GetItem, TimescaleUnit, Scope, Var};
use wellen::simple::Waveform;
use rayon::prelude::*;
use crate::{indexed_name, Context, LookupPoint};
use crate::stats::{calc_stats, SignalStats};
use crate::netlist::{Netlist, Module, ModuleLookupError};

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

struct ExporterState<'b, 'w, 'n, 's, W>
where
    W: std::io::Write
{
    out: &'b mut W,
    waveform: &'w Waveform,
    netlist_root: Vec<String>,
    stats: &'s HashMap<String, Vec<SignalStats>>,
    top_module: String,
    indent: usize,
    netlist: Option<&'n Netlist>,
    netlist_prefix: Vec<String>,
    blackboxes_only: bool
}

#[derive(Copy, Clone)]
enum ModuleRef<'n> {
    OutsideNetlist,
    Netlist(&'n Module),
    BlackBox,
}

impl<'b, 'w, 'n, 's, W> ExporterState<'b, 'w, 'n, 's, W>
where
    W: std::io::Write
{
    fn write_indent(&mut self) -> std::io::Result<()> where W: std::io::Write {
        for _ in 0..self.indent {
            write!(self.out, "  ")?;
        }
        Ok(())
    }

    fn visit_net(
        &mut self,
        port: &'w Var,
    ) -> std::io::Result<()> {
        let my_stats = &self.stats[&indexed_name(port.full_name(self.waveform.hierarchy()), port)];
        for (idx, stat) in my_stats.iter().enumerate() {
            let name = if my_stats.len() > 1 {
                format!("{}[{}]", port.name(self.waveform.hierarchy()), idx)
            } else {
                indexed_name(port.name(self.waveform.hierarchy()).into(), port)
            }.replace('\\', "\\\\");
            self.write_indent()?;
            write!(
                self.out,
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

    fn get_child_module_reference<'p>(
        &self,
        scope: &'w Scope,
        parent_module: ModuleRef<'p>
    ) -> ModuleRef<'n>
        where 'n: 'p
    {
        self.netlist.map(|netlist| {
            let name = scope.name(self.waveform.hierarchy());
            match (parent_module, &self.netlist_prefix) {
                (ModuleRef::OutsideNetlist, scope @ _) => if scope == &self.netlist_root {
                    match netlist.modules.get(&self.top_module) {
                        Some(module) => ModuleRef::Netlist(module),
                        None => panic!("Top module `{}` was not found", self.top_module),
                    }
                } else {
                    ModuleRef::OutsideNetlist
                },
                (ModuleRef::Netlist(parent), _) => match parent.get_module_of_cell(netlist, name) {
                    Ok(module) => ModuleRef::Netlist(module),
                    Err(ModuleLookupError::ModuleUndefined) => ModuleRef::BlackBox,
                    Err(ModuleLookupError::CellNotFound) => panic!("cell {} not found", name),
                },
                (ModuleRef::BlackBox, _) => panic!("Error: attempted to descnd into a blackbox")
            }
        }).unwrap_or(ModuleRef::OutsideNetlist)
    }

    fn visit_scope<'p> (
        &mut self,
        scope: &'w Scope,
        parent_module: ModuleRef<'p>
    ) -> std::io::Result<()>
        where 'n: 'p
    {
        let name = scope.name(self.waveform.hierarchy());

        if let ModuleRef::OutsideNetlist = parent_module {
            self.netlist_prefix.push(name.to_string());
        }

        let module = self.get_child_module_reference(scope, parent_module);

        let name_escaped = name
            .replace('[', "\\[")
            .replace(']', "\\]");

        let mut instance_empty = true;

        let export_nets = match (self.blackboxes_only, module) {
            (false, _) => true,
            (true, ModuleRef::BlackBox) => true,
            _ => false
        };
        if export_nets {
            for var_ref in scope.vars(self.waveform.hierarchy()) {
                if instance_empty {
                    self.begin_scope(format!("INSTANCE {name_escaped}").as_str())?;
                    self.begin_scope("NET")?;
                    instance_empty = false;
                }
                let var = self.waveform.hierarchy().get(var_ref);
                self.visit_net(var)?;
            }
        }
        if !instance_empty {
            self.end_scope()?; // NET
        }
        if let ModuleRef::BlackBox = module { /* Do not descend blackboxes */ } else {
            for scope_ref in scope.scopes(self.waveform.hierarchy()) {
                if instance_empty {
                    self.begin_scope(format!("INSTANCE {name}").as_str())?;
                    instance_empty = false;
                }
                let scope = self.waveform.hierarchy().get(scope_ref);
                self.visit_scope(scope, module)?;
            }
        }
        if instance_empty {
            return Ok(()); // Write only
        }
        self.end_scope() // INSTANCE
    }

    fn begin_scope(&mut self, params: &str) -> std::io::Result<()> {
        self.write_indent()?;
        self.indent += 1;
        write!(self.out, "({params}\n")
    }

    fn end_scope(&mut self) -> std::io::Result<()> {
        self.indent -= 1;
        self.write_indent()?;
        write!(self.out, ")\n")
    }
}

pub fn export<W>(
    ctx: Context,
    mut out: W
) -> std::io::Result<()>
    where W: std::io::Write
{
    let time_end = *ctx.wave.time_table().last().unwrap();
    let timescale = ctx.wave.hierarchy().timescale().unwrap();

    let var_stats: HashMap<String, Vec<SignalStats>> = ctx.all_sig_refs.par_iter()
        .map(|sig_ref| ctx.wave.get_signal(*sig_ref).unwrap())
        .zip(ctx.all_names)
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

    let net_cursor_root = match ctx.top_scope {
        Some(scope_ref) => {
            let scope = ctx.wave.hierarchy().get(scope_ref);
            scope.full_name(ctx.wave.hierarchy())
                .split('.')
                .map(String::from)
                .collect::<Vec<_>>()
        },
        None => Vec::new()
    };

    let mut exporter_state = ExporterState {
        out: &mut out,
        waveform: &ctx.wave,
        netlist_root: net_cursor_root,
        stats: &var_stats,
        top_module: ctx.top,
        indent: 1,
        netlist: ctx.netlist.as_ref(),
        netlist_prefix: Vec::new(),
        blackboxes_only: ctx.blackboxes_only
    };

    match ctx.lookup_point {
        LookupPoint::Top => for scope_ref in ctx.wave.hierarchy().scopes() {
            let scope = ctx.wave.hierarchy().get(scope_ref);
            exporter_state.visit_scope(scope, ModuleRef::OutsideNetlist)?;
        },
        LookupPoint::Scope(scope_ref) => {
            let scope = ctx.wave.hierarchy().get(scope_ref);
            exporter_state.visit_scope(scope, ModuleRef::OutsideNetlist)?;
        }
    }

    write!(out, ")\n")?;

    Ok(())
}
