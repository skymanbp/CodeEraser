//! Jumps and their targets (plan v2.31 step 4 A2; §5.1 rules 3 and 4):
//! the frames a break or continue may leave, the labels a goto lands
//! on, and the rewrites that keep every jump inside the tree contract —
//! a break out of a labelled block, out of a Python loop with an else,
//! and a continue past a C-style update each become a goto to a
//! synthetic label.

use super::build::{BREAK, CONTINUE, GOTO, LABEL, Lowerer, R, RETURN, THROW};
use tree_sitter::Node;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FrameKind {
    Loop,
    /// A switch; `passes` = a break inside leaves the enclosing loop
    /// (Rust and Python match).
    Switch {
        passes: bool,
    },
    /// A labelled statement that is no loop or switch: only a labelled
    /// break leaves it, as a goto to a label after it.
    Labelled,
}

pub(super) struct Frame {
    pub node: usize,
    pub kind: FrameKind,
    pub labels: Vec<String>,
    /// Where a break of this loop goes instead (Python's else label).
    pub break_to: Option<usize>,
    /// Where a continue of this loop goes instead (the update label).
    pub continue_to: Option<usize>,
    /// The label after a labelled statement, made on the first break.
    pub trailing: Option<usize>,
}

impl Lowerer<'_> {
    /// A frame for a loop or switch, taking the labels a labelled
    /// statement left for it and its own (Rust `'a: loop`).
    pub(super) fn open_frame(&mut self, node: Node<'_>, n: usize, kind: FrameKind) {
        let mut labels = std::mem::take(&mut self.pending_labels);
        labels.extend(self.self_label(node));
        self.frames.push(Frame {
            node: n,
            kind,
            labels,
            break_to: None,
            continue_to: None,
            trailing: None,
        });
    }

    pub(super) fn self_label(&self, node: Node<'_>) -> Option<String> {
        let label = self.at(node, &self.flow.self_label, &[]).into_iter().next();
        label.map(|l| self.text(l))
    }

    pub(super) fn labelled_frame(&mut self, n: usize, labels: Vec<String>) {
        self.frames.push(Frame {
            node: n,
            kind: FrameKind::Labelled,
            labels,
            break_to: None,
            continue_to: None,
            trailing: None,
        });
    }

    /// Closes a labelled frame, its trailing label (if a break made
    /// one) placed right after the statement.
    pub(super) fn close_labelled(&mut self, n: usize, parent: Option<usize>) {
        let frame = self.frames.pop().expect("the labelled frame");
        debug_assert_eq!(frame.node, n);
        if let Some(t) = frame.trailing {
            self.attach(parent, t);
        }
    }

    /// A jump or exit statement, lowered; false when the node is none.
    pub(super) fn jump(&mut self, node: Node<'_>, parent: Option<usize>) -> Result<bool, String> {
        let f = self.flow;
        let kind = [
            (&f.return_kinds, RETURN),
            (&f.throw_kinds, THROW),
            (&f.break_kinds, BREAK),
            (&f.continue_kinds, CONTINUE),
            (&f.yield_kinds, BREAK),
        ]
        .into_iter()
        .find(|(kinds, _)| self.is(kinds, node))
        .map(|(_, k)| k);
        if let Some((_, pos)) = f.gotos.iter().find(|(k, _)| k == node.kind()) {
            let n = self.add(parent, GOTO, node);
            let name = self.at(node, pos, &[]).first().map(|l| self.text(*l));
            self.gotos.push((n, name.unwrap_or_default()));
            return Ok(true);
        }
        let Some(kind) = kind else { return Ok(false) };
        let n = self.add(parent, kind, node);
        self.cur = Some(n);
        self.walk(node, &[]);
        if kind == RETURN && self.kids(node).is_empty() {
            for v in self.results.clone() {
                self.access(v, super::build::READ);
            }
        }
        let yields = self.is(&f.yield_kinds, node);
        match kind {
            BREAK => self.aim_break(node, n, yields),
            CONTINUE => self.aim_continue(node, n),
            _ => Ok(()),
        }
        .map(|()| true)
    }

    fn label_of(&self, node: Node<'_>) -> Option<String> {
        let label = self
            .kids(node)
            .into_iter()
            .find(|c| self.scan.label_kinds.contains(&c.kind()));
        label.map(|l| self.text(l))
    }

    fn frame_for(&self, label: Option<&str>, fits: impl Fn(FrameKind) -> bool) -> Option<usize> {
        self.frames.iter().rposition(|f| match label {
            Some(l) => f.labels.iter().any(|x| x == l),
            None => fits(f.kind),
        })
    }

    fn missing(&self, n: usize, what: &str) -> String {
        let at = self.nodes[n].at;
        format!("{what} at {}:{} has no target", at.0, at.1)
    }

    fn aim_break(&mut self, node: Node<'_>, n: usize, yields: bool) -> R {
        let label = if yields { None } else { self.label_of(node) };
        let fits = |k| match k {
            FrameKind::Loop => !yields,
            FrameKind::Switch { passes } => yields || !passes,
            FrameKind::Labelled => false,
        };
        let i = self
            .frame_for(label.as_deref(), fits)
            .ok_or_else(|| self.missing(n, "a break"))?;
        let frame = &self.frames[i];
        let (kind, aux) = match (frame.kind, frame.break_to) {
            (FrameKind::Loop, Some(to)) => (GOTO, to),
            (FrameKind::Labelled, _) => (GOTO, self.trailing(i)),
            _ => (BREAK, frame.node),
        };
        self.nodes[n].kind = kind;
        self.nodes[n].aux = Some(aux);
        Ok(())
    }

    fn trailing(&mut self, i: usize) -> usize {
        if let Some(t) = self.frames[i].trailing {
            return t;
        }
        let (at, end) = {
            let frame = &self.nodes[self.frames[i].node];
            (frame.at, frame.end)
        };
        let t = self.nodes.len();
        self.nodes.push(super::build::TNode {
            kind: LABEL,
            flags: 0,
            parent: None,
            children: Vec::new(),
            aux: None,
            at,
            end,
            text: "<synthetic:break-label>".to_owned(),
            uses: Vec::new(),
        });
        self.frames[i].trailing = Some(t);
        t
    }

    fn aim_continue(&mut self, node: Node<'_>, n: usize) -> R {
        let label = self.label_of(node);
        let i = self
            .frame_for(label.as_deref(), |k| k == FrameKind::Loop)
            .filter(|&i| self.frames[i].kind == FrameKind::Loop)
            .ok_or_else(|| self.missing(n, "a continue"))?;
        let frame = &self.frames[i];
        let (kind, aux) = frame
            .continue_to
            .map_or((CONTINUE, frame.node), |to| (GOTO, to));
        self.nodes[n].kind = kind;
        self.nodes[n].aux = Some(aux);
        Ok(())
    }

    /// A label statement (rule 3): kind 14 holding its statement; a
    /// loop or switch under it takes the name for its breaks and
    /// continues, anything else a labelled frame.
    pub(super) fn label(&mut self, node: Node<'_>, parent: Option<usize>) -> Result<bool, String> {
        let Some((_, name_pos, stmt_pos)) =
            self.flow.labels.iter().find(|(k, _, _)| k == node.kind())
        else {
            return Ok(false);
        };
        let name = self
            .at(node, name_pos, &[])
            .first()
            .map(|l| self.text(*l))
            .unwrap_or_default();
        let n = self.add(parent, LABEL, node);
        self.labels.push((n, name.clone()));
        let inner = self.at(node, stmt_pos, &[name_pos.as_str()]);
        let framed = match inner[..] {
            [only] => self.frames_itself(only),
            _ => false,
        };
        if framed {
            self.pending_labels.push(name);
            self.stmts(&inner, Some(n))?;
            self.pending_labels.clear();
        } else {
            self.labelled_frame(n, vec![name]);
            self.stmts(&inner, Some(n))?;
            self.close_labelled(n, parent);
        }
        Ok(true)
    }

    /// A loop or switch, through its wrappers: it opens its own frame.
    fn frames_itself(&self, node: Node<'_>) -> bool {
        let f = self.flow;
        if self.is(&f.wrapper_kinds, node)
            && let [only] = self.kids(node)[..]
        {
            return self.frames_itself(only);
        }
        f.loops.iter().any(|l| l.kind == node.kind())
            || f.switches.iter().any(|s| s.kind == node.kind())
    }

    /// Every goto aimed at its label: the one in the nearest container
    /// around the goto, else the unit's first of that name.
    pub(super) fn resolve_gotos(&mut self) -> R {
        for (g, name) in std::mem::take(&mut self.gotos) {
            let mut chain = vec![self.nodes[g].parent];
            while let Some(Some(p)) = chain.last().copied() {
                chain.push(self.nodes[p].parent);
            }
            let found = self
                .labels
                .iter()
                .filter(|(_, l)| *l == name)
                .min_by_key(|(n, _)| {
                    let p = self.nodes[*n].parent;
                    chain.iter().position(|c| *c == p).unwrap_or(usize::MAX)
                });
            let Some(&(label, _)) = found else {
                return Err(format!("a goto to `{name}` has no label"));
            };
            self.nodes[g].aux = Some(label);
        }
        Ok(())
    }
}
