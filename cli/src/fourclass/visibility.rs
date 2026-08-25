//! Symbol visibility as a LOCAL SYNTACTIC FACT (plan v2.14, ADR-008
//! clause three). Exported-ness is the public/private judgment axis
//! the graph family has always declared and never had a producer for:
//! `symbols.flags` has been reserved-and-zero since 2g because
//! "storing a guess would be inventing entry-point facts"
//! (graph/store.rs). Nothing here guesses — every bit is read off the
//! declaration itself, in the file that declares it, with no
//! resolution and no cross-file lookup.
//!
//! Boundaries, stated rather than papered over:
//!   - Python's `__all__` is not consulted (v1): a module that
//!     re-exports under a different name is judged by its
//!     declarations, so `__all__`-only exports read as private. The
//!     bit is a floor, never an invention.
//!   - Haskell's export list is read from the module header when one
//!     is present; a header without a list exports everything, which
//!     is the language's own rule, not an assumption.
//!   - Nested items (a `pub fn` inside a private `mod`) carry their
//!     OWN visibility. Reachability of the enclosing scope is a
//!     resolution question and belongs to the graph, not here.

use crate::scan::ast;
use crate::scan::lang::Lang;
use tree_sitter::Node;

/// Frozen bit positions for `symbols.flags`. Bit 0 is the one the
/// graph's verdict codes have always meant by "public"; the rest of
/// the word stays free for later facts.
pub const VIS_EXPORTED: i64 = 1 << 0;

/// The visibility bits of one declaration node.
pub fn bits(node: Node<'_>, src: &[u8], lang: Lang) -> i64 {
    if exported(node, src, lang) {
        VIS_EXPORTED
    } else {
        0
    }
}

/// Markdown sections: every heading is an anchor any document may
/// link to, so a section is public by construction — there is no
/// private heading to distinguish it from.
pub const MARKDOWN_VIS: i64 = VIS_EXPORTED;

fn exported(node: Node<'_>, src: &[u8], lang: Lang) -> bool {
    match lang {
        Lang::Rust => has_child_kind(node, "visibility_modifier"),
        Lang::TypeScript | Lang::Tsx => in_export_statement(node),
        Lang::Python => python_public(node, src),
        Lang::Go => go_exported(node, src),
        Lang::Haskell => haskell_exported(node, src),
        _ => false,
    }
}

fn has_child_kind(node: Node<'_>, kind: &str) -> bool {
    ast::children(node).iter().any(|c| c.kind() == kind)
}

/// TS/TSX: `export` wraps the declaration, so the fact is one hop up
/// (`export_statement`) — or two when a decorator sits between.
fn in_export_statement(node: Node<'_>) -> bool {
    let mut cur = node.parent();
    for _ in 0..2 {
        match cur {
            Some(p) if p.kind() == "export_statement" => return true,
            Some(p) => cur = p.parent(),
            None => return false,
        }
    }
    false
}

/// Python: the convention IS the language's visibility rule — a
/// leading underscore marks a name as internal. Dunder names
/// (`__init__`) are protocol, not private.
fn python_public(node: Node<'_>, src: &[u8]) -> bool {
    match name_text(node, src) {
        Some(name) => !name.starts_with('_') || name.starts_with("__") && name.ends_with("__"),
        None => false,
    }
}

/// Go: an identifier's first rune decides. Upper case is exported —
/// the whole rule, checked on the declaration's own name.
fn go_exported(node: Node<'_>, src: &[u8]) -> bool {
    match name_text(node, src) {
        Some(name) => name.chars().next().is_some_and(char::is_uppercase),
        None => false,
    }
}

/// Haskell: a `module M (a, b) where` header exports exactly that
/// list; a header without a list exports everything. The list is read
/// from the file's own header — no import is followed.
fn haskell_exported(node: Node<'_>, src: &[u8]) -> bool {
    let Some(name) = name_text(node, src) else {
        return false;
    };
    let Some(root) = root_of(node) else {
        return false;
    };
    match export_list(root, src) {
        None => true,
        Some(list) => list.split(',').any(|item| {
            let t = item.trim().trim_start_matches('(').trim_end_matches(')');
            t == name || t.split('(').next().map(str::trim) == Some(name.as_str())
        }),
    }
}

