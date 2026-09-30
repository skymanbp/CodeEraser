//! The parameter rows (plan v2.31 step 4 A2; §5.1 rules 8 and 10):
//! a unit's parameters and named results, the synthetic statement
//! reading its default arguments, and the names a nested unit binds for
//! itself, which its capture of host names leaves out.

use super::build::{IGNORED, Lowerer};
use super::scope::Bind;
use crate::scan::functions;
use std::collections::HashSet;
use tree_sitter::Node;

impl Lowerer<'_> {
    pub(super) fn param_entries<'t>(&self, unit: Node<'t>) -> Vec<Node<'t>> {
        match functions::param_list(unit, self.scan.param_list_kinds) {
            Some(list) => crate::scan::ast::entries(list),
            None => unit.child_by_field_name("parameter").into_iter().collect(),
        }
    }

    /// The binder nodes of one parameter entry, as `param_entry` binds them.
    pub(super) fn entry_binders<'t>(&self, entry: Node<'t>) -> Vec<Node<'t>> {
        match self.flow.params.iter().find(|(k, _)| k == entry.kind()) {
            Some((_, pos)) => self.at(entry, pos, &[]),
            None if self.bindable(entry) => vec![entry],
            None => Vec::new(),
        }
    }

    /// The names a nested unit's own parameters bind: no host name
    /// (rule 10), so its capture passes them by.
    pub(super) fn own_params(&self, unit: Node<'_>) -> HashSet<String> {
        let f = self.flow;
        let mut out = HashSet::new();
        for entry in self.param_entries(unit) {
            for b in self.entry_binders(entry) {
                let nodes = crate::scan::ast::preorder(b, |_| true, crate::scan::ast::children);
                for n in nodes {
                    if self.is(&f.ident_kinds, n) || self.is(&f.pattern_idents, n) {
                        out.insert(self.text(n));
                    }
                }
            }
        }
        out
    }

    /// The parameters (rule 8), a first one named as a receiver ignored,
    /// then the named results (ruling 6), each a parameter too.
    pub(super) fn params(&mut self, unit: Node<'_>) {
        for (i, entry) in self.param_entries(unit).into_iter().enumerate() {
            let before = self.vars.len();
            self.param_entry(entry);
            let receiver = i == 0 && self.vars.len() == before + 1;
            if receiver && self.flow.receiver_names.contains(&self.vars[before].name) {
                self.vars[before].flags |= 1 << IGNORED;
            }
        }
        for result in self.at(unit, &self.flow.results, &[]) {
            let before = self.vars.len();
            for entry in crate::scan::ast::entries(result) {
                self.param_entry(entry);
            }
            self.results.extend(before..self.vars.len());
        }
    }

    fn param_entry(&mut self, entry: Node<'_>) {
        for b in self.entry_binders(entry) {
            self.bind(b, Bind::PARAM, &[]);
        }
    }

    /// The unit's first statement, synthetic, reading the default
    /// arguments — only when one is written (rule 9).
    pub(super) fn defaults(&mut self, unit: Node<'_>) {
        let Some(list) = functions::param_list(unit, self.scan.param_list_kinds) else {
            return;
        };
        let values: Vec<Node<'_>> = crate::scan::ast::entries(list)
            .into_iter()
            .flat_map(|e| {
                let rows = self
                    .flow
                    .default_arg_fields
                    .iter()
                    .filter(|(k, _)| k == e.kind());
                rows.flat_map(|(_, pos)| self.at(e, pos, &[]))
                    .collect::<Vec<_>>()
            })
            .collect();
        if values.is_empty() {
            return;
        }
        let n = self.synth(None, super::build::STMT, list, "<synthetic:defaults>");
        self.cur = Some(n);
        for v in values {
            self.walk(v, &[]);
        }
    }
}
