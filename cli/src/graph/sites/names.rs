//! The names a site's statement imports (proto 9.1.0): Python's `from M
//! import a, b as c` names `a` and `b` — each may be a submodule
//! CPython imports, which the core's `also` rows reach
//! (CE.Resolve.PyAlso). Which children hold the names is the package's
//! site table (`names = {field, inner}`, graph/spec.rs Names); this
//! reader only follows it: every child under the field, by its own inner
//! field when it has one (an alias keeps its target there), else by its
//! whole text. A row without a names source reads none.

use crate::graph::spec::Names;

pub(super) fn read(node: tree_sitter::Node, src: &[u8], names: Option<&Names>) -> Vec<String> {
    let Some(names) = names else {
        return Vec::new();
    };
    let mut cursor = node.walk();
    node.children_by_field_name(names.field, &mut cursor)
        .filter_map(|child| {
            let at = child.child_by_field_name(names.inner).unwrap_or(child);
            super::node_text(at, src)
        })
        .filter(|name| !name.is_empty())
        .collect()
}
