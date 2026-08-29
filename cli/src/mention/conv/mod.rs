//! The convention-category word's AST half (sealed criterion §3.2,
//! plan v2.17 L round piece (4)): which of the twelve frozen category
//! bits a declaration earns from ITS OWN syntax — an attribute, a
//! decorator, an enclosing class or ambient block, a directive comment
//! — measured where the declaration node is in hand (fourclass::units)
//! and stored as `symbols.conv` (graph/store.rs, GRAPH_REV 12). The
//! other half of the word is read off the key and the path at wire
//! time and never stored; the two halves OR together there, and the
//! renderer reads the same assembled word. Every bit is an exemption
//! the core may grant an unmentioned declaration — `Cfg` alone is
//! rendered and never exempts — so a bit measured here can only
//! silence an advisory row: the safe direction of the veto.
//!
//! Positions are frozen. Adding, removing or re-reading an AST-half
//! producer is a GRAPH_REV bump; the name-table half moves freely.

pub mod name;
mod py;
mod rs;
#[cfg(test)]
#[path = "../../../tests/unit/mention/conv/tests.rs"]
mod tests;

use crate::fourclass::visibility::ancestors;
use crate::scan::ast;
use crate::scan::lang::Lang;
use std::collections::BTreeSet;
use tree_sitter::Node;

/// The twelve categories. A variant's discriminant IS its bit
/// position — in `symbols.conv` and on the wire — and `bit()` is its
/// mask; which half measures it is written on each variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum Conv {
    /// Python/Haskell `main` (name half).
    Main = 0,
    /// A test file by path (name half), or a Rust `cfg` predicate
    /// holding the atom `test` (AST half, rs.rs).
    Test = 1,
    /// A foreign-function surface: Rust export attributes and
    /// `extern`, a Haskell `foreign export`, a Go `//export` directive
    /// (AST half).
    Ffi = 2,
    /// Registered by a decorator: Python's registrar table, any TS
    /// decorator (AST half).
    Registration = 3,
    /// A framework protocol name (name half).
    Protocol = 4,
    /// A Python class member (AST half).
    Member = 5,
    /// A Go method on an unexported receiver (name half).
    MemberDispatch = 6,
    /// A Go method on an exported receiver (name half).
    MemberApi = 7,
    /// A named TS `export default function/class` (AST half).
    DefaultExport = 8,
    /// Under a TS `ambient_declaration` (AST half), or a declaration
    /// file by path (name half).
    Ambient = 9,
    /// A `ce:allow(unmentioned)` claim (name half), or a Rust
    /// `allow(dead_code)` / `expect(dead_code)` (AST half).
    Allow = 10,
    /// A Rust `cfg` predicate with atoms and no `test` — rendered,
    /// never exempting.
    Cfg = 11,
}

impl Conv {
    /// The category's mask in the word.
    pub const fn bit(self) -> i64 {
        1 << self as i64
    }
}

/// What the AST half reads beside the node: the names a Haskell
/// `foreign export` hands out — the exported name hits its declaration,
/// never the file (X-16). Empty for every other language.
#[derive(Default)]
pub struct FileFacts {
    foreign_exports: BTreeSet<String>,
}

/// One pass over the file root, before the units are walked.
pub fn file_facts(root: Node<'_>, src: &[u8], lang: Lang) -> FileFacts {
    FileFacts {
        foreign_exports: match lang {
            Lang::Haskell => foreign_exports(root, src),
            _ => BTreeSet::new(),
        },
    }
}

/// The AST half of one declaration's category word.
pub fn ast_bits(node: Node<'_>, src: &[u8], lang: Lang, facts: &FileFacts) -> i64 {
    match lang {
        Lang::Rust => rs::bits(node, src),
        Lang::TypeScript | Lang::Tsx => ts_bits(node),
        Lang::Python => py::bits(node, src),
        Lang::Go => go_bits(node, src),
        Lang::Haskell => hs_bits(node, src, &facts.foreign_exports),
        _ => 0,
    }
}

/// A node's bytes as text — shared with the self-mention walk, whose
/// own copy the clone gate paired with this one.
pub(super) fn text<'s>(node: Node<'_>, src: &'s [u8]) -> &'s str {
    node.utf8_text(src).unwrap_or("")
}

