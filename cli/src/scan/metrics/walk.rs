//! The one function-subtree walk the per-metric visitors share.
//! Split to its own leaf in the headroom sprint: cyclo.rs importing
//! it THROUGH the metrics hub made the module pair a cycle the
//! graph axis itself billed on the self-scan.

use crate::scan::spec::LangSpec;
use tree_sitter::Node;

/// Pre-order nodes of a function subtree, excluding nested standalone
/// function units (they are measured as their own units; the shared
/// is_unit_node predicate keeps this in lockstep with extraction) and
/// the compile-time text under an opaque field.
pub fn own_nodes<'t>(fn_node: Node<'t>, src: &[u8], spec: &LangSpec) -> Vec<Node<'t>> {
    crate::scan::ast::preorder(
        fn_node,
        |node| !crate::scan::functions::is_unit_node(node, src, spec),
        |node| measured(node, spec),
    )
}

/// A node's children minus those reached through one of the spec's
/// opaque fields (a `#if` condition): the parser types them as
/// expressions, the metrics read them as text. The walk's one throat
/// for children, so the cognitive walker and own_nodes cannot differ
/// on what a function contains.
pub fn measured<'t>(node: Node<'t>, spec: &LangSpec) -> Vec<Node<'t>> {
    let opaque: Vec<&str> = spec
        .opaque_fields
        .iter()
        .filter(|(kind, _)| *kind == node.kind())
        .map(|(_, field)| *field)
        .collect();
    if opaque.is_empty() {
        return crate::scan::ast::children(node);
    }
    (0..node.child_count())
        .filter(|&i| {
            !node
                .field_name_for_child(i)
                .is_some_and(|f| opaque.contains(&f))
        })
        .filter_map(|i| node.child(i))
        .collect()
}
