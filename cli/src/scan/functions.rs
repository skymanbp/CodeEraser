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
/// this one throat. A gated kind only opens a unit when the node
/// carries its field (fn_required_fields: Haskell's `bind` needs a
/// `name`, a Java method a `body`); a declarator-named kind (the C
/// family's function_definition) only when it is shaped like a
/// definition and not nested in one (declarator::defined — the source
/// is what lets a typeless definition prove it is a constructor).
pub fn is_unit_node(node: Node<'_>, src: &[u8], spec: &LangSpec) -> bool {
    let kind = node.kind();
    spec.fn_kinds.contains(&kind)
        && spec
            .fn_required_fields
            .iter()
            .all(|&(gated, field)| gated != kind || node.child_by_field_name(field).is_some())
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

/// The class a unit is a member of, spelled the way a call's
/// qualifier can name it. The C family spells it through the
/// declarator — the class chain a definition sits in, or the qualifier
/// an out-of-class one writes (`Outer::Inner`, `ns::K`) — and that
/// spelling is also the owner scan/callees.rs keys the member road by,
/// so `this->m()` inside `K::b` reaches `K::m` wherever `m` was
/// defined. Java spells it as the name of the type declaration whose
/// body holds the unit (LangSpec::owner_kinds); its member road keys by
/// the body itself (callees::Owner). None for a free function, for
/// every other grammar, and for a member of an anonymous class
/// (`new T() { … }`, an enum constant's body), which no qualifier can
/// name.
pub(crate) fn owner_of(node: Node<'_>, src: &[u8], spec: &LangSpec) -> Option<String> {
    if spec.owner_kinds.is_empty() {
        return declarator::identity(node, src)?.0;
    }
    let holder = ast::ancestors(node)
        .find(|a| spec.call_member_scopes.contains(&a.kind()))?
        .parent()?;
    if !spec.owner_kinds.contains(&holder.kind()) {
        return None;
    }
    Some(text(holder.child_by_field_name("name")?, src))
}

/// The argument counts a unit accepts, `(fewest, most)` with `most`
/// None = no upper bound, read where the grammar overloads
/// (LangSpec::overloads — its kinds say which parameter counts toward
/// which bound). A grammar that does not overload answers `(0, None)`:
/// its same-named units are one callable, and a count decides nothing.
/// A C `(void)` list, and a unit with no list, accept none.
pub(crate) fn arity(node: Node<'_>, src: &[u8], spec: &LangSpec) -> (usize, Option<usize>) {
    let Some(overloads) = spec.overloads else {
        return (0, None);
    };
    let Some(params) = param_list(node, spec.param_list_kinds) else {
        return (0, Some(0));
    };
    if void_only(&ast::entries(params), src) {
        return (0, Some(0));
    }
    let (mut fewest, mut most, mut open) = (0, 0, false);
    for kid in ast::children(params) {
        let kind = kid.kind();
        if overloads.variadic.contains(&kind) {
            open = true;
        } else if overloads.optional.contains(&kind) {
            most += 1;
        } else if kid.is_named() && !kind.contains("comment") && !overloads.ignored.contains(&kind)
        {
            fewest += 1;
            most += 1;
        }
    }
    (fewest, (!open).then_some(most))
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
/// EMPTY list (register D12). A Java receiver parameter (`void m(K
/// this)`) is no formal parameter either (JLS 8.4.1): the kinds
/// Overloads::ignored names, which take no argument, count for neither
/// reading.
fn param_count(node: Node<'_>, src: &[u8], spec: &LangSpec) -> usize {
    let Some(params) = param_list(node, spec.param_list_kinds) else {
        return 0;
    };
    let kids = ast::entries(params);
    if void_only(&kids, src) {
        return 0;
    }
    let ignored = spec.overloads.map_or(&[][..], |o| o.ignored);
    kids.iter().filter(|k| !ignored.contains(&k.kind())).count()
}

/// The parameter list a unit declares: its `parameters` field, else
/// the C family's declarator chain, else a child of one of `kinds` —
/// or, for a Java record's compact constructor, which spells no list,
/// the record header's: its components are the constructor's
/// implicitly declared parameters (JLS 8.10.4.2).
fn param_list<'t>(node: Node<'t>, kinds: &[&str]) -> Option<Node<'t>> {
    node.child_by_field_name("parameters")
        .or_else(|| chain_params(node))
        .or_else(|| child_of_kinds(node, kinds))
        .or_else(|| record_header(node))
}

fn record_header(node: Node<'_>) -> Option<Node<'_>> {
    if node.kind() != "compact_constructor_declaration" {
        return None;
    }
    node.parent()?.parent()?.child_by_field_name("parameters")
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
