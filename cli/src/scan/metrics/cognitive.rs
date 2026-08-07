//! Cognitive Complexity per the SonarSource whitepaper v1.7 rules:
//! (1) structural increments carry a nesting penalty, (2) `else` /
//! `elif` / `else if` are flat +1 hybrids with no penalty, (3) lambdas
//! raise nesting without incrementing, (4) a run of like boolean
//! operators counts once, each alternation counts again, (5) labeled
//! jumps (`goto L` / `break L` / `continue L`) are fundamental +1
//! (p.8); plain jumps are free. Operator runs use coc_operators, not
//! cc_operators: CoC ignores null-coalescing shorthand (p.6).
//!
//! else-branch handling: TS/Rust/Python surface else as a node kind
//! (coc_flat_kinds); Go hides it in the if's `alternative` field, so
//! the walker is field-aware. An `else if` never pays double: the
//! wrapping clause yields to the inner if, which scores flat +1 and
//! keeps its children at the chain's nesting level (Sonar Appendix B).
//!
//! Known M1 divergence (recorded, calibrated at cross-check):
//! recursion (+1 per Sonar) is not detected — needs a call graph (M5).

use crate::scan::ast::{self, operator_text};
use crate::scan::spec::LangSpec;
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
        for child in ast::children(node) {
            self.visit(child, nesting);
        }
    }

    fn visit(&mut self, node: Node<'_>, nesting: u32) {
        let kind = node.kind();
        if self.spec.fn_kinds.contains(&kind) {
            return; // nested standalone unit: measured separately
        }
        if self.is_logic_root(node) {
            self.score += operator_runs(node, self.src, self.spec);
        }
        if self.spec.chain_kinds.contains(&kind) {
            self.score += 1; // one run of anonymous `&&` (let_chain)
        }
        if self.spec.coc_nesting_kinds.contains(&kind) {
            self.structural(node, nesting);
        } else if self.spec.coc_flat_kinds.contains(&kind) {
            // An else-clause wrapping an if (`else if`) yields its +1
            // to the inner if; a plain else/elif pays here.
            if !has_if_child(node) {
                self.score += 1;
            }
            self.walk_children(node, nesting);
        } else if self.spec.coc_nest_only_kinds.contains(&kind) {
            self.walk_children(node, nesting + 1);
        } else {
            if self.spec.coc_jump_kinds.contains(&kind) && self.has_label(node) {
                self.score += 1; // fundamental: no nesting penalty
            }
            self.walk_children(node, nesting);
        }
    }

    fn has_label(&self, node: Node<'_>) -> bool {
        ast::children(node)
            .iter()
            .any(|c| self.spec.label_kinds.contains(&c.kind()))
    }

    fn structural(&mut self, node: Node<'_>, nesting: u32) {
        self.field_else_bonus(node);
        if self.is_else_if(node) {
            self.score += 1;
            self.walk_children(node, nesting); // stay at chain level
        } else {
            self.score += 1 + nesting;
            self.max_nesting = self.max_nesting.max(nesting + 1);
            self.walk_children(node, nesting + 1);
        }
    }

    /// Go-style plain else: `alternative` field holding a block.
    fn field_else_bonus(&mut self, node: Node<'_>) {
        if !node.kind().starts_with("if") {
            return;
        }
        if node
            .child_by_field_name("alternative")
            .is_some_and(|a| a.kind() == "block")
        {
            self.score += 1;
        }
    }

    fn is_else_if(&self, node: Node<'_>) -> bool {
        if !node.kind().starts_with("if") {
            return false;
        }
        let Some(parent) = node.parent() else {
            return false;
        };
        if self.spec.coc_flat_kinds.contains(&parent.kind()) {
            return true; // TS/Rust/Python: if directly under an else clause
        }
        parent.kind().starts_with("if")
            && parent
                .child_by_field_name("alternative")
                .is_some_and(|a| a.id() == node.id()) // Go: if as alternative
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

fn has_if_child(node: Node<'_>) -> bool {
    ast::children(node)
        .iter()
        .any(|c| c.kind().starts_with("if"))
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
fn collect_in_order<'s>(node: Node<'_>, src: &'s [u8], spec: &LangSpec, out: &mut Vec<&'s str>) {
    let Some(op) = logic_op(node, src, spec) else {
        return;
    };
    if let Some(left) = node.child_by_field_name("left") {
        collect_in_order(left, src, spec, out);
    }
    out.push(op);
    if let Some(right) = node.child_by_field_name("right") {
        collect_in_order(right, src, spec, out);
    }
}

fn logic_op<'s>(node: Node<'_>, src: &'s [u8], spec: &LangSpec) -> Option<&'s str> {
    let op = operator_text(node, src)?;
    spec.coc_operators.contains(&op).then_some(op)
}