/// The node's parent when it is of `kind`.
fn parent_of_kind<'t>(node: Node<'t>, kind: &str) -> Option<Node<'t>> {
    node.parent().filter(|p| p.kind() == kind)
}

fn under(node: Node<'_>, kind: &str) -> bool {
    ancestors(node).any(|a| a.kind() == kind)
}

// ---- Go (AST half) ----

// Go: `Ffi` from the directive comments cgo and the wasm target read
// — `//export Name` must name this very function (cgo's own lexing:
// the prefix `//export `, then the name with surrounding blanks
// trimmed), `//go:wasmexport <symbol>` exports the function it
// precedes under any symbol. Comments are siblings in tree-sitter-go,
// so the directive is found in the comment run above the
// declaration; the first non-comment sibling ends the run. Blank
// lines do not end it, so the run is wider than cgo's doc group —
// an over-exemption only, never a missed export.
fn go_bits(node: Node<'_>, src: &[u8]) -> i64 {
    let Some(name) = node.child_by_field_name("name") else {
        return 0;
    };
    let name = text(name, src);
    let mut prev = node.prev_sibling();
    while let Some(c) = prev.filter(|c| c.kind() == "comment") {
        let line = text(c, src);
        let exported = line
            .strip_prefix("//export ")
            .is_some_and(|rest| rest.trim() == name);
        if exported || line.starts_with("//go:wasmexport ") {
            return Conv::Ffi.bit();
        }
        prev = c.prev_sibling();
    }
    0
}

// ---- Haskell (AST half) ----

// Haskell: `Ffi`. `foreign export ccall hsAdd :: CInt -> CInt` is
// its own top-level declaration whose `signature` names the binding
// it exports; the bit lands on THAT binding (X-16), so the file's
// exported names are gathered once (`foreign_exports`, read by
// conv::file_facts before the units are walked) and each unit is
// looked up by its own name. The C-side entity string (`"hs_mul"`)
// is the foreign spelling and plays no part.
fn foreign_exports(root: Node<'_>, src: &[u8]) -> BTreeSet<String> {
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

/// The Haskell arm: the unit whose own name the file exports.
fn hs_bits(node: Node<'_>, src: &[u8], exported: &BTreeSet<String>) -> i64 {
    match node.child_by_field_name("name") {
        Some(n) if exported.contains(text(n, src)) => Conv::Ffi.bit(),
        _ => 0,
    }
}

// ---- TS/TSX (AST half) ----

// TS/TSX: three facts, each read off the tree the grammar actually
// builds (probed on tree-sitter-typescript 0.23.2):
//   - `Registration`: a decorator is a `decorator` child of the
//     declaration (`@dec class A`, `export @dec class B`) or of its
//     `export_statement` parent (`@dec export class C`) — the two
//     spellings judge alike (L3-F7a), and a comment between decorator
//     and name is just another child, never a hop.
//   - `DefaultExport`: a NAMED function or class declaration whose
//     `export_statement` carries the `default` keyword; an anonymous
//     default export has no name to be mentioned by and is out of the
//     domain already (`(anonymous)`).
//   - `Ambient`: any `ambient_declaration` ancestor — the criterion's
//     L5-F15 ruling that the container's shape is never the test,
//     only the ancestor's presence (`declare class C {}` has no
//     container at all).
// Declaration kinds `export default` can name.
const DEFAULTABLE: [&str; 3] = [
    "function_declaration",
    "generator_function_declaration",
    "class_declaration",
];

fn ts_bits(node: Node<'_>) -> i64 {
    let mut word = 0;
    let export = parent_of_kind(node, "export_statement");
    if has_child(node, "decorator") || export.is_some_and(|e| has_child(e, "decorator")) {
        word |= Conv::Registration.bit();
    }
    if DEFAULTABLE.contains(&node.kind())
        && node.child_by_field_name("name").is_some()
        && export.is_some_and(|e| has_child(e, "default"))
    {
        word |= Conv::DefaultExport.bit();
    }
    if under(node, "ambient_declaration") {
        word |= Conv::Ambient.bit();
    }
    word
}

/// Anonymous leaves (`default`) and named ones (`decorator`) alike.
fn has_child(node: Node<'_>, kind: &str) -> bool {
    ast::children(node).iter().any(|c| c.kind() == kind)
}
