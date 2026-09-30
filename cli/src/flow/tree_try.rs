//! Tries and Python `with` (plan v2.31 step 4 A2; §5.1 rule 2 and
//! ruling 5): resources, body, Python's else, catches and finally as
//! a try's children in the order the core's graph reads them; a
//! `with` is a try whose one catch is empty.

use super::build::{BLOCK, CATCH, EMPTY, FINALLY, Lowerer, R, TRY, WRITE};
use super::rows::{Scope, Try, With};
use super::scope::Bind;
use tree_sitter::Node;

impl Lowerer<'_> {
    /// A try (rule 2): resources read and declared here, then the body,
    /// Python's else, the catches and the finally as children.
    pub(super) fn try_(&mut self, node: Node<'_>, row: &Try, parent: Option<usize>) -> R {
        let others = [&row.body, &row.r#else, &row.resources].map(String::as_str);
        let (n, scope) = self.open_arm(parent, TRY, node, self.at(node, &row.resources, &others));
        self.part(&self.at(node, &row.body, &others), Some(n), node)?;
        for e in self.at(node, &row.r#else, &others) {
            self.else_part(e, Some(n))?;
        }
        for kid in self.kids(node) {
            let f = self.flow;
            if let Some(catch) = f.catches.iter().find(|c| c.kind == kid.kind()) {
                let k = self.add(Some(n), CATCH, kid);
                let inner = self.push_scope();
                self.cur = Some(k);
                let others = [&catch.param, &catch.value, &catch.body].map(String::as_str);
                let params = self.at(kid, &catch.param, &others);
                let ids: Vec<usize> = params.iter().map(|p| p.id()).collect();
                self.at(kid, &catch.value, &others)
                    .into_iter()
                    .for_each(|v| self.walk(v, &ids));
                params.iter().for_each(|p| self.bind_or_write(*p));
                self.part(&self.at(kid, &catch.body, &others), Some(k), kid)?;
                self.scopes.truncate(inner);
            } else if let Some((_, body)) = f.finally_kinds.iter().find(|(k, _)| k == kid.kind()) {
                let k = self.add(Some(n), FINALLY, kid);
                self.part(&self.at(kid, body, &[]), Some(k), kid)?;
            }
        }
        self.scopes.truncate(scope);
        Ok(())
    }

    /// A node opening a scope, the current statement for its heading
    /// statements (a try's resources, a switch's init): their accesses
    /// land on it, their declarations in its scope.
    pub(super) fn open_arm(
        &mut self,
        parent: Option<usize>,
        kind: i64,
        node: Node<'_>,
        heads: Vec<Node<'_>>,
    ) -> (usize, usize) {
        let n = self.add(parent, kind, node);
        let scope = self.push_scope();
        self.cur = Some(n);
        heads.into_iter().for_each(|h| self.simple(h));
        (n, scope)
    }

    /// Python's else of a try or loop: its body, one block.
    pub(super) fn else_part(&mut self, e: Node<'_>, parent: Option<usize>) -> R {
        let body = self.flow.else_kinds.iter().find(|(k, _)| k == e.kind());
        let nodes = body.map_or(vec![e], |(_, b)| self.at(e, b, &[]));
        self.part(&nodes, parent, e)
    }

    /// Statements as one block (a try's body, a handler's, a loop's
    /// else): a lone container is that block.
    pub(super) fn part(&mut self, nodes: &[Node<'_>], parent: Option<usize>, at: Node<'_>) -> R {
        match nodes {
            [only] if self.is(&self.flow.block_kinds, *only) => self.labelled_block(*only, parent),
            _ => {
                let b = self.synth(parent, BLOCK, at, "<synthetic:block>");
                let scope = self.push_scope();
                self.stmts(nodes, Some(b))?;
                self.scopes.truncate(scope);
                if self.nodes[b].children.is_empty() {
                    self.flag(b, EMPTY);
                }
                Ok(())
            }
        }
    }

    /// A binding a handler or with writes: declared, or written where
    /// the first write declares (rule 8).
    fn bind_or_write(&mut self, node: Node<'_>) {
        if self.flow.first_write_declares {
            self.target(node, WRITE);
        } else {
            let idx = self.scope_for(Scope::Block);
            self.bind(node, Bind::local(idx, true), &[]);
        }
    }

    /// A Python `with` (ruling 5): a try whose items are read and bound
    /// on its own node, its body a block, one empty catch.
    pub(super) fn with(&mut self, node: Node<'_>, row: &With, parent: Option<usize>) -> R {
        let n = self.add(parent, TRY, node);
        self.cur = Some(n);
        for item in self.at(node, &row.item, &[row.body.as_str()]) {
            let binders = self.at(item, &row.binder, &[row.value.as_str()]);
            let ids: Vec<usize> = binders.iter().map(|b| b.id()).collect();
            self.at(item, &row.value, &[row.binder.as_str()])
                .into_iter()
                .for_each(|v| self.walk(v, &ids));
            binders.iter().for_each(|b| self.bind_or_write(*b));
        }
        self.part(
            &self.at(node, &row.body, &[row.item.as_str()]),
            Some(n),
            node,
        )?;
        let c = self.synth(Some(n), CATCH, node, "<synthetic:catch>");
        self.flag(c, EMPTY);
        Ok(())
    }

    /// A node made now, placed later (a label the lowering of what
    /// comes before it must already aim at).
    pub(super) fn detached(&mut self, kind: i64, at: Node<'_>, text: &str) -> usize {
        self.synth(Some(usize::MAX), kind, at, text)
    }
}
