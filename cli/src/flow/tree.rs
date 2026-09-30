//! The statement tree (plan v2.31 step 4 A2; §5.1 rules 1, 2, 3 and 7):
//! every statement of a container lowered by the first row its kind
//! meets — a container, a structure (here: if; tree_arms.rs: loops,
//! switches, tries), a jump (tree_jumps.rs), a declaration or a plain
//! statement. Control flow in expression position is not lowered: an
//! exit hidden there only adds paths, which hides findings and never
//! invents one (ruling 4).

use super::build::{BLOCK, DYNAMIC, ELSE, EMPTY, IF, Lowerer, NORETURN, R, RETURN, STMT};
use super::rows::Decl;
use tree_sitter::Node;

impl Lowerer<'_> {
    pub(super) fn stmts(&mut self, nodes: &[Node<'_>], parent: Option<usize>) -> R {
        nodes.iter().try_for_each(|n| self.stmt(*n, parent))
    }

    /// One statement, appended to its parent; `stmt_scope` names its
    /// scope while it lowers and is restored after, so a head walked
    /// after nested statements (a do-while's condition, a later case's
    /// guard) never reads a popped scope.
    pub(super) fn stmt(&mut self, node: Node<'_>, parent: Option<usize>) -> R {
        let was = std::mem::replace(&mut self.stmt_scope, self.scopes.len() - 1);
        let done = self.stmt_inner(node, parent);
        self.stmt_scope = was;
        done
    }

    fn stmt_inner(&mut self, node: Node<'_>, parent: Option<usize>) -> R {
        let f = self.flow;
        let k = node.kind();
        let skip = [&f.empty_kinds, &f.fallthrough_kinds]
            .iter()
            .any(|ks| self.is(ks, node));
        if skip || self.scan.comment_kinds.contains(&k) {
            return Ok(());
        }
        if node.is_error() || node.is_missing() || self.is(&f.dynamic_kinds, node) {
            let n = self.plain(node, parent);
            self.flag(n, DYNAMIC);
            return Ok(());
        }
        if self.is(&f.wrapper_kinds, node) {
            return match self.kids(node)[..] {
                [only] => self.stmt(only, parent),
                _ => {
                    self.plain(node, parent);
                    Ok(())
                }
            };
        }
        self.structured(node, parent)
    }

    /// A container, branch, loop, arm or jump; else a leaf statement.
    fn structured(&mut self, node: Node<'_>, parent: Option<usize>) -> R {
        let (f, k) = (self.flow, node.kind());
        if self.is(&f.splice_kinds, node) {
            return self.stmts(&self.kids(node), parent);
        }
        if self.is(&f.block_kinds, node) {
            return self.labelled_block(node, parent);
        }
        if self.scan.if_kinds.contains(&k) {
            return self.if_(node, parent);
        }
        if f.loops.iter().any(|l| l.kind == k) {
            return self.loop_(node, parent);
        }
        if self.arm_like(node, parent)? || self.jump(node, parent)? {
            return Ok(());
        }
        self.leaf_stmt(node, parent)
    }

    /// A declaration, a nonlocal statement, a headed block or a plain
    /// statement.
    fn leaf_stmt(&mut self, node: Node<'_>, parent: Option<usize>) -> R {
        if let Some(row) = self.decl_row(node) {
            return self.declaration(node, row, parent);
        }
        if self.is(&self.flow.nonlocal_kinds, node) {
            self.add(parent, STMT, node);
            return Ok(());
        }
        if !self.headed_block(node, parent)? {
            self.plain(node, parent);
        }
        Ok(())
    }

    /// A statement with no structure: kind 1, or 15 / 9 when its
    /// top-level call names a noreturn or a returning callee (rule 3).
    pub(super) fn plain(&mut self, node: Node<'_>, parent: Option<usize>) -> usize {
        let callee = self.callee(node);
        let kind = match callee {
            Some(c) if self.flow.noreturn.contains(&c) || self.doomed.contains(&c) => NORETURN,
            Some(c) if self.flow.return_calls.contains(&c) => RETURN,
            _ => STMT,
        };
        let n = self.add(parent, kind, node);
        self.cur = Some(n);
        self.walk(node, &[]);
        n
    }

    /// A container, kind 0, its statements in a scope of their own —
    /// a labelled one (Rust `'a: { }`) a break target (rule 4).
    pub(super) fn labelled_block(&mut self, node: Node<'_>, parent: Option<usize>) -> R {
        let label = self.self_label(node);
        if label.is_none() {
            return self.block(node, parent).map(|_| ());
        }
        let n = self.add(parent, BLOCK, node);
        self.labelled_frame(n, label.into_iter().collect());
        self.fill_block(node, n)?;
        self.close_labelled(n, parent);
        Ok(())
    }

    pub(super) fn block(&mut self, node: Node<'_>, parent: Option<usize>) -> Result<usize, String> {
        let n = self.add(parent, BLOCK, node);
        self.fill_block(node, n)?;
        Ok(n)
    }

    fn fill_block(&mut self, node: Node<'_>, n: usize) -> R {
        self.push_scope();
        self.stmts(&self.kids(node), Some(n))?;
        self.scopes.pop();
        if self.nodes[n].children.is_empty() {
            self.flag(n, EMPTY);
        }
        Ok(())
    }

    /// Nodes lowered as exactly one child of `parent` (an if branch, a
    /// case body): a lone block is that child; anything else goes in a
    /// synthetic block, kept only when it holds other than one node.
    pub(super) fn branch(&mut self, nodes: &[Node<'_>], parent: usize, at: Node<'_>) -> R {
        if let [only] = nodes
            && self.is(&self.flow.block_kinds, *only)
        {
            return self.labelled_block(*only, Some(parent));
        }
        let b = self.synth(Some(parent), BLOCK, at, "<synthetic:block>");
        self.push_scope();
        self.stmts(nodes, Some(b))?;
        self.scopes.pop();
        match self.nodes[b].children[..] {
            [] => self.flag(b, EMPTY),
            [only] => {
                let slot = self.nodes[parent].children.iter().position(|&c| c == b);
                self.nodes[parent].children[slot.expect("the block's own slot")] = only;
                self.nodes[only].parent = Some(parent);
            }
            _ => {}
        }
        Ok(())
    }

    /// An if (rule 2): the init and the condition read here, then one
    /// child per branch — an elif chain nested in the else position.
    fn if_(&mut self, node: Node<'_>, parent: Option<usize>) -> R {
        let n = self.add(parent, IF, node);
        self.if_body(node, n)
    }

    fn if_body(&mut self, node: Node<'_>, n: usize) -> R {
        let (outer, pos) = (self.push_scope(), &self.flow.r#if);
        let others = [
            pos.cond.as_str(),
            pos.then.as_str(),
            pos.r#else.as_str(),
            pos.init.as_str(),
        ];
        self.cur = Some(n);
        for init in self.at(node, &pos.init, &others) {
            self.simple(init);
        }
        let then_scope = (!self.flow.branch_binders.is_empty()).then(|| self.push_scope());
        let was = std::mem::replace(&mut self.binder_scope, then_scope);
        for c in self.at(node, &pos.cond, &others) {
            self.head(c);
        }
        self.binder_scope = was;
        let then = self.at(node, &pos.then, &others);
        self.branch(&then, n, node)?;
        if then_scope.is_some() {
            self.scopes.pop();
        }
        let alts = self.at(node, &pos.r#else, &others);
        self.else_chain(&alts, n)?;
        self.scopes.truncate(outer);
        Ok(())
    }

    /// The else position: an elif becomes an if of its own, the rest of
    /// the chain its else; an else wrapper gives its body (rule 2).
    fn else_chain(&mut self, alts: &[Node<'_>], n: usize) -> R {
        let Some((&first, rest)) = alts.split_first() else {
            return Ok(());
        };
        self.flag(n, ELSE);
        if self.is(&self.flow.elif_kinds, first) {
            let inner = self.add(Some(n), IF, first);
            self.if_body(first, inner)?;
            return self.else_chain(rest, inner);
        }
        match self.flow.else_kinds.iter().find(|(k, _)| k == first.kind()) {
            Some((_, body)) => {
                let body = self.at(first, body, &[]);
                self.branch(&body, n, first)
            }
            None => self.branch(&[first], n, first),
        }
    }

    /// A condition in a head: a wrapper read through (its init run
    /// first), a declaration declared, anything else walked.
    pub(super) fn head(&mut self, node: Node<'_>) {
        match self
            .flow
            .cond_wrappers
            .iter()
            .find(|(k, _, _)| k == node.kind())
        {
            Some((_, value, init)) => {
                for i in self.at(node, init, &[value.as_str()]) {
                    self.simple(i);
                }
                for v in self.at(node, value, &[init.as_str()]) {
                    self.head(v);
                }
            }
            None => self.simple(node),
        }
    }

    /// A declaration or an expression, its accesses at the current node
    /// (a wrapper read through: C++'s `init_statement`).
    pub(super) fn simple(&mut self, node: Node<'_>) {
        if self.is(&self.flow.wrapper_kinds, node)
            && let [only] = self.kids(node)[..]
        {
            return self.simple(only);
        }
        match self.decl_row(node) {
            Some(row) => self.decl(node, row),
            None => self.walk(node, &[]),
        }
    }

    pub(super) fn decl_row(&self, node: Node<'_>) -> Option<&'static Decl> {
        let f: &'static super::spec::FlowSpec = self.flow;
        f.decls.iter().find(|d| {
            d.kind == node.kind() && (d.token.is_empty() || super::at::holds(node, &d.token))
        })
    }

    /// A declaration statement (rule 8) — with a refutable pattern's
    /// alternative (Rust let-else) an if whose one branch must leave.
    fn declaration(&mut self, node: Node<'_>, row: &Decl, parent: Option<usize>) -> R {
        let alternative = self.at(node, &row.alternative, &[]);
        let kind = if alternative.is_empty() { STMT } else { IF };
        let n = self.add(parent, kind, node);
        self.cur = Some(n);
        self.decl(node, row);
        if kind == IF {
            self.branch(&alternative, n, node)?;
        }
        Ok(())
    }

    /// A statement no row names but holding a container as a direct
    /// child (Java `synchronized (x) { }`): a block whose head is read
    /// here, its containers lowered in order.
    fn headed_block(&mut self, node: Node<'_>, parent: Option<usize>) -> Result<bool, String> {
        let kids = self.kids(node);
        let (blocks, head): (Vec<Node<'_>>, Vec<Node<'_>>) = kids
            .into_iter()
            .partition(|k| self.is(&self.flow.block_kinds, *k));
        let nested = crate::scan::functions::is_unit_node(node, self.src, self.scan)
            || self.is(&self.flow.capture_kinds, node);
        if blocks.is_empty() || nested || self.callee(node).is_some() {
            return Ok(false);
        }
        let n = self.add(parent, BLOCK, node);
        self.cur = Some(n);
        head.iter().for_each(|h| self.walk(*h, &[]));
        for b in blocks {
            self.labelled_block(b, Some(n))?;
        }
        Ok(true)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/flow/tree.rs"]
mod tests;
