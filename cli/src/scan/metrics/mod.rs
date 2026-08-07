//! Metric data model + shared AST walking helpers.

pub mod cognitive;
pub mod cyclo;
pub mod size;

use super::spec::LangSpec;
use serde::Serialize;
use tree_sitter::Node;

#[derive(Debug, Serialize)]
pub struct FnMetrics {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub lines: usize,
    pub params: usize,
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub max_nesting: u32,
}

#[derive(Debug, Serialize)]
pub struct FileMetrics {
    pub path: String,
    pub lang: &'static str,
    pub total_lines: usize,
    pub comment_lines: usize,
    pub functions: Vec<FnMetrics>,
}

/// Pre-order nodes of a function subtree, excluding nested standalone
/// function units (they are measured as their own units).
pub fn own_nodes<'t>(fn_node: Node<'t>, spec: &LangSpec) -> Vec<Node<'t>> {
    let mut out = Vec::new();
    let mut stack = vec![fn_node];
    while let Some(node) = stack.pop() {
        let nested_standalone = node.id() != fn_node.id() && spec.fn_kinds.contains(&node.kind());
        if nested_standalone {
            continue;
        }
        out.push(node);
        stack.extend(crate::scan::ast::children(node).into_iter().rev());
    }
    out
}
