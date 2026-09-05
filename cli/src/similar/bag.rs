//! The six-channel bag of one code unit (spec §三), read off facts
//! the tree already carries: the unit key (N), the declaration node's
//! kind / arity / return (P), the callee spellings in its own body
//! (C), the comment or docstring that belongs to it (D), the unitsig
//! kind histogram (S) and the KINDS — never the values — of its
//! literals (L). One parse per file; units and nth come from the
//! throats the unitsig cache uses (fourclass::units), so the bag
//! universe IS the T3 universe, and Markdown has no bags.

use super::docs::{DocSeg, doc_owner, doc_segments};
use super::terms::{self, Channel};
use crate::dedup::struct_fp;
use crate::fourclass::units::{self, Unit};
use crate::scan::ast;
use crate::scan::lang::Lang;
use crate::scan::metrics::own_nodes;
use crate::scan::spec::{self, LangSpec};
use crate::scan::{calls, functions};
use std::collections::BTreeMap;
use tree_sitter::Node;

/// One unit's bag: identity, span, and `term → (channel, tf)`.
#[derive(Debug, Clone)]
pub struct UnitBag {
    pub key: String,
    pub nth: i64,
    pub start_line: usize,
    pub end_line: usize,
    pub terms: BTreeMap<u64, (Channel, u32)>,
}

impl UnitBag {
    /// A unit with identity and no terms yet — the shape the stored
    /// rows are grouped back into (spans live on the reader's seat).
    pub fn empty(key: String, nth: i64) -> UnitBag {
        UnitBag {
            key,
            nth,
            start_line: 0,
            end_line: 0,
            terms: BTreeMap::new(),
        }
    }

    /// Σ tf — the BM25 document length.
    pub fn len(&self) -> u32 {
        self.terms.values().map(|(_, tf)| tf).sum()
    }

    /// A unit with no term at all (a parse that yielded a unit but no
    /// name, shape, body, doc, structure or literal — nothing to rank).
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// The sorted terms of one channel.
    pub fn channel(&self, ch: Channel) -> Vec<u64> {
        self.terms
            .iter()
            .filter(|(_, (c, _))| *c == ch)
            .map(|(t, _)| *t)
            .collect()
    }

    fn add(&mut self, ch: Channel, term: u64, n: u32) {
        self.terms.entry(term).or_insert((ch, 0)).1 += n;
    }
}

/// Every unit's bag for one file, in nth-throat order. A parse
/// failure or a grammarless language yields none.
pub fn file_bags(text: &str, lang: Lang) -> Vec<UnitBag> {
    let Some(tree) = ast::parse_lang(text, lang) else {
        return Vec::new();
    };
    let src = text.as_bytes();
    let sp = spec::spec(lang);
    let seated = units::node_segments(tree.root_node(), src, lang);
    let flat: Vec<Unit> = seated.iter().map(|(u, _)| u.clone()).collect();
    let docs = doc_segments(text, lang);
    let facts = FileFacts {
        owner: docs.iter().map(|d| doc_owner(d, &flat)).collect(),
        src,
        sp,
        spine: struct_fp::spine(tree.root_node()),
        docs,
    };
    let mut taken = vec![false; seated.len()];
    units::with_nth(&flat)
        .into_iter()
        .map(|(u, nth)| {
            let seat = seat_of(&seated, &mut taken, u);
            build(u, nth, seat, seated[seat].1, &facts)
        })
        .collect()
}

