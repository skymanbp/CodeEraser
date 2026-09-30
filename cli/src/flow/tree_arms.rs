//! Switches and the arm dispatch (plan v2.31 step 4 A2; §5.1 rules 2,
//! 4 and 6): a switch, try, with or label found by its row; a switch
//! lowered with its head's accesses on its own node and one case per
//! arm as children, in the order the core's graph reads them. Loops
//! live in tree_loop.rs, tries and `with` in tree_try.rs.

use super::build::{CASE, ELSE, FALL, Lowerer, NORETURN, R, SWITCH};
use super::rows::{Case, DefaultArm, Fall, Scope, Switch};
use super::scope::Bind;
use super::tree_jumps::FrameKind;
use tree_sitter::Node;

impl Lowerer<'_> {
    /// A switch, try, with or label; false when the node is none.
    pub(super) fn arm_like(
        &mut self,
        node: Node<'_>,
        parent: Option<usize>,
    ) -> Result<bool, String> {
        let (f, k) = (self.flow, node.kind());
        if let Some(row) = f.switches.iter().find(|s| s.kind == k) {
            return self.switch(node, row, parent).map(|()| true);
        }
        if let Some(row) = f.tries.iter().find(|t| t.kind == k) {
            return self.try_(node, row, parent).map(|()| true);
        }
        if let Some(row) = f.withs.iter().find(|w| w.kind == k) {
            return self.with(node, row, parent).map(|()| true);
        }
        self.label(node, parent)
    }

    /// A switch or match (rules 2 and 6): the head read here, one case
    /// per arm; with no arms, Go's `select {}` never returns.
    fn switch(&mut self, node: Node<'_>, row: &Switch, parent: Option<usize>) -> R {
        let others = [&row.subject, &row.arms, &row.init, &row.binder].map(String::as_str);
        let containers = self.at(node, &row.arms, &others);
        let is_arm = |me: &Self, n: Node<'_>| me.flow.cases.iter().any(|c| c.kind == n.kind());
        let all: Vec<Node<'_>> = containers.iter().flat_map(|c| self.kids(*c)).collect();
        let (arms, stray): (Vec<Node<'_>>, Vec<Node<'_>>) =
            all.into_iter().partition(|n| is_arm(self, *n));
        let empty = arms.is_empty();
        let kind = if empty && row.empty_noreturn {
            NORETURN
        } else {
            SWITCH
        };
        let (n, scope) = self.open_arm(parent, kind, node, self.at(node, &row.init, &others));
        self.at(node, &row.subject, &others)
            .into_iter()
            .for_each(|s| self.head(s));
        for b in self.at(node, &row.binder, &others) {
            self.bind(b, Bind::local(scope, true), &[]);
        }
        if row.arms != "." {
            let was = std::mem::replace(&mut self.local_only, true);
            stray.iter().for_each(|s| self.walk(*s, &[]));
            self.local_only = was;
        }
        self.open_frame(
            node,
            n,
            FrameKind::Switch {
                passes: row.passes_break,
            },
        );
        let mut default = row.always_default;
        for arm in arms {
            default |= self.case(arm, n)?;
        }
        self.frames.pop();
        self.scopes.truncate(scope);
        if default && !empty && kind == SWITCH {
            self.flag(n, ELSE);
        }
        Ok(())
    }

    /// One arm (rules 2, 6, 8 and 9): its values read, its pattern bound,
    /// its guard read, its body as children. True when it is a default.
    fn case(&mut self, arm: Node<'_>, n: usize) -> Result<bool, String> {
        let row: &Case = self
            .flow
            .cases
            .iter()
            .find(|c| c.kind == arm.kind())
            .expect("an arm row");
        let others = [&row.pattern, &row.value, &row.guard, &row.body].map(String::as_str);
        let c = self.add(Some(n), CASE, arm);
        let scope = self.push_scope();
        self.cur = Some(c);
        let (patterns, guards) = (
            self.at(arm, &row.pattern, &others),
            self.at(arm, &row.guard, &others),
        );
        let guard_ids: Vec<usize> = guards.iter().map(|g| g.id()).collect();
        let mut claimed: Vec<usize> = patterns.iter().map(|p| p.id()).collect();
        claimed.extend(&guard_ids);
        for v in self.at(arm, &row.value, &others) {
            match self.decl_row(v) {
                Some(_) => self.simple(v),
                None => self.walk(v, &claimed),
            }
        }
        let idx = self.scope_for(Scope::Block);
        patterns
            .iter()
            .for_each(|p| self.bind(*p, Bind::local(idx, true), &guard_ids));
        guards.iter().for_each(|g| self.walk(*g, &[]));
        let default = self.default_arm(arm, row, &others, guards.is_empty());
        let bodies = self.flat(self.at(arm, &row.body, &others));
        let falls = match row.fallthrough {
            Fall::Never => false,
            Fall::Always => true,
            Fall::Statement => bodies
                .last()
                .is_some_and(|b| self.is(&self.flow.fallthrough_kinds, *b)),
        };
        if falls {
            self.flag(c, FALL);
        }
        self.stmts(&bodies, Some(c))?;
        self.scopes.truncate(scope);
        Ok(default)
    }

    /// Whether an arm is the switch's default (rule 6): by kind, by a
    /// missing pattern, or by a catch-all token with no guard.
    fn default_arm(&self, arm: Node<'_>, row: &Case, others: &[&str], unguarded: bool) -> bool {
        match &row.default {
            DefaultArm::Never => false,
            DefaultArm::Kind => true,
            DefaultArm::Missing(pos) => self.at(arm, pos, others).is_empty(),
            DefaultArm::Holds(pos, tok) => {
                let hit = self
                    .at(arm, pos, others)
                    .iter()
                    .any(|x| super::at::holds(*x, tok));
                hit && unguarded
            }
        }
    }

    /// Nodes with the splice containers opened (Go's statement_list).
    fn flat<'t>(&self, nodes: Vec<Node<'t>>) -> Vec<Node<'t>> {
        let splice = |n: Node<'t>| self.is(&self.flow.splice_kinds, n);
        nodes
            .into_iter()
            .flat_map(|n| {
                if splice(n) {
                    self.flat(self.kids(n))
                } else {
                    vec![n]
                }
            })
            .collect()
    }
}
