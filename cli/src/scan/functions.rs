//! Function-unit extraction from a parsed tree, driven by LangSpec.
//! A "function unit" is a node whose kind is in `fn_kinds`; nested
//! standalone units are measured separately and skipped inside their
//! host (see metrics walkers).
//!
//! Three roads spell a unit's name, tried in this order: the node's
//! own `name` field (five grammars), the leaf of its declarator chain
//! (the C family — a definition names itself through nested
//! declarators, see scan/declarator.rs), and the enclosing declarator /
//! pair / assignment an anonymous function hangs off (arrow functions).

use super::ast;
use super::declarator;
use super::spec::LangSpec;
use tree_sitter::Node;

pub struct FnUnit<'t> {
    pub node: Node<'t>,
    pub name: String,
    pub start_line: usize, // 1-based
    pub end_line: usize,   // 1-based inclusive
    pub params: usize,
}

/// THE standalone-unit predicate — extraction, own_nodes and the
/// cognitive walker must agree on what a unit is, so they all call
/// this one throat. Name-gated kinds (Haskell `bind`, which is also
/// the do-statement / pattern-bind kind) only open a unit when the
/// node carries a `name` field; a declarator-named kind (the C
/// family's function_definition) only when it is shaped like a
/// definition and not nested in one (declarator::defined — the source
/// is what lets a typeless definition prove it is a constructor).
pub fn is_unit_node(node: Node<'_>, src: &[u8], spec: &LangSpec) -> bool {
    spec.fn_kinds.contains(&node.kind())
        && (!spec.fn_named_only_kinds.contains(&node.kind())
            || node.child_by_field_name("name").is_some())
        && declarator::defined(node, src)
}

pub fn extract<'t>(root: Node<'t>, src: &[u8], spec: &LangSpec) -> Vec<FnUnit<'t>> {
    let mut out: Vec<FnUnit<'t>> = ast::preorder(root, |_| true, ast::children)
        .into_iter()
        .filter(|&node| is_unit_node(node, src, spec))
        .map(|node| unit(node, src, spec))
        .collect();
    out.sort_by_key(|f| f.start_line);
    out
}

fn unit<'t>(node: Node<'t>, src: &[u8], spec: &LangSpec) -> FnUnit<'t> {
    FnUnit {
        node,
        name: name_of(node, src),
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        params: param_count(node, src, spec),
    }
}

/// Name from the node's `name` field, else from the C family's
/// declarator chain, else from an enclosing variable_declarator/pair
/// (arrow functions), else "(anonymous)".
/// Go methods carry their receiver TYPE as a prefix — `(T) add` and
/// `(*U) add` are different identities (attack review F7), and the
/// qualification lives HERE at the one extraction root so metric
/// names, unit keys and continuous baseline entities agree by
/// construction (M5-close review D4: a post-pass in fourclass let
/// the baseline re-spell the key without it). C++ members carry
/// their class chain the same way: `Outer::Inner::m` for a definition
/// inside the bodies, `K::b` for one written out of class — the two
/// spellings meet in the owner half (owner_of), which scan/calls.rs
/// keys its member road by. Crate-visible because the TS visibility
/// climb's identity guard compares a declarator's name against THIS
/// spelling of the unit's name — the one producer of the key's name
/// part, so the guard cannot drift from the key (plan v2.17 L round,
/// criterion T3).
pub(crate) fn name_of(node: Node<'_>, src: &[u8]) -> String {
    if let Some(name) = node.child_by_field_name("name") {
        let base = text(name, src);
        return match receiver_type(node, src) {
            Some(recv) => format!("({recv}) {base}"),
            None => base,
        };
    }
    if let Some((owner, base)) = declarator::identity(node, src) {
        return match owner {
            Some(owner) => format!("{owner}::{base}"),
            None => base,
        };
    }
    if let Some(parent) = node.parent()
        && matches!(parent.kind(), "variable_declarator" | "pair" | "assignment")
        && let Some(name) = parent.child_by_field_name(field_for(parent.kind()))
    {
        return text(name, src);
    }
    "(anonymous)".to_string()
}

