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

mod go;
mod hs;
pub mod name;
mod py;
mod rs;
#[cfg(test)]
mod tests;
mod ts;

use crate::fourclass::visibility::ancestors;
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
            Lang::Haskell => hs::foreign_exports(root, src),
            _ => BTreeSet::new(),
        },
    }
}

/// The AST half of one declaration's category word.
pub fn ast_bits(node: Node<'_>, src: &[u8], lang: Lang, facts: &FileFacts) -> i64 {
    match lang {
        Lang::Rust => rs::bits(node, src),
        Lang::TypeScript | Lang::Tsx => ts::bits(node),
        Lang::Python => py::bits(node, src),
        Lang::Go => go::bits(node, src),
        Lang::Haskell => hs::bits(node, src, &facts.foreign_exports),
        _ => 0,
    }
}

/// A node's bytes as text — shared with the self-mention walk, whose
/// own copy the clone gate paired with this one.
pub(super) fn text<'s>(node: Node<'_>, src: &'s [u8]) -> &'s str {
    node.utf8_text(src).unwrap_or("")
}

/// The one letter alphabet the conv test tables spell their expected
/// words in — a letter per category, `-` for none. ONE table for both
/// halves: the AST-half and name-half batteries each kept a copy of
/// this match until the clone gate paired them.
#[cfg(test)]
pub(super) fn bit_of(letter: char) -> i64 {
    let category = match letter {
        'm' => Conv::Main,
        'T' => Conv::Test,
        'F' => Conv::Ffi,
        'G' => Conv::Registration,
        'P' => Conv::Protocol,
        'M' => Conv::Member,
        'd' => Conv::MemberDispatch,
        'a' => Conv::MemberApi,
        'D' => Conv::DefaultExport,
        'A' => Conv::Ambient,
        'L' => Conv::Allow,
        'C' => Conv::Cfg,
        '-' => return 0,
        other => panic!("unknown bit letter {other:?}"),
    };
    category.bit()
}

/// The node's parent when it is of `kind`.
fn parent_of_kind<'t>(node: Node<'t>, kind: &str) -> Option<Node<'t>> {
    node.parent().filter(|p| p.kind() == kind)
}

fn under(node: Node<'_>, kind: &str) -> bool {
    ancestors(node).any(|a| a.kind() == kind)
}