/// The first unclaimed seat holding `u` — two units can share key AND
/// span (two closures sliced to one line, t3 review record), and
/// with_nth tells them apart only by order.
fn seat_of(seated: &[(Unit, Node<'_>)], taken: &mut [bool], u: &Unit) -> usize {
    let i = seated
        .iter()
        .enumerate()
        .position(|(i, (s, _))| !taken[i] && s == u)
        .expect("with_nth returns the units it was given");
    taken[i] = true;
    i
}

/// The per-file facts every unit's bag reads; `owner[i]` = the seat
/// of the unit doc segment `i` belongs to (doc_owner), if any.
struct FileFacts<'f> {
    owner: Vec<Option<usize>>,
    src: &'f [u8],
    sp: &'f LangSpec,
    spine: struct_fp::Spine,
    docs: Vec<DocSeg>,
}

fn build(u: &Unit, nth: i64, seat: usize, node: Node<'_>, f: &FileFacts<'_>) -> UnitBag {
    let mut bag = UnitBag {
        key: u.key.clone(),
        nth,
        start_line: u.start_line,
        end_line: u.end_line,
        terms: BTreeMap::new(),
    };
    for w in name_words(&u.key) {
        bag.add(Channel::Name, terms::word_term(Channel::Name, &w), 1);
    }
    for s in shape(u, node, f.sp) {
        bag.add(
            Channel::Shape,
            terms::feature_term(Channel::Shape, s.as_bytes()),
            1,
        );
    }
    let own = own_nodes(node, f.sp);
    for w in callees(&own, f.src, f.sp) {
        bag.add(Channel::Callee, terms::word_term(Channel::Callee, &w), 1);
    }
    for kind in own.iter().filter_map(|n| literal_kind(*n)) {
        bag.add(
            Channel::Literal,
            terms::feature_term(Channel::Literal, kind.as_bytes()),
            1,
        );
    }
    let seq = struct_fp::unit_seq(&f.spine, u.start_line, u.end_line);
    for (kind, n) in struct_fp::histogram(&seq) {
        let term = terms::feature_term(Channel::Structure, &kind.to_le_bytes());
        bag.add(Channel::Structure, term, n);
    }
    let owned = f
        .docs
        .iter()
        .zip(&f.owner)
        .filter(|(_, o)| **o == Some(seat));
    for w in owned.flat_map(|(d, _)| &d.words) {
        bag.add(Channel::Doc, terms::word_term(Channel::Doc, w), 1);
    }
    bag
}

/// The arity suffix of a function key (`name/3` → Some("3")).
fn arity(key: &str) -> Option<&str> {
    key.rsplit_once('/')
        .filter(|(_, n)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .map(|(_, n)| n)
}

/// The words of a unit key: the arity dropped, an `impl T for U` its
/// two keywords, an anonymous unit its placeholder (a closure has no
/// name evidence at all); split_ident does the rest.
fn name_words(key: &str) -> Vec<String> {
    let base = arity(key).map_or(key, |n| &key[..key.len() - n.len() - 1]);
    if base == "(anonymous)" {
        return Vec::new();
    }
    let mut words = terms::split_ident(base);
    if base.starts_with("impl ") {
        words.retain(|w| w != "impl" && w != "for");
    }
    words
}

/// Shape features: the unit's kind word, its arity, and for a
/// callable whether it declares a return.
fn shape(u: &Unit, node: Node<'_>, sp: &LangSpec) -> Vec<String> {
    let mut out = vec![format!("k:{}", kind_word(u, node, sp))];
    if let Some(n) = arity(&u.key) {
        out.push(format!("p:{n}"));
    }
    if functions::is_unit_node(node, sp) {
        let ret = ["return_type", "result"]
            .iter()
            .any(|f| node.child_by_field_name(f).is_some());
        out.push(format!("ret:{}", u8::from(ret)));
    }
    out
}

/// The kind word: a callable is a lambda, a method (the grammar's own
/// method kinds, or a declaration in a member scope by the reading
/// scan/calls.rs uses) or a plain fn; the named register's kinds fold
/// to const / mod / impl / class / type.
fn kind_word(u: &Unit, node: Node<'_>, sp: &LangSpec) -> &'static str {
    if functions::is_unit_node(node, sp) {
        let method = matches!(node.kind(), "method_declaration" | "method_definition")
            || calls::in_member_scope(node, sp);
        return if u.key.starts_with("(anonymous)") {
            "lambda"
        } else if method {
            "method"
        } else {
            "fn"
        };
    }
    match node.kind() {
        "const_item" | "static_item" => "const",
        "mod_item" => "mod",
        "impl_item" => "impl",
        "class_definition"
        | "class_declaration"
        | "class"
        | "trait_item"
        | "interface_declaration" => "class",
        _ => "type",
    }
}

/// Callee spellings in the unit's own body: the `function` field of
/// every call node, a bare name whole or a member's last segment.
fn callees(own: &[Node<'_>], src: &[u8], sp: &LangSpec) -> Vec<String> {
    let mut out = Vec::new();
    for node in own.iter().filter(|n| sp.call_kinds.contains(&n.kind())) {
        let Some(callee) = node.child_by_field_name("function") else {
            continue;
        };
        let name = if sp.call_name_kinds.contains(&callee.kind()) {
            Some(callee)
        } else if sp.call_member_kinds.contains(&callee.kind()) {
            ast::named_children(callee).last().copied()
        } else {
            None
        };
        if let Some(text) = name.and_then(|n| n.utf8_text(src).ok()) {
            out.extend(terms::split_ident(text));
        }
    }
    out
}

/// The literal KIND of a node, counted once per literal: a node whose
/// parent is itself a literal (a string's content piece, a boolean's
/// keyword) is not counted again.
fn literal_kind(node: Node<'_>) -> Option<&'static str> {
    let word = literal_word(node.kind())?;
    node.parent()
        .and_then(|p| literal_word(p.kind()))
        .is_none()
        .then_some(word)
}

fn literal_word(kind: &str) -> Option<&'static str> {
    if matches!(kind, "true" | "false" | "boolean_literal") {
        return Some("l:bool");
    }
    if kind.contains("string") || kind.contains("char") || kind.contains("rune") {
        return Some("l:str");
    }
    if kind.contains("int") || kind.contains("float") || kind.contains("number") {
        return Some("l:num");
    }
    kind.ends_with("literal").then_some("l:other")
}

#[cfg(test)]
#[path = "../../tests/unit/similar/bag.rs"]
mod tests;
