//! The lowering's working state (plan v2.31 step 4 A2): the statement
//! tree as it is built, the variables, the scope stack, and the one
//! unit entry — parameters, the synthetic default-argument statement,
//! the body — with the numbering pass that turns the tree into rows.
//! A unit whose function-wide names (Python, R, TypeScript `var`) are
//! bound late is lowered twice: the first run finds the names, the
//! second declares them up front, so a read the source places before
//! the binding (a loop's back edge) still resolves (§5.1 rule 8).

use super::at::{self, kids};
use super::spec::FlowSpec;
use crate::scan::spec::LangSpec;
use std::collections::{HashMap, HashSet};
use tree_sitter::Node;

pub(super) type R = Result<(), String>;

/// A group of codes numbered from zero in the order written — the
/// order the core numbers them in (CE.Flow.Cost): the enum's own
/// discriminants, each re-exported as the i64 the tables carry.
macro_rules! codes {
    ($group:ident: $($name:ident),+ $(,)?) => {
        #[allow(clippy::upper_case_acronyms)]
        enum $group {
            $($name),+
        }
        $(pub(super) const $name: i64 = $group::$name as i64;)+
    };
}

codes!(Kind: BLOCK, STMT, IF, LOOP, SWITCH, CASE, TRY, CATCH, FINALLY, RETURN, THROW, BREAK,
    CONTINUE, GOTO, LABEL, NORETURN);
codes!(StmtFlag: ELSE, INFINITE, FALL, DYNAMIC, EMPTY);
codes!(VarFlag: PARAM, CAPTURED, IGNORED, ADDRESS);
codes!(Mode: READ, WRITE, RW);

/// A statement as it is built: its children in order, its jump target
/// as a node index, its accesses in evaluation order.
pub(super) struct TNode {
    pub kind: i64,
    pub flags: i64,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub aux: Option<usize>,
    pub at: (u32, u32),
    pub text: String,
    pub uses: Vec<(usize, i64)>,
}

pub(super) struct Var {
    pub name: String,
    /// The declaring node; None for a parameter.
    pub decl: Option<usize>,
    pub flags: i64,
    pub at: (u32, u32),
}

/// A binding found in the first run, declared up front in the second:
/// (name, declaring node, place).
pub(super) type Early = (String, usize, (u32, u32));

pub(super) struct Lowerer<'s> {
    pub src: &'s [u8],
    pub flow: &'static FlowSpec,
    pub scan: &'static LangSpec,
    /// Callee names this file declares noreturn (§5.1 rule 3).
    pub doomed: &'s HashSet<String>,
    pub nodes: Vec<TNode>,
    pub roots: Vec<usize>,
    pub vars: Vec<Var>,
    pub scopes: Vec<HashMap<String, usize>>,
    pub frames: Vec<super::tree_jumps::Frame>,
    pub pending_labels: Vec<String>,
    pub gotos: Vec<(usize, String)>,
    pub labels: Vec<(usize, String)>,
    /// Names a nonlocal statement keeps out (Python `global`).
    pub globals: HashSet<String>,
    pub early: Vec<Early>,
    pub found: Vec<Early>,
    /// The node the walker's accesses land on; None while parameters
    /// are bound.
    pub cur: Option<usize>,
    /// The walker is in a conditional position (rule 9).
    pub cond: bool,
    /// The walker is in a scope whose bindings are not the unit's.
    pub local_only: bool,
    /// Where pattern binders declare (a branch's own scope).
    pub binder_scope: Option<usize>,
    /// The scope the current statement sits in.
    pub stmt_scope: usize,
    /// Named results, read by a bare return (Go).
    pub results: Vec<usize>,
}

impl<'s> Lowerer<'s> {
    fn new(
        src: &'s [u8],
        flow: &'static FlowSpec,
        scan: &'static LangSpec,
        doomed: &'s HashSet<String>,
        early: Vec<Early>,
    ) -> Self {
        Self {
            src,
            flow,
            scan,
            doomed,
            nodes: Vec::new(),
            roots: Vec::new(),
            vars: Vec::new(),
            scopes: Vec::new(),
            frames: Vec::new(),
            pending_labels: Vec::new(),
            gotos: Vec::new(),
            labels: Vec::new(),
            globals: HashSet::new(),
            early,
            found: Vec::new(),
            cur: None,
            cond: false,
            local_only: false,
            binder_scope: None,
            stmt_scope: 0,
            results: Vec::new(),
        }
    }

