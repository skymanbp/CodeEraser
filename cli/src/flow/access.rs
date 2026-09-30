//! The accesses of one statement, in evaluation order (plan v2.31 step
//! 4 A2; §5.1 rules 7, 9 and 10). The walker reads an expression in
//! source order, an assignment right before left; a write in a
//! conditional position is a read-write; a member, index or deref
//! target reads its base; a nested unit or scope only marks the host
//! names it mentions captured (capture.rs). Every choice here adds reads or drops
//! writes when in doubt — the side on which a finding is lost, never
//! invented.

use super::access_write::braced;
use super::build::{ADDRESS, Lowerer, READ, WRITE};
use crate::scan::functions;
use tree_sitter::Node;

impl Lowerer<'_> {
    /// One access at the current statement.
    pub(super) fn access(&mut self, v: usize, mode: i64) {
        if let Some(c) = self.cur {
            self.nodes[c].uses.push((v, mode));
        }
    }

    pub(super) fn read_name(&mut self, name: &str) {
        if let Some(v) = self.resolve(name) {
            self.access(v, READ);
        }
    }

    /// The children of a node that are no variable references (rule 9).
    pub(super) fn name_positions(&self, node: Node<'_>) -> Vec<usize> {
        let rows = self
            .flow
            .name_positions
            .iter()
            .filter(|(k, p)| k == node.kind() && p != ".");
        rows.flat_map(|(_, p)| self.at(node, p, &[]))
            .map(|n| n.id())
            .collect()
    }

    fn whole_name_position(&self, node: Node<'_>) -> bool {
        self.flow
            .name_positions
            .iter()
            .any(|(k, p)| k == node.kind() && p == ".")
    }

    /// The callee a call spells (LangSpec::call_fields, a FlowSpec call
    /// form, a macro's name and its `!`), or None when the node is no
    /// call.
    pub(super) fn callee(&self, node: Node<'_>) -> Option<String> {
        if self.scan.call_kinds.contains(&node.kind()) {
            let (field, object) = self.scan.call_fields;
            let name = self.text(node.child_by_field_name(field)?);
            let object = object.and_then(|o| node.child_by_field_name(o));
            return Some(object.map_or(name.clone(), |o| format!("{}.{name}", self.text(o))));
        }
        if let Some((_, field)) = self.flow.call_forms.iter().find(|(k, _)| k == node.kind()) {
            return Some(self.text(node.child_by_field_name(field)?));
        }
        let row = self.flow.macros.iter().find(|m| m.kind == node.kind())?;
        let name = self
            .at(node, &row.name, &[])
            .first()
            .map(|n| self.text(*n))?;
        Some(format!("{name}!"))
    }

    fn dynamic_name(&self, name: &str) -> bool {
        self.flow
            .dynamic_names
            .iter()
            .any(|d| match d.strip_suffix('*') {
                Some(prefix) => name.starts_with(prefix),
                None => d == name,
            })
    }

    /// The accesses an expression makes, at the current statement.
    pub(super) fn walk(&mut self, node: Node<'_>, excl: &[usize]) {
        if excl.contains(&node.id()) || self.scan.comment_kinds.contains(&node.kind()) {
            return;
        }
        if self.leaf(node) || self.writes(node, excl) {
            return;
        }
        self.compound(node, excl);
    }

    /// The dynamic marks, a nested unit's capture and a name's read:
    /// true when nothing under the node is left to walk.
    fn leaf(&mut self, node: Node<'_>) -> bool {
        let f = self.flow;
        if node.is_error() || node.is_missing() || self.is(&f.dynamic_kinds, node) {
            self.dynamic_here();
        }
        if functions::is_unit_node(node, self.src, self.scan) || self.is(&f.capture_kinds, node) {
            self.capture(node);
            return true;
        }
        if self.whole_name_position(node) {
            self.type_reads(node);
            return true;
        }
        if let Some(name) = self.callee(node) {
            if self.dynamic_name(&name) {
                self.dynamic_here();
            }
            self.dispatch(&name);
        }
        if self.is(&f.ident_kinds, node) || self.is(&f.shorthand_kinds, node) {
            self.read_name(&self.text(node));
            return true;
        }
        if self.is(&f.interpolated_strings, node) {
            self.read_braced(node);
        }
        false
    }

    /// A row that writes (access_write.rs): true when one matched.
    fn writes(&mut self, node: Node<'_>, excl: &[usize]) -> bool {
        self.assign(node, excl) || self.update(node) || self.macro_(node) || self.binder(node, excl)
    }

    /// Any other node: its children, a local-only scope's under the flag,
    /// an address-of mark first; an expression loop re-reads its reads.
    fn compound(&mut self, node: Node<'_>, excl: &[usize]) {
        let f = self.flow;
        let k = node.kind();
        if self.is(&f.local_only_scopes, node) {
            let was = std::mem::replace(&mut self.local_only, true);
            self.walk_kids(node, excl);
            self.local_only = was;
            return;
        }
        if f.address_ops
            .iter()
            .any(|(kind, mark)| kind == k && self.marked(node, mark))
        {
            self.address_mark(node);
        }
        let start = self.cur.map_or(0, |c| self.nodes[c].uses.len());
        self.walk_kids(node, excl);
        if f.loops.iter().any(|l| l.kind == k) {
            self.reread_from(start);
        }
    }

    /// The children, a conditional position's under the conditional flag.
    fn walk_kids(&mut self, node: Node<'_>, excl: &[usize]) {
        let skip = self.name_positions(node);
        let cond: Vec<usize> = self
            .flow
            .conditional_ctx
            .iter()
            .filter(|(k, _, ops)| {
                k == node.kind()
                    && (ops.is_empty() || ops.iter().any(|o| super::at::holds(node, o)))
            })
            .flat_map(|(_, p, _)| self.at(node, p, &[]))
            .map(|n| n.id())
            .collect();
        for c in self
            .kids(node)
            .into_iter()
            .filter(|c| !skip.contains(&c.id()))
        {
            let was = self.cond;
            self.cond |= cond.contains(&c.id());
            self.walk(c, excl);
            self.cond = was;
        }
    }

    /// A loop in expression position runs its body again: every name it
    /// reads is read once more after it, so a write the next round reads
    /// is not taken for dead (rule 9, the back edge a statement hides).
    fn reread_from(&mut self, start: usize) {
        let Some(c) = self.cur else { return };
        let mut again: Vec<usize> = self.nodes[c].uses[start..]
            .iter()
            .filter(|(_, m)| *m != WRITE)
            .map(|(v, _)| *v)
            .collect();
        again.sort_unstable();
        again.dedup();
        again.into_iter().for_each(|v| self.access(v, READ));
    }

    pub(super) fn marked(&self, node: Node<'_>, mark: &str) -> bool {
        match mark.strip_prefix('@') {
            Some(kind) => self.kids(node).iter().any(|c| c.kind() == kind),
            None => super::at::holds(node, mark),
        }
    }

    /// Every name under a node is address-taken (rule 10).
    pub(super) fn address_mark(&mut self, node: Node<'_>) {
        for n in crate::scan::ast::preorder(node, |_| true, crate::scan::ast::children) {
            if self.is(&self.flow.ident_kinds, n)
                && let Some(v) = self.resolve(&self.text(n))
            {
                self.vars[v].flags |= 1 << ADDRESS;
            }
        }
    }

    fn read_braced(&mut self, node: Node<'_>) {
        for name in braced(&self.text(node)) {
            self.read_name(&name);
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/flow/access.rs"]
mod tests;
