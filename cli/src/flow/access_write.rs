//! The writes of one statement (plan v2.31 step 4 A2; §5.1 rule 9):
//! assignment rows read their value before writing the target,
//! updates read-write, macro and binder rows bind through the scope
//! stack, and a member, index or deref target reads its base.

use super::build::{Lowerer, RW, WRITE};
use super::rows::Mode;
use tree_sitter::Node;

/// The `{name}` pieces of a string: a run of name characters right
/// after a `{` that no second `{` escapes.
pub(super) fn braced(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    for (i, _) in text.match_indices('{') {
        if i > 0 && bytes[i - 1] == b'{' || bytes.get(i + 1) == Some(&b'{') {
            continue;
        }
        let name: String = text[i + 1..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
            .collect();
        if name.starts_with(|c: char| c.is_alphabetic() || c == '_') {
            out.push(name);
        }
    }
    out
}

impl Lowerer<'_> {
    /// An assignment row: the value read, then the target written.
    pub(super) fn assign(&mut self, node: Node<'_>, excl: &[usize]) -> bool {
        let row =
            self.flow.assigns.iter().find(|a| {
                a.kind == node.kind() && (a.op.is_empty() || super::at::holds(node, &a.op))
            });
        let Some(row) = row else { return false };
        let others = [row.left.as_str(), row.right.as_str()];
        for r in self.at(node, &row.right, &others) {
            self.walk(r, excl);
        }
        let mode = match row.mode {
            Mode::Write => WRITE,
            Mode::ReadWrite => RW,
            Mode::Outer => return true,
        };
        for l in self.at(node, &row.left, &others) {
            self.target(l, mode);
        }
        true
    }

    pub(super) fn update(&mut self, node: Node<'_>) -> bool {
        let Some((_, pos)) = self
            .flow
            .update_kinds
            .iter()
            .find(|(k, _)| k == node.kind())
        else {
            return false;
        };
        for t in self.at(node, pos, &[]) {
            self.target(t, RW);
        }
        true
    }

    /// A macro call: every name in its arguments read, and every
    /// `{name}` in a string there (rule 9).
    pub(super) fn macro_(&mut self, node: Node<'_>) -> bool {
        let Some(row) = self.flow.macros.iter().find(|m| m.kind == node.kind()) else {
            return false;
        };
        for args in self.at(node, &row.args, &[]) {
            for n in crate::scan::ast::preorder(args, |_| true, crate::scan::ast::children) {
                if self.is(&self.flow.ident_kinds, n) {
                    self.read_name(&self.text(n));
                } else if n.kind() == row.strings {
                    self.read_braced(n);
                }
            }
        }
        true
    }

    /// An expression that binds (Rust `let` in a condition, Java
    /// `instanceof T t`): the rest read, then the pattern's names
    /// declared and written here, in the branch scope when one is set.
    pub(super) fn binder(&mut self, node: Node<'_>, excl: &[usize]) -> bool {
        let Some((_, pos)) = self
            .flow
            .pattern_binders
            .iter()
            .find(|(k, _)| k == node.kind())
        else {
            return false;
        };
        let patterns = self.at(node, pos, &[]);
        let mut skip: Vec<usize> = patterns.iter().map(|n| n.id()).collect();
        skip.extend_from_slice(excl);
        for c in self.kids(node) {
            self.walk(c, &skip);
        }
        let branch = self.is(&self.flow.branch_binders, node);
        let idx = if branch { self.binder_scope } else { None }.unwrap_or(self.stmt_scope);
        let idx = if self.flow.scoping == super::rows::Scope::Function {
            0
        } else {
            idx
        };
        for p in patterns {
            self.bind(p, super::scope::Bind::local(idx, true), excl);
        }
        true
    }

    /// A write target (rule 9): a name written (declared first where a
    /// first write declares), a pattern's names each, a member / index /
    /// deref target read.
    pub(super) fn target(&mut self, node: Node<'_>, mode: i64) {
        let f = self.flow;
        if self.is(&f.ident_kinds, node) || self.is(&f.pattern_idents, node) {
            return self.name_target(node, mode);
        }
        if self.is(&f.pattern_kinds, node) {
            let skip = self.name_positions(node);
            for c in self
                .kids(node)
                .into_iter()
                .filter(|c| !skip.contains(&c.id()))
            {
                self.target(c, mode);
            }
            return;
        }
        self.walk(node, &[]);
    }

    /// A name written: no local under a local-only scope or for a
    /// discard name; a write in a conditional position a read-write; a
    /// first write declares where the language says so (rule 8).
    fn name_target(&mut self, node: Node<'_>, mode: i64) {
        let f = self.flow;
        let name = self.text(node);
        if self.local_only || f.discard_names.contains(&name) {
            return;
        }
        let mode = if self.cond && mode == WRITE { RW } else { mode };
        let v = match self.resolve(&name) {
            Some(v) => Some(v),
            None if f.first_write_declares => self.declare_in(&name, 0, super::at::place(node)),
            None => None,
        };
        if let Some(v) = v {
            self.access(v, mode);
        }
    }
}
