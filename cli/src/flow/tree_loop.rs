//! Loops (plan v2.31 step 4 A2; §5.1 rules 2, 4, 5 and 8): the head's
//! accesses on the loop node, the body as its one child, then the
//! update label a C-style for's continues reach; the loop target
//! written or declared each round; a constant condition infinite.

use super::build::{BLOCK, EMPTY, INFINITE, LABEL, LOOP, Lowerer, R, WRITE};
use super::rows::{Loop, Scope};
use super::scope::Bind;
use super::tree_jumps::FrameKind;
use tree_sitter::Node;

impl Lowerer<'_> {
    fn loop_row<'t>(&self, node: Node<'t>) -> Option<(&'static Loop, Node<'t>)> {
        let f: &'static super::spec::FlowSpec = self.flow;
        f.loops
            .iter()
            .filter(|l| l.kind == node.kind())
            .find_map(|row| {
                if row.header.is_empty() {
                    return Some((row, node));
                }
                let others = [row.body.as_str(), row.r#else.as_str()];
                self.at(node, &row.header, &others)
                    .into_iter()
                    .next()
                    .map(|h| (row, h))
            })
    }

    /// A loop (rules 2, 4, 5 and 8): the init before it in a scope
    /// wrapping both, the head on the loop node, the body, the update
    /// label a C-style for's continues reach, Python's else after it.
    pub(super) fn loop_(&mut self, node: Node<'_>, parent: Option<usize>) -> R {
        let Some((row, holder)) = self.loop_row(node) else {
            self.plain(node, parent);
            return Ok(());
        };
        let head = [&row.cond, &row.init, &row.update, &row.target, &row.iter].map(String::as_str);
        let pick = |me: &Self, pos: &str| {
            let found = me.at(holder, pos, &head);
            found
                .into_iter()
                .filter(|n| !me.is(&me.flow.empty_kinds, *n))
                .collect::<Vec<_>>()
        };
        let conds = pick(self, &row.cond);
        let updates = pick(self, &row.update);
        let wrap = self.push_scope();
        self.stmts(&pick(self, &row.init), parent)?;
        let n = self.add(parent, LOOP, node);
        self.open_frame(node, n, FrameKind::Loop);
        let elses = self.loop_labels(node, row, &updates);
        let (targets, iters) = (pick(self, &row.target), pick(self, &row.iter));
        if self.infinite(row, &conds, &iters) {
            self.flag(n, INFINITE);
        }
        let own = self.push_scope();
        self.cur = Some(n);
        let was = self.binder_scope.replace(own);
        if !row.body_first {
            conds.iter().for_each(|c| self.head(*c));
        }
        iters.iter().for_each(|i| self.walk(*i, &[]));
        self.loop_target(row, &targets, &iters);
        let bodies = self.at(node, &row.body, &[row.r#else.as_str()]);
        self.loop_body(&bodies, n, node, row.body_first.then_some(&conds[..]))?;
        self.binder_scope = was;
        let frame = self.frames.last().expect("the loop's frame");
        if let Some(l) = frame.continue_to {
            self.attach(Some(n), l);
            self.stmts(&updates, Some(l))?;
        }
        let frame = self.frames.pop().expect("the loop's frame");
        self.scopes.truncate(own);
        self.loop_else(elses.first().copied(), frame.break_to, parent)?;
        self.scopes.truncate(wrap);
        Ok(())
    }

    /// The loop frame's two synthetic labels, detached until placed: the
    /// update's (a continue's target) and Python's else's (a break's).
    fn loop_labels<'t>(
        &mut self,
        node: Node<'t>,
        row: &Loop,
        updates: &[Node<'_>],
    ) -> Vec<Node<'t>> {
        let frame = self.frames.len() - 1;
        if let Some(u) = updates.first() {
            self.frames[frame].continue_to = Some(self.detached(LABEL, *u, "<synthetic:continue>"));
        }
        let elses = self.at(node, &row.r#else, &[row.body.as_str()]);
        if let Some(e) = elses.first() {
            self.frames[frame].break_to = Some(self.detached(LABEL, *e, "<synthetic:else-label>"));
        }
        elses
    }

    /// No condition and no iteration, or a constant one (rule 5).
    fn infinite(&self, row: &Loop, conds: &[Node<'_>], iters: &[Node<'_>]) -> bool {
        match row.until {
            true => conds.iter().any(|c| self.constant(*c, false)),
            false => {
                (conds.is_empty() && iters.is_empty())
                    || conds.iter().any(|c| self.constant(*c, true))
            }
        }
    }

    /// Python's for/while else: its body after the loop, then the label
    /// a break inside the loop reaches, past the else.
    fn loop_else(&mut self, e: Option<Node<'_>>, label: Option<usize>, parent: Option<usize>) -> R {
        let (Some(e), Some(l)) = (e, label) else {
            return Ok(());
        };
        self.else_part(e, parent)?;
        self.attach(parent, l);
        Ok(())
    }

    /// The body, one child; a condition checked after it (do-while,
    /// repeat) read on the loop node from inside the body's scope, where
    /// Lua's `until` sees the body's locals.
    fn loop_body(
        &mut self,
        bodies: &[Node<'_>],
        n: usize,
        node: Node<'_>,
        after: Option<&[Node<'_>]>,
    ) -> R {
        let Some(conds) = after else {
            return self.branch(bodies, n, node);
        };
        let b = match bodies {
            [only] if self.is(&self.flow.block_kinds, *only) => self.add(Some(n), BLOCK, *only),
            _ => self.synth(Some(n), BLOCK, node, "<synthetic:block>"),
        };
        let inner = self.push_scope();
        let stmts: Vec<Node<'_>> = match bodies {
            [only] if self.is(&self.flow.block_kinds, *only) => self.kids(*only),
            _ => bodies.to_vec(),
        };
        self.stmts(&stmts, Some(b))?;
        if self.nodes[b].children.is_empty() {
            self.flag(b, EMPTY);
        }
        self.cur = Some(n);
        conds.iter().for_each(|c| self.head(*c));
        self.scopes.truncate(inner);
        Ok(())
    }

    /// The loop's target, written each round (rule 8): declared where a
    /// marker token says so or the row declares, else written.
    fn loop_target(&mut self, row: &Loop, targets: &[Node<'_>], iters: &[Node<'_>]) {
        let Some(holder) = targets.first().and_then(|t| t.parent()) else {
            return;
        };
        let marked = row
            .marker
            .iter()
            .find(|(tok, _)| super::at::holds(holder, tok));
        let declare = match (row.marker.is_empty(), marked) {
            (false, Some((_, scope))) => Some(self.scope_for(*scope)),
            (false, None) => None,
            (true, _) if self.flow.first_write_declares => None,
            (true, _) => Some(self.scope_for(Scope::Block)),
        };
        for t in targets {
            if self.holds_kind(*t, &self.flow.ref_binding_kinds) {
                iters.iter().for_each(|i| self.address_mark(*i));
            }
            match declare {
                Some(idx) => self.bind(*t, Bind::local(idx, true), &[]),
                None => self.target(*t, WRITE),
            }
        }
    }

    /// A literal a loop condition cannot change (rule 5): read through
    /// the condition wrappers, true or (for `until`) false.
    fn constant(&self, node: Node<'_>, truth: bool) -> bool {
        if let Some((_, value, init)) = self
            .flow
            .cond_wrappers
            .iter()
            .find(|(k, _, _)| k == node.kind())
        {
            let inner = self.at(node, value, &[init.as_str()]);
            return self.at(node, init, &[value.as_str()]).is_empty()
                && matches!(inner[..], [one] if self.constant(one, truth));
        }
        let table = if truth {
            &self.flow.const_true
        } else {
            &self.flow.const_false
        };
        let literal = table.iter().any(|(kind, tok)| {
            node.kind() == kind && (tok.is_empty() || super::at::holds(node, tok))
        });
        let number = truth
            && self.is(&self.flow.int_kinds, node)
            && self
                .text(node)
                .chars()
                .any(|c| c.is_ascii_digit() && c != '0');
        literal || number
    }
}
