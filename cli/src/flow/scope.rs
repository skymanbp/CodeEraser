//! The variable table (plan v2.31 step 4 A2; §5.1 rules 8 and 10):
//! parameters, declarations and pattern bindings, the scope stack they
//! land in and the nearest-visible resolution every read and write
//! goes through. A name a nonlocal statement names, a discard name and
//! a name no declaration reaches resolve to nothing: those accesses
//! never enter the table (rule 9).

use super::build::{ADDRESS, IGNORED, Lowerer, PARAM, WRITE};
use super::rows::{Decl, Scope};
use crate::scan::functions;
use std::collections::HashMap;
use tree_sitter::Node;

/// How a binder binds: as a parameter (`scope` None) or a local
/// declared in the scope at that stack index, written or not.
#[derive(Clone, Copy)]
pub(super) struct Bind {
    pub scope: Option<usize>,
    pub write: bool,
    pub redeclare: bool,
    pub address: bool,
}

impl Bind {
    pub const PARAM: Self = Self {
        scope: None,
        write: false,
        redeclare: false,
        address: false,
    };

    pub fn local(scope: usize, write: bool) -> Self {
        Self {
            scope: Some(scope),
            write,
            redeclare: false,
            address: false,
        }
    }
}

impl Lowerer<'_> {
    /// The names a nonlocal statement keeps out of the table, read
    /// before anything is declared: `global x` holds for the whole unit.
    pub(super) fn prescan_globals(&mut self, unit: Node<'_>) {
        if self.flow.nonlocal_kinds.is_empty() {
            return;
        }
        let nodes = crate::scan::ast::preorder(
            unit,
            |n| !functions::is_unit_node(n, self.src, self.scan),
            crate::scan::ast::children,
        );
        let stmts: Vec<Node<'_>> = nodes
            .into_iter()
            .filter(|n| self.is(&self.flow.nonlocal_kinds, *n))
            .collect();
        for stmt in stmts {
            for name in crate::scan::ast::named_children(stmt) {
                if self.is(&self.flow.ident_kinds, name) {
                    self.globals.insert(self.text(name));
                }
            }
        }
    }

    pub(super) fn new_var(&mut self, name: &str, decl: Option<usize>, at: (u32, u32)) -> usize {
        let flags = if name.starts_with('_') {
            1 << IGNORED
        } else {
            0
        };
        self.vars.push(super::build::Var {
            name: name.to_owned(),
            decl,
            flags,
            at,
        });
        self.vars.len() - 1
    }

    /// The stack index a declaration of this scope lands in.
    pub(super) fn scope_for(&self, scope: Scope) -> usize {
        if self.flow.scoping == Scope::Function || scope == Scope::Function {
            0
        } else {
            self.scopes.len() - 1
        }
    }

    /// A new local at the current statement — or, function-wide, the
    /// one already there (a second `var x`, a Python rebinding).
    pub(super) fn declare_in(&mut self, name: &str, idx: usize, at: (u32, u32)) -> Option<usize> {
        if self.flow.discard_names.iter().any(|d| d == name) || self.globals.contains(name) {
            return None;
        }
        if idx == 0
            && let Some(&v) = self.scopes[0].get(name)
        {
            return Some(v);
        }
        let v = self.new_local(name, idx, at);
        if let (0, Some(c)) = (idx, self.cur) {
            self.found.push((name.to_owned(), c, at));
        }
        Some(v)
    }

    /// The nearest visible declaration of a name.
    pub(super) fn resolve(&self, name: &str) -> Option<usize> {
        if self.globals.contains(name) {
            return None;
        }
        self.scopes.iter().rev().find_map(|s| s.get(name).copied())
    }

    pub(super) fn push_scope(&mut self) -> usize {
        self.scopes.push(HashMap::new());
        self.scopes.len() - 1
    }

    /// A node a binder may bind through.
    pub(super) fn bindable(&self, node: Node<'_>) -> bool {
        let f = self.flow;
        [
            &f.ident_kinds,
            &f.pattern_idents,
            &f.pattern_kinds,
            &f.dotted_patterns,
        ]
        .iter()
        .any(|kinds| self.is(kinds, node))
            || f.binder_paths.iter().any(|(k, _)| k == node.kind())
    }

    /// Binds every name a binder holds (rule 8): a name binds itself, a
    /// pattern its names, a declarator the name at its leaf; anything
    /// else inside is read (a default value, a pattern's constant).
    pub(super) fn bind(&mut self, node: Node<'_>, b: Bind, excl: &[usize]) {
        if excl.contains(&node.id()) {
            return;
        }
        let f = self.flow;
        if self.is(&f.ident_kinds, node) || self.is(&f.pattern_idents, node) {
            return self.bind_name(node, b);
        }
        if let Some((kind, inner)) = f.binder_paths.iter().find(|(k, _)| k == node.kind()) {
            let address = b.address || f.ref_binding_kinds.contains(kind);
            for n in self.at(node, inner, &[]) {
                self.bind(n, Bind { address, ..b }, excl);
            }
            return;
        }
        if self.is(&f.dotted_patterns, node) {
            match self.kids(node)[..] {
                [only] if self.is(&f.ident_kinds, only) => self.bind_name(only, b),
                _ => self.walk(node, excl),
            }
            return;
        }
        if self.is(&f.pattern_kinds, node) {
            return self.bind_pattern(node, b, excl);
        }
        self.walk(node, excl);
    }

    /// A pattern's parts: each bindable one bound, the rest (a default,
    /// an index) walked; a field name no binder.
    fn bind_pattern(&mut self, node: Node<'_>, b: Bind, excl: &[usize]) {
        let skip = self.name_positions(node);
        for c in self
            .kids(node)
            .into_iter()
            .filter(|c| !skip.contains(&c.id()))
        {
            if self.bindable(c) {
                self.bind(c, b, excl);
            } else {
                self.walk(c, excl);
            }
        }
    }

    fn bind_name(&mut self, node: Node<'_>, b: Bind) {
        let name = self.text(node);
        let upper = self.flow.upper_pattern_paths && name.starts_with(|c: char| c.is_uppercase());
        if upper || self.flow.discard_names.contains(&name) {
            return;
        }
        let at = super::at::place(node);
        let v = match b.scope {
            None => {
                let v = self.new_var(&name, None, at);
                self.vars[v].flags |= 1 << PARAM;
                self.scopes[0].insert(name, v);
                Some(v)
            }
            Some(_) if b.redeclare && self.scopes.last().is_some_and(|s| s.contains_key(&name)) => {
                self.scopes.last().and_then(|s| s.get(&name).copied())
            }
            Some(idx) => self.declare_in(&name, idx, at),
        };
        let Some(v) = v else { return };
        if b.write {
            self.access(v, WRITE);
        }
        if b.address {
            self.vars[v].flags |= 1 << ADDRESS;
        }
    }

    /// A declaration row applied at the current statement (rule 8):
    /// each item's initializer read, then its binder bound.
    pub(super) fn decl(&mut self, node: Node<'_>, row: &Decl) {
        let lasting = self.at(node, &row.storage, &[]).into_iter().any(|s| {
            let t = self.text(s);
            self.flow
                .lasting_storage
                .iter()
                .any(|w| *w == t || super::at::holds(s, w))
        });
        let items = match row.items.as_str() {
            "" => vec![node],
            items => self.at(node, items, &[]),
        };
        let idx = self.scope_for(row.scope);
        for item in items {
            self.decl_item(item, row, idx, lasting);
        }
    }

    /// One declared item: its initialisers read, then — unless its
    /// storage lasts past the call — its binders bound, written when
    /// initialised; a prototype declarator binds nothing (ruling 7).
    fn decl_item(&mut self, item: Node<'_>, row: &Decl, idx: usize, lasting: bool) {
        let (binders, inits) = match row.pair.is_empty() || item.kind() == row.pair {
            true => {
                let others = [
                    row.binder.as_str(),
                    row.init.as_str(),
                    row.alternative.as_str(),
                ];
                self.decl_types(item, &others);
                (
                    self.at(item, &row.binder, &others),
                    self.at(item, &row.init, &others),
                )
            }
            false => (vec![item], Vec::new()),
        };
        for i in &inits {
            self.walk(*i, &[]);
        }
        if lasting {
            return;
        }
        if binders
            .iter()
            .any(|b| self.holds_kind(*b, &self.flow.ref_binding_kinds))
        {
            inits.iter().for_each(|i| self.address_mark(*i));
        }
        let b = Bind {
            redeclare: row.redeclare,
            ..Bind::local(idx, !inits.is_empty())
        };
        for binder in binders {
            match self.prototype(binder) {
                true => self.prototype_args(binder),
                false => self.bind(binder, b, &[]),
            }
        }
    }

    /// A declarator naming a function directly (ruling 7): a prototype
    /// in a body, no local. A name wrapped in a pointer or parentheses
    /// inside it (`(*fp)(int)`) is a local.
    fn prototype(&self, binder: Node<'_>) -> bool {
        self.is(&self.flow.prototype_kinds, binder)
            && binder
                .child_by_field_name("declarator")
                .is_some_and(|d| !self.flow.binder_paths.iter().any(|(k, _)| k == d.kind()))
    }

    /// Whether a node or a descendant is of one of these kinds.
    pub(super) fn holds_kind(&self, node: Node<'_>, kinds: &[String]) -> bool {
        !kinds.is_empty()
            && crate::scan::ast::preorder(node, |_| true, crate::scan::ast::children)
                .into_iter()
                .any(|n| self.is(kinds, n))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/flow/scope.rs"]
mod tests;