/// The module header's export list text, or None when the header
/// declares none (which in Haskell means "export everything").
fn export_list(root: Node<'_>, src: &[u8]) -> Option<String> {
    let header = ast::named_children(root)
        .into_iter()
        .find(|c| c.kind() == "header" || c.kind() == "module")?;
    let exports = ast::named_children(header)
        .into_iter()
        .find(|c| c.kind() == "exports" || c.kind() == "export_list")?;
    Some(String::from_utf8_lossy(&src[exports.byte_range()]).into_owned())
}

fn root_of(node: Node<'_>) -> Option<Node<'_>> {
    let mut cur = node;
    while let Some(p) = cur.parent() {
        cur = p;
    }
    Some(cur)
}

fn name_text(node: Node<'_>, src: &[u8]) -> Option<String> {
    let name = node.child_by_field_name("name")?;
    Some(String::from_utf8_lossy(&src[name.byte_range()]).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::functions;
    use crate::scan::spec;

    /// (unit name, exported?) for every function-shaped declaration —
    /// the extractor's own root, so the test measures what the
    /// producer will actually see.
    fn fns(src: &str, lang: Lang) -> Vec<(String, bool)> {
        let tree = ast::parse_lang(src, lang).expect("parses");
        let bytes = src.as_bytes();
        functions::extract(tree.root_node(), bytes, spec::spec(lang))
            .into_iter()
            .map(|f| {
                (
                    f.name.clone(),
                    bits(f.node, bytes, lang) & VIS_EXPORTED != 0,
                )
            })
            .collect()
    }

    /// One language's rule as a row: the language, a source sample,
    /// and the (name, exported) pairs it must produce. Named because
    /// the tuple is the case grammar — spelling it inline is the
    /// "very complex type" clippy refuses under -D warnings.
    type VisCase<'a> = (Lang, &'a str, &'a [(&'a str, bool)]);

    /// Each language's rule, one row per language. Written as a table
    /// rather than a test per language because a per-language
    /// assertion body is a T2 clone chain by this repo's own measure
    /// — the same reason EraseProps carries its truth table as data.
    #[test]
    fn exported_bits_follow_each_language_declaration_rule() {
        let cases: &[VisCase] = &[
            (
                Lang::Rust,
                "pub fn open() {}\nfn shut() {}\npub(crate) fn near() {}\n",
                &[("open", true), ("shut", false), ("near", true)],
            ),
            (
                Lang::TypeScript,
                "export function open() {}\nfunction shut() {}\n",
                &[("open", true), ("shut", false)],
            ),
            (
                Lang::Python,
                "def open():\n    pass\ndef _shut():\n    pass\n",
                &[("open", true), ("_shut", false)],
            ),
            (
                Lang::Go,
                "package p\nfunc Open() {}\nfunc shut() {}\n",
                &[("Open", true), ("shut", false)],
            ),
        ];
        for (lang, src, want) in cases {
            let expect: Vec<(String, bool)> =
                want.iter().map(|(n, e)| ((*n).to_string(), *e)).collect();
            assert_eq!(fns(src, *lang), expect, "{lang:?}");
        }
    }

    /// A header WITHOUT an export list exports everything — the
    /// language's rule, so both names read exported.
    #[test]
    fn haskell_headerless_module_exports_everything() {
        let src = "module M where\nopen :: Int\nopen = 1\nshut :: Int\nshut = 2\n";
        for (_, exported) in fns(src, Lang::Haskell) {
            assert!(exported, "a list-less header exports everything");
        }
    }

    /// A header WITH a list exports exactly it. The negative half is
    /// the load-bearing one: without it the export-list reader could
    /// be returning None for every module and the test above would
    /// still pass.
    #[test]
    fn haskell_export_list_is_read_from_the_header() {
        let src = "module M (open) where\nopen :: Int\nopen = 1\nshut :: Int\nshut = 2\n";
        let got = fns(src, Lang::Haskell);
        assert!(!got.is_empty(), "the Haskell extractor found no units");
        for (name, exported) in got {
            assert_eq!(exported, name == "open", "{name} judged {exported}");
        }
    }

    /// Markdown has no private heading: the constant units.rs stamps
    /// on every section says so, and this pins it against a silent
    /// flip to 0 (which would make every section read private).
    #[test]
    fn markdown_sections_are_public_by_construction() {
        assert_eq!(MARKDOWN_VIS & VIS_EXPORTED, VIS_EXPORTED);
    }
}