/// The owner half of a C-family unit's identity — the class chain a
/// definition sits in, or the qualifier an out-of-class definition
/// spells (`Outer::Inner`, `ns::K`) — and None for every other grammar
/// and for a free function. scan/calls.rs keys its member road by it:
/// `this->m()` inside `K::b` reaches `K::m` whether `m` was defined in
/// the class body or out of it.
pub(crate) fn owner_of(node: Node<'_>, src: &[u8]) -> Option<String> {
    declarator::identity(node, src)?.0
}

/// The receiver's type text (`T`, `*U`) for a Go method_declaration —
/// the identity part; the binding name is deliberately excluded so
/// renaming `(t T)` to `(x T)` keeps the cross-version key stable.
fn receiver_type(node: Node<'_>, src: &[u8]) -> Option<String> {
    if node.kind() != "method_declaration" {
        return None;
    }
    let recv = node.child_by_field_name("receiver")?;
    ast::named_children(recv)
        .into_iter()
        .find(|c| c.kind() == "parameter_declaration")
        .and_then(|p| p.child_by_field_name("type"))
        .map(|ty| text(ty, src))
}

fn field_for(parent_kind: &str) -> &'static str {
    match parent_kind {
        "pair" => "key",
        "assignment" => "left",
        _ => "name",
    }
}

/// The `parameters` FIELD wins over the kind scan (M5 close, repaying
/// the 3h blind-audit defect): Go method_declaration's RECEIVER is
/// itself a parameter_list, so a first-of-kind scan counted the
/// receiver (always 1) and collapsed every method's arity. The field
/// survey (probe 2026-08-14): Go/Rust/Python/TS carry `parameters`
/// naming exactly the node the kind scan found — identical counts —
/// while Go methods name the REAL list past the receiver; Haskell has
/// no such field and keeps the `patterns` kind fallback. Go's grouped
/// `a, b int` stays ONE declaration — arity counts declarations, the
/// pre-existing stance, untouched here. The C family names no such
/// field on the definition: its list hangs off the innermost
/// function_declarator (declarator::chain), and `(void)` spells an
/// EMPTY list (register D12).
fn param_count(node: Node<'_>, src: &[u8], spec: &LangSpec) -> usize {
    let params = match node.child_by_field_name("parameters") {
        Some(field) => field,
        None => match chain_params(node).or_else(|| child_of_kinds(node, spec.param_list_kinds)) {
            Some(found) => found,
            None => return 0,
        },
    };
    let kids: Vec<Node<'_>> = ast::named_children(params)
        .into_iter()
        .filter(|c| !c.kind().contains("comment"))
        .collect();
    if void_only(&kids, src) {
        return 0;
    }
    kids.len()
}

fn chain_params(node: Node<'_>) -> Option<Node<'_>> {
    declarator::chain(node)?
        .1?
        .child_by_field_name("parameters")
}

/// C's `f(void)`: one parameter_declaration whose type reads `void`
/// and which binds no name — the spelling of "no parameters", not a
/// parameter (register D12; `void *p` binds a declarator and counts).
fn void_only(kids: &[Node<'_>], src: &[u8]) -> bool {
    let [only] = kids else {
        return false;
    };
    only.kind() == "parameter_declaration"
        && only.child_by_field_name("declarator").is_none()
        && only
            .child_by_field_name("type")
            .is_some_and(|ty| ty.utf8_text(src) == Ok("void"))
}

fn child_of_kinds<'t>(node: Node<'t>, kinds: &[&str]) -> Option<Node<'t>> {
    ast::children(node)
        .into_iter()
        .find(|c| kinds.contains(&c.kind()))
}

fn text(node: Node<'_>, src: &[u8]) -> String {
    node.utf8_text(src).unwrap_or("(non-utf8)").to_string()
}
