//! Cognitive Complexity per the SonarSource whitepaper v1.7 rules:
//! (1) structural increments carry a nesting penalty, (2) `else` /
//! `elif` / `else if` are flat +1 hybrids with no penalty, (3) lambdas
//! raise nesting without incrementing, (4) a run of like boolean
//! operators counts once, each alternation counts again, (5) labeled
//! jumps (`goto L` / `break L` / `continue L`) are fundamental +1
//! (p.8); plain jumps are free. Operator runs use coc_operators, not
//! cc_operators: CoC ignores null-coalescing shorthand (p.6).
//!
//! else-branch handling: TS/Rust/Python/C/Lua surface else as a node
//! kind (coc_flat_kinds); Go, Java and R hang it in the if's
//! `alternative` field, so the walker is field-aware, and it knows an
//! if by the exact if_kinds table — a prefix test would read every
//! kind spelled `if…` as one. An `else if` never pays double: the
//! wrapping clause yields to the inner if, which scores flat +1 and
//! keeps its children at the chain's nesting level (Sonar Appendix B).
//!
//! Headers do not nest (user ruling 2026-09-24, all languages): a
//! structure's condition, loop clause, switch value or catch parameter
//! scores at the structure's own level and only its body raises
//! nesting; each coc_nesting_kinds entry names its structure's body.
//! The whitepaper lists the structures that nest (p.9, Appendix B2)
//! and is silent on their headers. The oracles agree only on an if's
//! own condition, which sonar-java and PMD scan before nesting++; PMD
//! 7.27.0 nests the loops', the switch's and an else-if's headers
//! (register D31). An else-if's or an elif's condition sits at the
//! chain's level, where the first if's does. The ternaries' entries
//! name no body, so they nest whole: every operand is an expression,
//! and PMD nests the ternary's condition too.
//!
//! What this module does NOT do: the recursion increment (p.8,
//! Appendix B1 — one point for each function in a recursion cycle).
//! That rule needs a call relation, so it is split (ADR-008 fourth
//! instalment): scan/calls.rs states the arcs, the core finds the
//! cycles, and scan/coc.rs writes the settled value back. `measure`
//! here therefore answers the PRE-cycle number, on purpose — it is
//! what the structure family, the one reader that never looks at
//! complexity, should keep getting. Everything that grades complexity
//! reads scan::settle instead.

use crate::scan::ast::{self, operator_text};
use crate::scan::spec::{Kinds, LangSpec};
use tree_sitter::Node;

pub struct Cognitive {
    pub score: u32,
    pub max_nesting: u32,
}

pub fn measure(fn_node: Node<'_>, src: &[u8], spec: &LangSpec) -> Cognitive {
    let mut w = Walker {
        src,
        spec,
        score: 0,
        max_nesting: 0,
    };
    w.walk_children(fn_node, 0);
    Cognitive {
        score: w.score,
        max_nesting: w.max_nesting,
    }
}

struct Walker<'s, 'sp> {
    src: &'s [u8],
    spec: &'sp LangSpec,
    score: u32,
    max_nesting: u32,
}

impl Walker<'_, '_> {
    fn walk_children(&mut self, node: Node<'_>, nesting: u32) {
        for child in super::walk::measured(node, self.spec) {
            self.visit(child, nesting);
        }
    }

    fn visit(&mut self, node: Node<'_>, nesting: u32) {
        // keyword tokens can share their structure's kind name
        // (Haskell: the anon `case` token inside the `case` node) —
        // the kind tables below describe named STRUCTURE nodes only,
        // and anon tokens are leaves with nothing to walk
        if !node.is_named() {
            return;
        }
        let kind = node.kind();
        if crate::scan::functions::is_unit_node(node, self.src, self.spec) {
            return; // nested standalone unit: measured separately
        }
        if self.is_logic_root(node) {
            self.score += operator_runs(node, self.src, self.spec);
        }
        if self.spec.chain_kinds.contains(&kind) {
            self.score += 1; // one run of anonymous `&&` (let_chain)
        }
        if let Some(entry) = entry_of(self.spec.coc_nesting_kinds, kind) {
            self.structural(node, entry, nesting);
        } else if let Some(entry) = entry_of(self.spec.coc_flat_kinds, kind) {
            // An else-clause wrapping an if (`else if`) yields its +1
            // to the inner if; a plain else/elif pays here.
            if !has_child_of(node, self.spec.if_kinds) {
                self.score += 1;
            }
            // a flat branch that carries a condition (Python's elif)
            // holds it at the chain's level, like an else-if
            self.walk_split(node, entry, nesting.saturating_sub(1), nesting);
        } else if self.spec.coc_nest_only_kinds.contains(&kind) {
            self.walk_children(node, nesting + 1);
        } else {
            if self.spec.coc_jump_kinds.contains(&kind) && has_child_of(node, self.spec.label_kinds)
            {
                self.score += 1; // fundamental: no nesting penalty
            }
            self.walk_children(node, nesting);
        }
    }

