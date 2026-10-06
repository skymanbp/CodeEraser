//! The six-channel bag of one code unit (spec §三), read off facts
//! the tree already carries: the unit key (N), the declaration node's
//! kind / arity / return (P), the callee spellings in its own body
//! (C), the comment or docstring that belongs to it (D), the unitsig
//! kind histogram (S) and the KINDS — never the values — of its
//! literals (L). One parse per file; units and nth come from the
//! throats the unitsig cache uses (fourclass::units), so the bag
//! universe IS the T3 universe, and Markdown has no bags. This side
//! reads the tree and sends each unit's facts as one row; the words,
//! stems, hashes and the bag itself are the core's (bags.rs, bags/1).

use super::docs::{DocSeg, doc_owner, doc_segments};
use super::terms::Channel;
use crate::dedup::struct_fp;
use crate::fourclass::units::{self, Unit};
use crate::scan::ast;
use crate::scan::lang::Lang;
use crate::scan::metrics::own_nodes;
use crate::scan::spec::{self, LangSpec};
use crate::scan::{callees, functions};
use serde_json::{Value, json};
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
}

/// Every unit's bag for one file, in nth-throat order. A parse
/// failure or a grammarless language yields none; a core that cannot
/// answer is the named refusal.
pub fn file_bags(text: &str, lang: Lang) -> Result<Vec<UnitBag>, String> {
    bagged(file_rows(text, lang))
}

/// The core's terms seated on the units the rows were read from.
pub fn bagged(rows: Vec<(UnitBag, Value)>) -> Result<Vec<UnitBag>, String> {
    let (mut bags, rows): (Vec<UnitBag>, Vec<Value>) = rows.into_iter().unzip();
    let (terms, _) = super::bags::ask(rows, &[])?;
    for (bag, terms) in bags.iter_mut().zip(terms) {
        bag.terms = terms;
    }
    Ok(bags)
}

/// Every unit of one file with its bags/1 row, the bag's terms still
/// empty — what the core is asked about.
pub fn file_rows(text: &str, lang: Lang) -> Vec<(UnitBag, Value)> {
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

/// One unit's row: `[key, kind, ret, callees, literals, structure,
/// doc lines]` — the key (N, and the arity P reads), the kind word and,
/// for a callable, whether it declares a return (P), the callee
/// spellings of its own body (C), its literal kinds (L), its structure
/// histogram `[kind, n]` (S) and the lines of the docs it owns (D).
fn build(u: &Unit, nth: i64, seat: usize, node: Node<'_>, f: &FileFacts<'_>) -> (UnitBag, Value) {
    let own = own_nodes(node, f.src, f.sp);
    let literals: Vec<&str> = own.iter().filter_map(|n| literal_kind(*n)).collect();
    let seq = struct_fp::unit_seq(&f.spine, u.start_line, u.end_line);
    let structure: Vec<[u64; 2]> = struct_fp::histogram(&seq)
        .into_iter()
        .map(|(kind, n)| [kind, u64::from(n)])
        .collect();
    let docs: Vec<&String> = f
        .docs
        .iter()
        .zip(&f.owner)
        .filter(|(_, o)| **o == Some(seat))
        .flat_map(|(d, _)| &d.lines)
        .collect();
    let ret =
        functions::is_unit_node(node, f.src, f.sp).then(|| u8::from(declares_return(node, f.src)));
    let kind = kind_word(u, node, f.src, f.sp);
    let callees = callees(&own, f.src, f.sp);
    let row = json!([u.key, kind, ret, callees, literals, structure, docs]);
    let bag = UnitBag {
        start_line: u.start_line,
        end_line: u.end_line,
        ..UnitBag::empty(u.key.clone(), nth)
    };
    (bag, row)
}

/// Whether a callable declares what it returns: a `return_type`
/// (Rust, Python, TypeScript) or `result` (Go) field, or a `type`
/// field that is not `void` — the C family and Java spell the return
/// type in front of the name, and a constructor spells none.
fn declares_return(node: Node<'_>, src: &[u8]) -> bool {
    ["return_type", "result"]
        .iter()
        .any(|f| node.child_by_field_name(f).is_some())
        || node
            .child_by_field_name("type")
            .is_some_and(|ty| ty.utf8_text(src) != Ok("void"))
}

/// The kind word: a callable is a lambda, a method (the grammar's own
/// method kinds, or a declaration in a member scope by the reading
/// scan/callees.rs uses) or a plain fn; the named register's kinds
/// fold to const / mod / impl / class / type.
fn kind_word(u: &Unit, node: Node<'_>, src: &[u8], sp: &LangSpec) -> &'static str {
    if functions::is_unit_node(node, src, sp) {
        let method = matches!(node.kind(), "method_declaration" | "method_definition")
            || callees::in_member_scope(node, sp);
        return if u.key.starts_with("(anonymous)") {
            "lambda"
        } else if method {
            "method"
        } else {
            "fn"
        };
    }
    match node.kind() {
        "const_item" | "static_item" | "preproc_def" | "preproc_function_def" => "const",
        "mod_item" | "namespace_definition" => "mod",
        "impl_item" => "impl",
        "class_definition"
        | "class_declaration"
        | "class"
        | "class_specifier"
        | "trait_item"
        | "interface_declaration" => "class",
        _ => "type",
    }
}

/// Callee spellings in the unit's own body: the callee field of every
/// call node (LangSpec::call_fields — the arcs read the same one), a
/// bare name whole or a member's last segment, as spelled (the core
/// splits it).
fn callees<'t, 's>(own: &[Node<'t>], src: &'s [u8], sp: &LangSpec) -> Vec<&'s str> {
    let named = |callee: Node<'t>| -> Option<Node<'t>> {
        if sp.call_name_kinds.contains(&callee.kind()) {
            Some(callee)
        } else if sp.call_member_kinds.contains(&callee.kind()) {
            ast::named_children(callee).last().copied()
        } else {
            None
        }
    };
    own.iter()
        .filter(|n| sp.call_kinds.contains(&n.kind()))
        .filter_map(|n| n.child_by_field_name(sp.call_fields.0))
        .filter_map(named)
        .filter_map(|n| n.utf8_text(src).ok())
        .collect()
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
