// Copyright (c) 2024-2026 Antmicro <www.antmicro.com>
// SPDX-License-Identifier: Apache-2.0

use wellen::{self, Hierarchy, Scope, VarRef};

// Wellen has no good iterator over ALL VarRefs, so I made one. The generics are here to deal with
// hidden iterator types.
struct VarRefIterator<'w, HIter, F>
where
    HIter: Iterator<Item = VarRef> + 'w,
    F: Fn(&'w Scope) -> HIter,
{
    hierarchy: &'w Hierarchy,
    get_iter_of_scope: F,
    scopes: Vec<&'w Scope>,
    iter: HIter,
}

impl<'w, HIter, F> Iterator for VarRefIterator<'w, HIter, F>
where
    HIter: Iterator<Item = VarRef> + 'w,
    F: Fn(&'w Scope) -> HIter,
{
    type Item = VarRef;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(v) => Some(v),
            None => {
                if let Some(scope) = self.scopes.pop() {
                    self.iter = (self.get_iter_of_scope)(scope);
                    let mut siter = scope.scopes(self.hierarchy);
                    self.scopes
                        .extend(siter.by_ref().map(|s| &self.hierarchy[s]));
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
            scopes: self.scopes().map(|s| &self[s]).collect(),
            get_iter_of_scope: |s: &Scope| s.vars(self),
            iter: self.first_scope().expect("Top scope not found").vars(self),
        }
    }
}
