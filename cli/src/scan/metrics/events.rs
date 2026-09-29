//! The structural event stream of one function (plan v2.30 step 7b
//! ③): every node the LangSpec tables classify, in pre-order, with
//! the facts the complexity rules read — its class bits, its nearest
//! classified ancestor, whether that ancestor is its tree parent,
//! whether it sits in the ancestor's body or header, and for a
//! boolean chain the operator tokens in source order. The rules
//! themselves — the whitepaper's increments and nesting penalty,
//! cyclomatic's decision points, the nesting depth — live in the core
//! (`CE.Scan.Complexity`); this side states the tree and never adds a
//! number to another. Parsing does not move (hard constraint 1): the
//! table lookups here ARE the language knowledge, and the core reads
//! only bits.
//!
//! A node is emitted when any CLASS bit holds (nesting, flat, nest-
//! only, labelled jump, if kind, chain, cyclomatic kind or operator,
//! logic root); everything else is transparent and its children
//! inherit its position under the same ancestor — which is exactly
//! how the level passed through an unclassified block in the walker
//! this module replaced. Nested standalone units and anonymous tokens
//! are not descended (the walk::measured / is_unit_node throats, so a
//! unit's events and its unit set can never disagree), and children
//! under an opaque field never appear — the one asymmetry with the
//! old walker's direct-child probe, which read raw children, is moot
//! because no flat kind carries an opaque field.
//!
//! A flat clause's exemption ("`else if` yields to the inner if") and
//! an if's else-if position are the core's to derive from the DIRECT
//! and IN_ALT bits; the class of an if's first `alternative` child
//! (none / if / flat / other) is stated here because a plain-else body
//! is often a node no class bit names (a Go block, a Java `return`),
//! and the core could not see it otherwise.

use super::vocab::{
    ALT_SHIFT, CC_KIND, CC_OP, CHAIN, DIRECT, Event, FLAT, IF_KIND, IN_ALT, LABELLED_JUMP,
    LOGIC_ROOT, NEST_ONLY, NESTING,
};
use crate::scan::ast::{self, operator_text};
use crate::scan::spec::{Kinds, LangSpec};
use tree_sitter::Node;

pub fn emit(fn_node: Node<'_>, src: &[u8], spec: &LangSpec) -> Vec<Event> {
    let mut e = Emitter {
        src,
        spec,
        out: Vec::new(),
    };
    let root = Frame {
        seq: None,
        node: fn_node.id(),
        split: None,
    };
    for child in super::walk::measured(fn_node, spec) {
        e.visit(child, &root, 0);
    }
    e.out
}

/// The nearest emitted ancestor as its children see it: its seq, its
/// node id (the DIRECT test) and, for a structure that splits its
/// children, the body positions its entry names — the spot names and
/// the ids of the children those fields hold.
struct Frame {
    seq: Option<u32>,
    node: usize,
    split: Option<(Vec<&'static str>, Vec<usize>)>,
}

/// A child's position under its frame: body (1) when the frame's
/// entry names no position at all (the ternaries nest whole), or when
/// a named field holds the child, or when the child's kind is named;
/// header (0) otherwise, and 0 under a non-splitting frame.
fn child_pos(frame: &Frame, child: Node<'_>) -> u8 {
    match &frame.split {
        None => 0,
        Some((spots, fielded)) => u8::from(
            spots.is_empty() || fielded.contains(&child.id()) || spots.contains(&child.kind()),
        ),
    }
}

struct Emitter<'s, 'sp> {
    src: &'s [u8],
    spec: &'sp LangSpec,
    out: Vec<Event>,
}

impl Emitter<'_, '_> {
    fn visit(&mut self, node: Node<'_>, frame: &Frame, pos: u8) {
        // anon tokens are leaves that can share a structure's kind
        // name (Haskell's `case`); a nested unit is measured on its own
        if !node.is_named() || crate::scan::functions::is_unit_node(node, self.src, self.spec) {
            return;
        }
        let class = self.class_bits(node);
        if class == 0 {
            for child in super::walk::measured(node, self.spec) {
                self.visit(child, frame, pos);
            }
            return;
        }
        let ops = if self.is_logic_root(node) {
            self.in_order(node)
        } else {
            Vec::new()
        };
        let seq = self.out.len() as u32;
        self.out.push(Event {
            seq,
            parent: frame.seq,
            pos,
            flags: class | relation_bits(node, frame) | self.alt_class(node, class),
            aux: if class & CHAIN != 0 {
                ast::named_children(node).len() as u32
            } else {
                0
            },
            ops,
        });
        let inner = Frame {
            seq: Some(seq),
            node: node.id(),
            split: self.split_of(node, class),
        };
        for child in super::walk::measured(node, self.spec) {
            self.visit(child, &inner, child_pos(&inner, child));
        }
    }

