//! Size metrics: total lines and comment lines (whole file).
//! Function length / params live on FnUnit (functions.rs).

use crate::scan::spec::LangSpec;
use tree_sitter::Node;

pub fn total_lines(src: &[u8]) -> usize {
    if src.is_empty() {
        return 0;
    }
    let newlines = src.iter().filter(|b| **b == b'\n').count();
    if src.ends_with(b"\n") {
        newlines
    } else {
        newlines + 1
    }
}

/// Lines covered by comment nodes (a line with any comment counts once).
pub fn comment_lines(root: Node<'_>, spec: &LangSpec) -> usize {
    let mut lines = std::collections::HashSet::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if spec.comment_kinds.contains(&node.kind()) {
            for row in node.start_position().row..=node.end_position().row {
                lines.insert(row);
            }
        }
        stack.extend(crate::scan::ast::children(node).into_iter().rev());
    }
    lines.len()
}
