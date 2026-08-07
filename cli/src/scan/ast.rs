//! Tiny AST helpers shared by scan modules. tree-sitter 0.26 mixes
//! usize (`child_count`) with u32 (`child(i)`) — centralize the cast
//! here instead of sprinkling `as u32` through every walker.

use tree_sitter::Node;

pub fn children<'t>(node: Node<'t>) -> Vec<Node<'t>> {
    // cast is safe: a single file cannot hold > u32::MAX AST children
    (0..node.child_count())
        .filter_map(|i| node.child(i as u32))
        .collect()
}

pub fn named_children<'t>(node: Node<'t>) -> Vec<Node<'t>> {
    (0..node.named_child_count())
        .filter_map(|i| node.named_child(i as u32))
        .collect()
}

/// Text of a node's `operator` field, if any (uniform across grammars:
/// python boolean_operator, ts/rust/go binary_expression all expose it).
pub fn operator_text<'s>(node: Node<'_>, src: &'s [u8]) -> Option<&'s str> {
    let op = node.child_by_field_name("operator")?;
    op.utf8_text(src).ok()
}
