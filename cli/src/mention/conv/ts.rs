//! TS/TSX: three facts, each read off the tree the grammar actually
//! builds (probed on tree-sitter-typescript 0.23.2):
//!   - `Registration`: a decorator is a `decorator` child of the
//!     declaration (`@dec class A`, `export @dec class B`) or of its
//!     `export_statement` parent (`@dec export class C`) — the two
//!     spellings judge alike (L3-F7a), and a comment between decorator
//!     and name is just another child, never a hop.
//!   - `DefaultExport`: a NAMED function or class declaration whose
//!     `export_statement` carries the `default` keyword; an anonymous
//!     default export has no name to be mentioned by and is out of the
//!     domain already (`(anonymous)`).
//!   - `Ambient`: any `ambient_declaration` ancestor — the criterion's
//!     L5-F15 ruling that the container's shape is never the test,
//!     only the ancestor's presence (`declare class C {}` has no
//!     container at all).

use super::{Conv, parent_of_kind, under};
use crate::scan::ast;
use tree_sitter::Node;

/// Declaration kinds `export default` can name.
const DEFAULTABLE: [&str; 3] = [
    "function_declaration",
    "generator_function_declaration",
    "class_declaration",
];

pub(super) fn bits(node: Node<'_>) -> i64 {
    let mut word = 0;
    let export = parent_of_kind(node, "export_statement");
    if has_child(node, "decorator") || export.is_some_and(|e| has_child(e, "decorator")) {
        word |= Conv::Registration.bit();
    }
    if DEFAULTABLE.contains(&node.kind())
        && node.child_by_field_name("name").is_some()
        && export.is_some_and(|e| has_child(e, "default"))
    {
        word |= Conv::DefaultExport.bit();
    }
    if under(node, "ambient_declaration") {
        word |= Conv::Ambient.bit();
    }
    word
}

/// Anonymous leaves (`default`) and named ones (`decorator`) alike.
fn has_child(node: Node<'_>, kind: &str) -> bool {
    ast::children(node).iter().any(|c| c.kind() == kind)
}
