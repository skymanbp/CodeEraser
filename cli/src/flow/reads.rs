//! The reads a grammar hides (plan v2.31 step 4 E; design booklet
//! docs/reference/analysis-track.md §5.1 rules 8 and 9): a value a type
//! is taken from (TS-2), the fields a unit's entry initialises (TS-3,
//! CPP-1), the call that hands every parameter on (R-1) and the
//! arguments of a declaration the grammar reads as a prototype
//! (CPP-2). Each only adds reads: a finding lost, never one invented.

use super::build::{Lowerer, PARAM, READ, STMT};
use crate::scan::ast::{ancestors, children, preorder};
use tree_sitter::Node;

impl Lowerer<'_> {
    /// The FlowSpec::type_reads nodes inside a name position, walked
    /// (rule 9, TS-2): `x` in `let y: typeof x` is read.
    pub(super) fn type_reads(&mut self, node: Node<'_>) {
        let kinds = &self.flow.type_reads;
        let found: Vec<Node<'_>> = preorder(node, |_| true, children)
            .into_iter()
            .filter(|n| self.is(kinds, *n))
            .collect();
        let outer = |n: &Node<'_>| !ancestors(*n).any(|a| found.iter().any(|f| f.id() == a.id()));
        let outermost: Vec<Node<'_>> = found.iter().copied().filter(outer).collect();
        for n in outermost {
            self.walk(n, &[]);
        }
    }

    /// A declared item's other parts — its type (TS-2: `let t: typeof
    /// x`) — read for their FlowSpec::type_reads nodes, before its
    /// initializer as the source writes them.
    pub(super) fn decl_types(&mut self, item: Node<'_>, others: &[&str]) {
        if self.flow.type_reads.is_empty() {
            return;
        }
        for part in self.at(item, "*", others) {
            self.type_reads(part);
        }
    }

    /// The unit's synthetic entry statement reading what its entry
    /// initialises (rule 9): a parameter that declares a field (TS-3)
    /// and a FlowSpec::head_reads child (CPP-1) — only when one is
    /// written.
    pub(super) fn entry_reads(&mut self, unit: Node<'_>) {
        let mut reads = Vec::new();
        for entry in self.param_entries(unit) {
            let field = self.flow.field_params.iter();
            if field
                .filter(|(k, _)| k == entry.kind())
                .any(|(_, mark)| self.marked(entry, mark))
            {
                reads.extend(self.entry_binders(entry));
            }
        }
        reads.extend(
            self.kids(unit)
                .into_iter()
                .filter(|c| self.is(&self.flow.head_reads, *c)),
        );
        let Some(first) = reads.first().copied() else {
            return;
        };
        let n = self.synth(None, STMT, first, "<synthetic:entry>");
        self.cur = Some(n);
        for r in reads {
            self.walk(r, &[]);
        }
    }

    /// A dispatch call (rule 9, R-1): every parameter read here.
    pub(super) fn dispatch(&mut self, callee: &str) {
        if !self.flow.dispatch_calls.iter().any(|d| d == callee) {
            return;
        }
        for v in 0..self.vars.len() {
            if self.vars[v].flags >> PARAM & 1 == 1 {
                self.access(v, READ);
            }
        }
    }

    /// A declarator read as a prototype (ruling 7) whose parameter
    /// types name variables (CPP-2): each such name read.
    pub(super) fn prototype_args(&mut self, binder: Node<'_>) {
        let Some(list) = binder.child_by_field_name("parameters") else {
            return;
        };
        for n in preorder(list, |_| true, children) {
            if self.is(&self.flow.prototype_reads, n) {
                self.read_name(&self.text(n));
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/flow/reads.rs"]
mod tests;
