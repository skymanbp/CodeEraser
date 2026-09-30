//! Positions resolved against a tree (plan v2.31 step 4 A2; the syntax
//! is pos.rs's). A position gives named nodes only, comments aside;
//! `*` gives the children no other position of the same row names at
//! the same node, so the caller hands the row's other positions in.

use super::pos::{Step, paths};
use tree_sitter::Node;

/// The named, non-comment children of a node.
pub fn kids<'t>(node: Node<'t>, comments: &[&str]) -> Vec<Node<'t>> {
    crate::scan::ast::named_children(node)
        .into_iter()
        .filter(|c| !comments.contains(&c.kind()))
        .collect()
}

/// The nodes at `position` under `node`; `others` are the row's other
/// positions, read only by `*`.
pub fn at<'t>(node: Node<'t>, position: &str, others: &[&str], comments: &[&str]) -> Vec<Node<'t>> {
    match position {
        "" => Vec::new(),
        "." => vec![node],
        "*" => star(node, others, comments),
        _ => paths(position)
            .into_iter()
            .map(|path| walk(node, &path, comments))
            .find(|found| !found.is_empty())
            .unwrap_or_default(),
    }
}

fn walk<'t>(node: Node<'t>, path: &[Step<'_>], comments: &[&str]) -> Vec<Node<'t>> {
    let mut here = vec![node];
    for step in path {
        here = here
            .into_iter()
            .flat_map(|n| step_at(n, *step, comments))
            .collect();
    }
    here
}

fn step_at<'t>(node: Node<'t>, step: Step<'_>, comments: &[&str]) -> Vec<Node<'t>> {
    match step {
        Step::Field(field) => {
            let mut cursor = node.walk();
            node.children_by_field_name(field, &mut cursor)
                .filter(|c| c.is_named() && !comments.contains(&c.kind()))
                .collect()
        }
        Step::Kind(kind) => kids(node, comments)
            .into_iter()
            .filter(|c| c.kind() == kind)
            .collect(),
    }
}

/// `*`: the children the first step of no other position gives.
fn star<'t>(node: Node<'t>, others: &[&str], comments: &[&str]) -> Vec<Node<'t>> {
    let named: Vec<usize> = others
        .iter()
        .filter(|p| !matches!(**p, "" | "*" | "."))
        .flat_map(|p| paths(p))
        .filter_map(|path| path.first().copied())
        .flat_map(|first| step_at(node, first, comments))
        .map(|n| n.id())
        .collect();
    kids(node, comments)
        .into_iter()
        .filter(|c| !named.contains(&c.id()))
        .collect()
}

/// Whether a node holds this token as a direct child (anonymous or
/// named: an operator, a storage word, a marker).
pub fn holds(node: Node<'_>, token: &str) -> bool {
    crate::scan::ast::children(node)
        .iter()
        .any(|c| c.kind() == token)
}

/// The node's first source line, trimmed, at most 80 characters.
pub fn first_line(node: Node<'_>, src: &[u8]) -> String {
    let text = node.utf8_text(src).unwrap_or("");
    text.lines()
        .next()
        .unwrap_or("")
        .trim()
        .chars()
        .take(80)
        .collect()
}

/// 1-based (line, column) of a node's start.
pub fn place(node: Node<'_>) -> (u32, u32) {
    let p = node.start_position();
    (p.row as u32 + 1, p.column as u32 + 1)
}
