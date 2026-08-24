//! The one function-subtree walk the per-metric visitors share.
//! Split to its own leaf in the headroom sprint: cyclo.rs importing
//! it THROUGH the metrics hub made the module pair a cycle the
//! graph axis itself billed on the self-scan.

use crate::scan::spec::LangSpec;
use tree_sitter::Node;

/// Pre-order nodes of a function subtree, excluding nested standalone
/// function units (they are measured as their own units; the shared
/// is_unit_node predicate keeps this in lockstep with extraction).
pub fn own_nodes<'t>(fn_node: Node<'t>, spec: &LangSpec) -> Vec<Node<'t>> {
    let mut out = Vec::new();
    let mut stack = vec![fn_node];
    while let Some(node) = stack.pop() {
        let nested_standalone =
            node.id() != fn_node.id() && crate::scan::functions::is_unit_node(node, spec);
        if nested_standalone {
            continue;
        }
        out.push(node);
        stack.extend(crate::scan::ast::children(node).into_iter().rev());
    }
    out
}
