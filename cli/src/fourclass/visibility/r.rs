//! R visibility (plan v2.30 step 4, booklet §7, register D16), read in
//! the declaring file alone. A file that documents with roxygen (`#'`
//! comments) exports exactly what its blocks tag `@export` — roxygen
//! writes the package's NAMESPACE from those tags, `@exportS3Method`
//! included — and a file without roxygen falls back to R's own
//! convention, which hides a name starting with a dot (`ls()` does not
//! list it). NAMESPACE itself is another file and is never read (the
//! one-file rule of visibility/mod.rs), so an export written there by
//! hand reads as private in a roxygen file: the one shape this reading
//! misses. An anonymous function value names nothing to export. Bit 1:
//! no function encloses the declaration. Bit 2 never.

use super::{ancestors, word};
use crate::scan::ast;
use crate::scan::binding;
use tree_sitter::Node;

pub(super) fn bits(node: Node<'_>, src: &[u8]) -> i64 {
    let Some(b) = binding::of(node, src) else {
        return 0;
    };
    let exported = if roxygen_file(node, src) {
        tagged_export(b.statement, src)
    } else {
        binding::spelled(b.name, src).is_some_and(|n| !n.starts_with('.'))
    };
    let enclosed = ancestors(node).any(|a| a.kind() == "function_definition");
    word(exported, !enclosed)
}

/// Whether the file documents with roxygen: a `#'` comment among the
/// top-level nodes, where roxygen blocks sit.
fn roxygen_file(node: Node<'_>, src: &[u8]) -> bool {
    let root = ancestors(node).last().unwrap_or(node);
    ast::children(root)
        .into_iter()
        .any(|c| roxygen_line(c, src).is_some())
}

/// Whether the roxygen block documenting `statement` — the `#'` lines
/// among the comments right above it; roxygen skips blank lines and
/// plain comments on its way to the next expression — carries an
/// `@export` tag.
fn tagged_export(statement: Node<'_>, src: &[u8]) -> bool {
    let mut prev = statement.prev_sibling();
    while let Some(c) = prev.filter(|c| c.kind() == "comment") {
        if roxygen_line(c, src).is_some_and(|l| l.trim_start().starts_with("@export")) {
            return true;
        }
        prev = c.prev_sibling();
    }
    false
}

/// A roxygen comment's text past its `#'` marker.
fn roxygen_line<'s>(node: Node<'_>, src: &'s [u8]) -> Option<&'s str> {
    if node.kind() != "comment" {
        return None;
    }
    node.utf8_text(src).ok()?.strip_prefix("#'")
}

#[cfg(test)]
#[path = "../../../tests/unit/fourclass/visibility/tests_r.rs"]
mod tests;