    fn structural(&mut self, node: Node<'_>, entry: &str, nesting: u32) {
        self.field_else_bonus(node);
        if self.is_else_if(node) {
            self.score += 1;
            // the chain's level: the body where the first if's sits,
            // the condition where that if's own condition does
            self.walk_split(node, entry, nesting.saturating_sub(1), nesting);
        } else {
            self.score += 1 + nesting;
            self.max_nesting = self.max_nesting.max(nesting + 1);
            self.walk_split(node, entry, nesting, nesting + 1);
        }
    }

    /// A structure's children: its body (the positions its entry names
    /// after the kind) at `body`, every other child (its header) at
    /// `head`. An entry that names no position is all body: the
    /// ternaries nest whole.
    fn walk_split(&mut self, node: Node<'_>, entry: &str, head: u32, body: u32) {
        let spots: Vec<&str> = entry.split(' ').skip(1).collect();
        if spots.is_empty() {
            return self.walk_children(node, body);
        }
        // a field can hold several children (Python's elif branches
        // all hang on `alternative`), so every one of them is read
        let mut cursor = node.walk();
        let mut fielded = Vec::new();
        for spot in &spots {
            fielded.extend(
                node.children_by_field_name(spot, &mut cursor)
                    .map(|c| c.id()),
            );
        }
        for child in super::walk::measured(node, self.spec) {
            let in_body = fielded.contains(&child.id()) || spots.contains(&child.kind());
            self.visit(child, if in_body { body } else { head });
        }
    }

    /// A plain else hung off the if's `alternative` FIELD: anything
    /// there but the next if or an else node is the else body itself —
    /// a Go block, or a Java single statement (`else return x;`).
    fn field_else_bonus(&mut self, node: Node<'_>) {
        if !self.spec.if_kinds.contains(&node.kind()) {
            return;
        }
        if node.child_by_field_name("alternative").is_some_and(|a| {
            !self.spec.if_kinds.contains(&a.kind())
                && entry_of(self.spec.coc_flat_kinds, a.kind()).is_none()
        }) {
            self.score += 1;
        }
    }

    fn is_else_if(&self, node: Node<'_>) -> bool {
        if !self.spec.if_kinds.contains(&node.kind()) {
            return false;
        }
        let Some(parent) = node.parent() else {
            return false;
        };
        if entry_of(self.spec.coc_flat_kinds, parent.kind()).is_some() {
            return true; // TS/Rust/Python/C: if directly under an else clause
        }
        self.spec.if_kinds.contains(&parent.kind())
            && parent
                .child_by_field_name("alternative")
                .is_some_and(|a| a.id() == node.id()) // Go, Java: if as alternative
    }

    /// Root of a maximal boolean chain: a short-circuit node whose
    /// parent is not one.
    fn is_logic_root(&self, node: Node<'_>) -> bool {
        logic_op(node, self.src, self.spec).is_some()
            && node
                .parent()
                .is_none_or(|p| logic_op(p, self.src, self.spec).is_none())
    }
}

/// A kind's entry in a kind table: the kind, then, for a structure
/// that nests or a branch that carries a condition, the positions of
/// its body (spec.rs, coc_nesting_kinds).
fn entry_of(table: Kinds, kind: &str) -> Option<&'static str> {
    table
        .iter()
        .copied()
        .find(|e| e.split(' ').next() == Some(kind))
}

/// Whether a direct child of `node` is one of `kinds` — a jump's
/// label, the if an else clause wraps.
fn has_child_of(node: Node<'_>, kinds: &[&str]) -> bool {
    ast::children(node)
        .iter()
        .any(|c| kinds.contains(&c.kind()))
}

/// Operator runs in source order within one boolean chain:
/// `a && b && c` = 1, `a || b && c || d` = 3 (three runs).
fn operator_runs(root: Node<'_>, src: &[u8], spec: &LangSpec) -> u32 {
    let mut ops = Vec::new();
    collect_in_order(root, src, spec, &mut ops);
    let mut runs = 0;
    let mut prev: Option<&str> = None;
    for op in ops {
        if prev != Some(op) {
            runs += 1;
        }
        prev = Some(op);
    }
    runs
}

/// In-order (left, self, right) so runs reflect source token order.
/// Operand fields differ by grammar family: python/ts/rust/go/lua
/// expose `left`/`right`, tree-sitter-haskell `left_operand` /
/// `right_operand` (AST-probed 3k), tree-sitter-r `lhs`/`rhs` (plan
/// v2.30 step 4) — one alias lookup instead of a per-lang walker.
fn collect_in_order<'s>(node: Node<'_>, src: &'s [u8], spec: &LangSpec, out: &mut Vec<&'s str>) {
    let Some(op) = logic_op(node, src, spec) else {
        return;
    };
    let operand = |fields: [&str; 3]| fields.iter().find_map(|f| node.child_by_field_name(f));
    if let Some(left) = operand(["left", "left_operand", "lhs"]) {
        collect_in_order(left, src, spec, out);
    }
    out.push(op);
    if let Some(right) = operand(["right", "right_operand", "rhs"]) {
        collect_in_order(right, src, spec, out);
    }
}

fn logic_op<'s>(node: Node<'_>, src: &'s [u8], spec: &LangSpec) -> Option<&'s str> {
    let op = operator_text(node, src)?;
    spec.coc_operators.contains(&op).then_some(op)
}
