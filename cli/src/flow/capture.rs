//! Captures (plan v2.31 step 4 A2 and E; design booklet
//! docs/reference/analysis-track.md §5.1 rule 10): a nested unit or
//! scope marks every host name it mentions captured. Where the table
//! says a nested scope may name a binding declared after it (TS-1:
//! TypeScript's temporal dead zone binds at call time, so a closure
//! reads a `const` declared later, or the one its own initializer
//! declares), a name that resolves to nothing yet leaves a waiting mark
//! in every scope open at the capture; a declaration landing in a
//! scope that holds its mark is captured. The mark lives in the scope's
//! own map, keyed so no name can spell it, and so dies with the scope.

use super::access_write::braced;
use super::build::{CAPTURED, Lowerer};
use super::format::format_names;
use tree_sitter::Node;

/// The key a waiting mark takes in a scope map: no source name holds
/// a NUL.
fn waiting(name: &str) -> String {
    format!("\0{name}")
}

impl Lowerer<'_> {
    /// A nested unit or scope: every host name it mentions — as a name,
    /// a shorthand or a placeholder in a string — is captured (rule 10).
    pub(super) fn capture(&mut self, node: Node<'_>) {
        let own = self.own_params(node);
        for n in crate::scan::ast::preorder(node, |_| true, crate::scan::ast::children) {
            for name in self.mentions(n).into_iter().filter(|n| !own.contains(n)) {
                match self.resolve(&name) {
                    Some(v) => self.vars[v].flags |= 1 << CAPTURED,
                    None if self.flow.forward_captures => self.wait_for(&name),
                    None => {}
                }
            }
        }
    }

    /// The host names one node of a nested scope mentions.
    fn mentions(&self, n: Node<'_>) -> Vec<String> {
        let f = self.flow;
        let named = [&f.ident_kinds, &f.shorthand_kinds, &f.pattern_idents];
        if named.iter().any(|kinds| self.is(kinds, n)) {
            return vec![self.text(n)];
        }
        if f.macros.iter().any(|m| self.is(&m.strings, n)) {
            return format_names(&self.text(n));
        }
        if self.is(&f.interpolated_strings, n) {
            return braced(&self.text(n));
        }
        Vec::new()
    }

    fn wait_for(&mut self, name: &str) {
        for scope in &mut self.scopes {
            scope.insert(waiting(name), usize::MAX);
        }
    }

    /// A new local in the scope at this stack index — captured when a
    /// nested scope named it before it was declared.
    pub(super) fn new_local(&mut self, name: &str, idx: usize, at: (u32, u32)) -> usize {
        let v = self.new_var(name, self.cur, at);
        self.scopes[idx].insert(name.to_owned(), v);
        if self.scopes[idx].contains_key(&waiting(name)) {
            self.vars[v].flags |= 1 << CAPTURED;
        }
        v
    }
}