    /// The table memberships of one node — the class bits, and the
    /// logic-root bit when the node heads a maximal boolean chain.
    fn class_bits(&self, node: Node<'_>) -> u16 {
        let sp = self.spec;
        let kind = node.kind();
        let cc_op = operator_text(node, self.src).is_some_and(|op| sp.cc_operators.contains(&op));
        let facts = [
            (entry_of(sp.coc_nesting_kinds, kind).is_some(), NESTING),
            (entry_of(sp.coc_flat_kinds, kind).is_some(), FLAT),
            (sp.coc_nest_only_kinds.contains(&kind), NEST_ONLY),
            (
                sp.coc_jump_kinds.contains(&kind) && has_child_of(node, sp.label_kinds),
                LABELLED_JUMP,
            ),
            (sp.if_kinds.contains(&kind), IF_KIND),
            (sp.chain_kinds.contains(&kind), CHAIN),
            (sp.cc_kinds.contains(&kind), CC_KIND),
            (cc_op, CC_OP),
            (self.is_logic_root(node), LOGIC_ROOT),
        ];
        facts
            .into_iter()
            .filter_map(|(holds, bit)| holds.then_some(bit))
            .fold(0, |acc, bit| acc | bit)
    }

    /// The class of an if's first `alternative` child (bits 11–12);
    /// 0 on anything that is no if kind.
    fn alt_class(&self, node: Node<'_>, class: u16) -> u16 {
        if class & IF_KIND == 0 {
            return 0;
        }
        let alt = match node.child_by_field_name("alternative") {
            None => 0,
            Some(a) if self.spec.if_kinds.contains(&a.kind()) => 1,
            Some(a) if entry_of(self.spec.coc_flat_kinds, a.kind()).is_some() => 2,
            Some(_) => 3,
        };
        alt << ALT_SHIFT
    }

    /// The body positions a structure's children are read against:
    /// its nesting entry's, else its flat entry's (the walker's own
    /// order of the two tables); none for every other class.
    fn split_of(&self, node: Node<'_>, class: u16) -> Option<(Vec<&'static str>, Vec<usize>)> {
        let entry = if class & NESTING != 0 {
            entry_of(self.spec.coc_nesting_kinds, node.kind())
        } else if class & FLAT != 0 {
            entry_of(self.spec.coc_flat_kinds, node.kind())
        } else {
            None
        }?;
        let spots: Vec<&'static str> = entry.split(' ').skip(1).collect();
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
        Some((spots, fielded))
    }

    /// Root of a maximal boolean chain: a short-circuit node whose
    /// parent is not one.
    fn is_logic_root(&self, node: Node<'_>) -> bool {
        self.logic_op(node).is_some() && node.parent().is_none_or(|p| self.logic_op(p).is_none())
    }

    fn logic_op(&self, node: Node<'_>) -> Option<u32> {
        let op = operator_text(node, self.src)?;
        self.spec
            .coc_operators
            .iter()
            .position(|&o| o == op)
            .map(|i| i as u32)
    }

    /// The chain's operator tokens in source order — in-order (left,
    /// self, right), so a run count over them reads the source's
    /// runs. Operand fields differ by grammar family: python/ts/rust/
    /// go/lua expose `left`/`right`, tree-sitter-haskell
    /// `left_operand` / `right_operand`, tree-sitter-r `lhs`/`rhs` —
    /// one alias lookup instead of a per-lang walker.
    fn in_order(&self, node: Node<'_>) -> Vec<u32> {
        let mut out = Vec::new();
        self.collect_in_order(node, &mut out);
        out
    }

    fn collect_in_order(&self, node: Node<'_>, out: &mut Vec<u32>) {
        let Some(op) = self.logic_op(node) else {
            return;
        };
        let operand = |fields: [&str; 3]| fields.iter().find_map(|f| node.child_by_field_name(f));
        if let Some(left) = operand(["left", "left_operand", "lhs"]) {
            self.collect_in_order(left, out);
        }
        out.push(op);
        if let Some(right) = operand(["right", "right_operand", "rhs"]) {
            self.collect_in_order(right, out);
        }
    }
}

/// The two relation bits: whether the tree parent is the frame's
/// node, and whether the node is that parent's first `alternative`
/// child (Go and Java hang the next if or the else body there).
fn relation_bits(node: Node<'_>, frame: &Frame) -> u16 {
    let Some(parent) = node.parent() else {
        return 0;
    };
    let direct = if parent.id() == frame.node { DIRECT } else { 0 };
    let in_alt = parent
        .child_by_field_name("alternative")
        .is_some_and(|a| a.id() == node.id());
    direct | if in_alt { IN_ALT } else { 0 }
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
/// label.
fn has_child_of(node: Node<'_>, kinds: &[&str]) -> bool {
    ast::children(node)
        .iter()
        .any(|c| kinds.contains(&c.kind()))
}

#[cfg(test)]
#[path = "../../../tests/unit/scan/metrics/events.rs"]
mod tests;
