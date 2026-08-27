//! Rust: attributes are read where rustc reads them — the item's own
//! preceding `attribute_item` siblings (a doc comment may sit between
//! attribute and item and is stepped over, R9), the `inner_attribute_item`
//! rows of the item's own `declaration_list` body (a `mod`'s `#![cfg(test)]`
//! governs the `mod`) and of every enclosing body — the file, a
//! `declaration_list`, a function's `block` (whose `#![…]` governs
//! what is inside it, never the function itself) — and the attributes
//! of every ancestor item, because `cfg`, lint levels and the export
//! attributes on an `impl` all reach the items inside.
//!
//! One walk over that attribute set yields three facts:
//!   - `Test` / `Cfg` from ONE predicate-tree reading: a `cfg` whose
//!     tree holds the atom `test` is `Test` (`all(test, …)`,
//!     `any(test, …)`, `not(test)` and an inherited `#[cfg(test)] mod`
//!     alike); a tree holding other atoms and never `test` is `Cfg`
//!     (`target_os = "…"` is the atom `target_os`). Disjoint by
//!     construction — `Cfg` is the "other atoms, no test" remainder.
//!   - `Ffi`: an attribute whose path's LAST segment is in the export
//!     table (`#[pyo3::pyfunction]` matches `pyfunction`),
//!     `proc_macro*`, `unsafe(…)` wrapping one of them, `doc(hidden)`,
//!     or an `extern "…"` modifier on the function itself.
//!   - `Allow`: `allow(dead_code)` / `expect(dead_code)`.

use super::{Conv, text};
use crate::fourclass::visibility::ancestors;
use crate::scan::ast;
use tree_sitter::Node;

/// Attribute paths (last segment) that hand a declaration to a
/// foreign caller — the linker, a language binding, a macro host.
const FFI_ATTRS: [&str; 11] = [
    "no_mangle",
    "export_name",
    "used",
    "link_section",
    "wasm_bindgen",
    "pyfunction",
    "pymodule",
    "napi",
    "ctor",
    "global_allocator",
    "panic_handler",
];

pub(super) fn bits(node: Node<'_>, src: &[u8]) -> i64 {
    let mut word = 0;
    let mut cfg = Predicate::default();
    for attr in attributes(node) {
        let Some(path) = ast::named_children(attr).into_iter().next() else {
            continue;
        };
        let args = attr.child_by_field_name("arguments");
        let last = text(path, src).rsplit("::").next().unwrap_or("");
        match last {
            "cfg" => args.into_iter().for_each(|t| cfg.read(t, src)),
            "allow" | "expect" if names(args, src, "dead_code") => word |= Conv::Allow.bit(),
            "doc" if names(args, src, "hidden") => word |= Conv::Ffi.bit(),
            "unsafe" if args.is_some_and(|t| idents(t).any(|i| ffi_name(text(i, src)))) => {
                word |= Conv::Ffi.bit();
            }
            seg if ffi_name(seg) => word |= Conv::Ffi.bit(),
            _ => {}
        }
    }
    if extern_fn(node) {
        word |= Conv::Ffi.bit();
    }
    word | cfg.bits()
}

fn ffi_name(seg: &str) -> bool {
    FFI_ATTRS.contains(&seg) || seg.starts_with("proc_macro")
}

/// Whether the argument token tree holds the identifier `ident`.
fn names(args: Option<Node<'_>>, src: &[u8], ident: &str) -> bool {
    args.is_some_and(|t| idents(t).any(|i| text(i, src) == ident))
}

/// Every identifier at the top of one token tree.
fn idents(tree: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    ast::named_children(tree)
        .into_iter()
        .filter(|c| c.kind() == "identifier")
}

fn extern_fn(node: Node<'_>) -> bool {
    node.kind() == "function_item"
        && ast::named_children(node)
            .into_iter()
            .filter(|c| c.kind() == "function_modifiers")
            .flat_map(ast::named_children)
            .any(|m| m.kind() == "extern_modifier")
}

/// The `attribute` nodes that govern `node`: its own outer ones, its
/// own body's inner ones, and the same for every ancestor.
fn attributes(node: Node<'_>) -> Vec<Node<'_>> {
    std::iter::once(node)
        .chain(ancestors(node))
        .flat_map(|n| outer(n).chain(inner(n)))
        .collect()
}

/// The `attribute_item` run right above `n`, comments stepped over.
fn outer(n: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    let mut out = Vec::new();
    let mut prev = n.prev_sibling();
    while let Some(p) = prev {
        match p.kind() {
            "attribute_item" => out.extend(ast::named_children(p)),
            "line_comment" | "block_comment" => {}
            _ => break,
        }
        prev = p.prev_sibling();
    }
    out.into_iter()
}

/// The `inner_attribute_item` rows `n` carries: of `n` itself when it
/// is a body (the file, a `declaration_list`, a `block`), and of its
/// own `declaration_list` body — a `block` body is not the item's own
/// (`fn f() { #![allow(dead_code)] }` governs the block, not `f`).
fn inner(n: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    let own_body = n
        .child_by_field_name("body")
        .filter(|b| b.kind() == "declaration_list");
    [n].into_iter()
        .chain(own_body)
        .filter(|c| matches!(c.kind(), "source_file" | "declaration_list" | "block"))
        .flat_map(ast::named_children)
        .filter(|i| i.kind() == "inner_attribute_item")
        .flat_map(ast::named_children)
}

/// The two facts one `cfg` predicate tree yields, accumulated over
/// every `cfg` that governs the declaration.
#[derive(Default)]
struct Predicate {
    test: bool,
    other: bool,
}

impl Predicate {
    /// Atoms are the identifiers not followed by a token tree (those
    /// are the combinators `all`/`any`/`not`); a `key = "value"` pair
    /// is the atom `key`.
    fn read(&mut self, tree: Node<'_>, src: &[u8]) {
        for c in ast::named_children(tree) {
            match c.kind() {
                "token_tree" => self.read(c, src),
                "identifier" if c.next_sibling().is_some_and(|n| n.kind() == "token_tree") => {}
                "identifier" if text(c, src) == "test" => self.test = true,
                "identifier" => self.other = true,
                _ => {}
            }
        }
    }

    fn bits(&self) -> i64 {
        match (self.test, self.other) {
            (true, _) => Conv::Test.bit(),
            (false, true) => Conv::Cfg.bit(),
            (false, false) => 0,
        }
    }
}