    /// One unit lowered, twice where a function-wide name was bound.
    pub fn run(
        unit: Node<'_>,
        src: &'s [u8],
        flow: &'static FlowSpec,
        scan: &'static LangSpec,
        doomed: &'s HashSet<String>,
    ) -> Result<Self, String> {
        let mut first = Self::new(src, flow, scan, doomed, Vec::new());
        first.unit(unit)?;
        if first.found.is_empty() {
            return Ok(first);
        }
        let mut second = Self::new(src, flow, scan, doomed, std::mem::take(&mut first.found));
        second.unit(unit)?;
        Ok(second)
    }

    fn unit(&mut self, unit: Node<'_>) -> R {
        self.scopes.push(HashMap::new());
        self.prescan_globals(unit);
        self.params(unit);
        for (name, node, at) in std::mem::take(&mut self.early) {
            if !self.scopes[0].contains_key(&name) {
                let v = self.new_var(&name, Some(node), at);
                self.scopes[0].insert(name, v);
            }
        }
        self.defaults(unit);
        match unit.child_by_field_name("body") {
            None => {}
            Some(body) if self.flow.block_kinds.iter().any(|k| k == body.kind()) => {
                self.scopes.push(HashMap::new());
                self.stmts(&kids(body, self.scan.comment_kinds), None)?;
                self.scopes.pop();
            }
            Some(expr) => {
                let r = self.add(None, RETURN, expr);
                self.cur = Some(r);
                self.walk(expr, &[]);
            }
        }
        if unit.has_error() && !self.nodes.iter().any(|n| n.flags >> DYNAMIC & 1 == 1) {
            let at = self.roots.first().copied();
            let n = at.unwrap_or_else(|| self.synth(None, STMT, unit, "<synthetic:error>"));
            self.flag(n, DYNAMIC);
        }
        self.resolve_gotos()
    }

    /// A node from source, attached to its parent (None = the unit body).
    pub fn add(&mut self, parent: Option<usize>, kind: i64, src: Node<'_>) -> usize {
        let text = at::first_line(src, self.src);
        self.make(parent, kind, at::place(src), text)
    }

    /// A synthetic node standing at a source node's place.
    pub fn synth(&mut self, parent: Option<usize>, kind: i64, src: Node<'_>, text: &str) -> usize {
        self.make(parent, kind, at::place(src), text.to_owned())
    }

    fn make(&mut self, parent: Option<usize>, kind: i64, at: (u32, u32), text: String) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(TNode {
            kind,
            flags: 0,
            parent: None,
            children: Vec::new(),
            aux: None,
            at,
            text,
            uses: Vec::new(),
        });
        self.attach(parent, idx);
        idx
    }

    /// Hangs a node under a parent (None = the unit body; a detached
    /// node, made with `usize::MAX`, waits for its place).
    pub fn attach(&mut self, parent: Option<usize>, idx: usize) {
        match parent {
            Some(usize::MAX) => {}
            Some(p) => {
                self.nodes[p].children.push(idx);
                self.nodes[idx].parent = Some(p);
            }
            None => self.roots.push(idx),
        }
    }

    pub fn flag(&mut self, idx: usize, bit: i64) {
        self.nodes[idx].flags |= 1 << bit;
    }

    /// The current statement cannot be read: the unit is not judged.
    pub fn dynamic_here(&mut self) {
        if let Some(c) = self.cur {
            self.flag(c, DYNAMIC);
        }
    }

    pub fn is(&self, kinds: &[String], node: Node<'_>) -> bool {
        kinds.iter().any(|k| k == node.kind())
    }

    pub fn text(&self, node: Node<'_>) -> String {
        node.utf8_text(self.src).unwrap_or("").trim().to_owned()
    }

    pub fn kids<'t>(&self, node: Node<'t>) -> Vec<Node<'t>> {
        kids(node, self.scan.comment_kinds)
    }

    /// A position under a node (at.rs), comments aside.
    pub fn at<'t>(&self, node: Node<'t>, pos: &str, others: &[&str]) -> Vec<Node<'t>> {
        at::at(node, pos, others, self.scan.comment_kinds)
    }
}
