//! Cyclomatic complexity: base 1, +1 per decision point (gocyclo
//! definition: if / loops / case / short-circuit operators). Alignment
//! targets: lizard (TS/Py), rust-code-analysis (Rust), gocyclo (Go).

use super::own_nodes;
use crate::scan::ast::{self, operator_text};
use crate::scan::spec::LangSpec;
use tree_sitter::Node;

pub fn measure(fn_node: Node<'_>, src: &[u8], spec: &LangSpec) -> u32 {
    let mut cc: u32 = 1;
    for node in own_nodes(fn_node, spec) {
        if spec.cc_kinds.contains(&node.kind()) {
            cc += 1;
        } else if spec.chain_kinds.contains(&node.kind()) {
            // N children joined by N-1 anonymous short-circuit tokens.
            cc += ast::named_children(node).len().saturating_sub(1) as u32;
        } else if let Some(op) = operator_text(node, src)
            && spec.cc_operators.contains(&op)
        {
            cc += 1;
        }
    }
    cc
}
