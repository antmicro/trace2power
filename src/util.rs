// Copyright (c) 2024-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use wellen::{self, GetItem, Hierarchy, Scope, ScopeRef, VarRef};

// Wellen has no good iterator over ALL VarRefs, so I made one. The generics are here to deal with
// hidden iterator types.
struct VarRefIterator<'w, HIter, SIter, F, S>
where
    HIter: Iterator<Item = VarRef> + 'w,
    SIter: Iterator<Item = ScopeRef> + 'w,
    F: Fn(&'w Scope) -> HIter,
    S: Fn(&'w Scope) -> SIter,
{
    hierarchy: &'w Hierarchy,
    get_iter_of_scope: F,
    get_siter_of_scope: S,
    scopes: Vec<&'w Scope>,
    iter: HIter,
    siter: SIter,
}

impl<'w, HIter, SIter, F, S> Iterator for VarRefIterator<'w, HIter, SIter, F, S>
where
    HIter: Iterator<Item = VarRef> + 'w,
    SIter: Iterator<Item = ScopeRef> + 'w,
    F: Fn(&'w Scope) -> HIter,
    S: Fn(&'w Scope) -> SIter,
{
    type Item = VarRef;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(v) => Some(v),
            None => {
                if let Some(scope) = self.scopes.pop() {
                    self.iter = (self.get_iter_of_scope)(scope);
                    self.siter = (self.get_siter_of_scope)(scope);
                    self.scopes
                        .extend(self.siter.by_ref().map(|s| self.hierarchy.get(s)));
                    self.next()
                } else {
                    None
                }
            }
        }
    }
}

pub trait VarRefsIter {
    fn var_refs_iter<'s>(&'s self) -> impl Iterator<Item = VarRef> + 's;
}

impl VarRefsIter for Hierarchy {
    fn var_refs_iter<'s>(&'s self) -> impl Iterator<Item = VarRef> + 's {
        VarRefIterator {
            hierarchy: &self,
            scopes: self.scopes().map(|s| self.get(s)).collect(),
            get_iter_of_scope: |s: &Scope| s.vars(self),
            get_siter_of_scope: |s: &Scope| s.scopes(self),
            iter: self.vars(),
            siter: self.scopes(),
        }
    }
}
