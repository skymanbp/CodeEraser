//! Haskell: `Ffi`. `foreign export ccall hsAdd :: CInt -> CInt` is
//! its own top-level declaration whose `signature` names the binding
//! it exports; the bit lands on THAT binding (X-16), so the file's
//! exported names are gathered once (`foreign_exports`, read by
//! conv::file_facts before the units are walked) and each unit is
//! looked up by its own name. The C-side entity string (`"hs_mul"`)
//! is the foreign spelling and plays no part.

use super::{Conv, text};
use crate::scan::ast;
use std::collections::BTreeSet;
use tree_sitter::Node;

pub(super) fn foreign_exports(root: Node<'_>, src: &[u8]) -> BTreeSet<String> {
    root.child_by_field_name("declarations")
        .into_iter()
        .flat_map(ast::named_children)
        .filter(|d| d.kind() == "foreign_export")
        .filter_map(|d| {
            d.child_by_field_name("signature")?
                .child_by_field_name("name")
        })
        .map(|n| text(n, src).to_string())
        .collect()
}

pub(super) fn bits(node: Node<'_>, src: &[u8], exported: &BTreeSet<String>) -> i64 {
    match node.child_by_field_name("name") {
        Some(n) if exported.contains(text(n, src)) => Conv::Ffi.bit(),
        _ => 0,
    }
}
