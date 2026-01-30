// Copyright (c) 2024-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

pub mod saif;
pub mod tcl;

use crate::LookupPoint;
use crate::netlist::{Module, ModuleLookupError, Netlist};
use std::io::Write;
use wellen::simple::Waveform;
use wellen::{GetItem, Scope, VarRef};

#[derive(Debug, Copy, Clone)]
enum ModuleRef<'n> {
    OutsideNetlist,
    Netlist(&'n Module),
    BlackBox,
}

/// Holds context required by TraceVisitorAgent when to traverse the hierarchy
struct TraceVisitCtx<'b, 'w, 'n, W>
where
    W: std::io::Write,
{
    // TODO: out should not be required for traversal
    out: &'b mut W,
    waveform: &'w Waveform,
    netlist_root: Vec<String>,
    top_module: &'w String,
    netlist: Option<&'n Netlist>,
    netlist_prefix: Vec<String>,
    blackboxes_only: bool,
    remove_virtual_pins: bool,
}

/// Traverses a hierarchy of scopes and variables loaded from a trace. For a given scope nets are
/// visited first, then scopes
trait TraceVisitorAgent<'w, W>
where
    W: Write,
{
    type Error;
    /// Called upon entering a new scope
    fn enter_scope(
        &mut self,
        ctx: &mut TraceVisitCtx<W>,
        scope: &'w Scope,
    ) -> Result<(), Self::Error>;
    /// Called upon leaving a scope
    fn exit_scope(
        &mut self,
        _ctx: &mut TraceVisitCtx<W>,
        _scope: &'w Scope,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    /// Called once all scopes in the current scope have been visited
    fn end_scopes(&mut self, _ctx: &mut TraceVisitCtx<W>) -> Result<(), Self::Error> {
        Ok(())
    }
    /// Called upon entering a new net
    fn enter_net(&mut self, ctx: &mut TraceVisitCtx<W>, var_ref: VarRef)
    -> Result<(), Self::Error>;
    /// Called once all nets in the current scope have been visited
    fn end_nets(&mut self, _ctx: &mut TraceVisitCtx<W>) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Implements common functionalities for `TraceVisitorAgent`s
trait TraceVisit<'b, 'w, 'n, W>
where
    W: Write,
{
    type Error;
    fn visit_scope<'p>(
        &mut self,
        ctx: &mut TraceVisitCtx<'b, 'w, 'n, W>,
        scope: &'w Scope,
        parent_module: ModuleRef<'p>,
    ) -> Result<(), Self::Error>
    where
        'n: 'p;

    /// Visint a hierarchy starting from `lookup_point`
    fn visit_hierarchy(
        &mut self,
        lookup_point: LookupPoint,
        ctx: &mut TraceVisitCtx<'b, 'w, 'n, W>,
    ) -> Result<(), Self::Error>;
}

/// Retrieve a module reference for `scope` given its parent module reference.
/// If no netlist is present, this will always be `ModuleRef::OutsideNetlist`
/// Note, this function require `ctx.netlist_prefix` to point to current scope.
/// This function will not fail when referencing scope paths which don't lie under the netlist,
/// even if they are not present in hierarchy.
fn get_child_module_reference<'p, 'b, 'w, 'n, W>(
    ctx: &TraceVisitCtx<'b, 'w, 'n, W>,
    scope: &'w Scope,
    parent_module: ModuleRef<'p>,
) -> ModuleRef<'n>
where
    'n: 'p,
    W: Write,
{
    ctx.netlist
        .map(|netlist| {
            let name = scope.name(ctx.waveform.hierarchy());
            match (parent_module, &ctx.netlist_prefix) {
                (ModuleRef::OutsideNetlist, scope @ _) => {
                    if scope == &ctx.netlist_root {
                        match netlist.modules.get(ctx.top_module) {
                            Some(module) => ModuleRef::Netlist(module),
                            None => panic!("Top module `{}` was not found", ctx.top_module),
                        }
                    } else {
                        ModuleRef::OutsideNetlist
                    }
                }
                (ModuleRef::Netlist(parent), _) => match parent.get_module_of_cell(netlist, name) {
                    Ok(module) => ModuleRef::Netlist(module),
                    Err(ModuleLookupError::ModuleUndefined) => ModuleRef::BlackBox,
                    Err(ModuleLookupError::CellNotFound) => panic!("cell {} not found", name),
                },
                (ModuleRef::BlackBox, _) => panic!("Error: attempted to descnd into a blackbox"),
            }
        })
        .unwrap_or(ModuleRef::OutsideNetlist)
}

impl<'b, 'w, 'n, W, A> TraceVisit<'b, 'w, 'n, W> for A
where
    W: Write,
    A: TraceVisitorAgent<'w, W>,
{
    type Error = A::Error;

    fn visit_scope<'p>(
        &mut self,
        ctx: &mut TraceVisitCtx<'b, 'w, 'n, W>,
        scope: &'w Scope,
        parent_module: ModuleRef<'p>,
    ) -> Result<(), A::Error>
    where
        'n: 'p,
    {
        self.enter_scope(ctx, scope)?;

        let hier = ctx.waveform.hierarchy();

        let name = scope.name(hier);

        if let ModuleRef::OutsideNetlist = parent_module {
            ctx.netlist_prefix.push(name.to_string());
        }
        let module = get_child_module_reference(ctx, scope, parent_module);

        let export_nets = match (ctx.blackboxes_only, module) {
            (false, _) => true,
            (true, ModuleRef::BlackBox) => true,
            _ => false,
        };
        if export_nets {
            for var_ref in scope.vars(hier) {
                if let ModuleRef::BlackBox = module {
                    if ctx.remove_virtual_pins {
                        let var = hier.get(var_ref);
                        if let "VGND" | "VNB" | "VPB" | "VPWR" = var.name(hier) {
                            continue;
                        }
                    }
                }
                self.enter_net(ctx, var_ref)?;
            }
        }
        self.end_nets(ctx)?;
        if let ModuleRef::BlackBox = module { /* Do not descend blackboxes */
        } else {
            for scope_ref in scope.scopes(hier) {
                let scope = hier.get(scope_ref);
                self.visit_scope(ctx, scope, module)?;
            }
        }
        self.end_scopes(ctx)?;

        self.exit_scope(ctx, scope)
    }

    fn visit_hierarchy(
        &mut self,
        lookup_point: LookupPoint,
        ctx: &mut TraceVisitCtx<'b, 'w, 'n, W>,
    ) -> Result<(), Self::Error> {
        match lookup_point {
            LookupPoint::Top => {
                for scope_ref in ctx.waveform.hierarchy().scopes() {
                    let scope = ctx.waveform.hierarchy().get(scope_ref);
                    self.visit_scope(ctx, scope, ModuleRef::OutsideNetlist)?;
                }
            }
            LookupPoint::Scope(scope_ref) => {
                // Set up module_ref and ctx.netlist_prefix
                // TODO: Simplify this logic

                let hier = ctx.waveform.hierarchy();
                let scope = hier.get(scope_ref);
                let mut full_path: Vec<_> = scope
                    .full_name(hier)
                    .split('.')
                    .map(ToString::to_string)
                    .collect();
                let mut module_ref = ModuleRef::OutsideNetlist;
                let mut child_scope = ctx
                    .waveform
                    .hierarchy()
                    .scopes()
                    .map(|scope_ref| hier.get(scope_ref))
                    .find(|scope| scope.name(hier) == full_path[0])
                    .expect("Child scope should be valid");

                ctx.netlist_prefix.push(full_path[0].to_string());

                full_path.pop(); // We need to point to parent module

                if full_path.len() >= 1 {
                    for scope_name in &full_path[1..] {
                        child_scope = child_scope
                            .scopes(hier)
                            .map(|scope_ref| hier.get(scope_ref))
                            .find(|scope| scope.name(hier) == scope_name)
                            .expect("Child scope should be valid");
                        module_ref = get_child_module_reference(ctx, child_scope, module_ref);
                        if let ModuleRef::OutsideNetlist = module_ref {
                            ctx.netlist_prefix.push(scope_name.to_string());
                        }
                    }
                }

                self.visit_scope(ctx, scope, module_ref)?;
            }
        }
        Ok(())
    }
}
