//! Function-unit extraction from a parsed tree, driven by LangSpec.
//! A "function unit" is a node whose kind is in `fn_kinds`; nested
//! standalone units are measured separately and skipped inside their
//! host (see metrics walkers).

use super::ast;
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
/// node carries a `name` field.
pub fn is_unit_node(node: Node<'_>, spec: &LangSpec) -> bool {
    spec.fn_kinds.contains(&node.kind())
        && (!spec.fn_named_only_kinds.contains(&node.kind())
            || node.child_by_field_name("name").is_some())
}

pub fn extract<'t>(root: Node<'t>, src: &[u8], spec: &LangSpec) -> Vec<FnUnit<'t>> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if is_unit_node(node, spec) {
            out.push(unit(node, src, spec));
        }
        stack.extend(ast::children(node).into_iter().rev());
    }
    out.sort_by_key(|f| f.start_line);
    out
}

fn unit<'t>(node: Node<'t>, src: &[u8], spec: &LangSpec) -> FnUnit<'t> {
    FnUnit {
        node,
        name: name_of(node, src),
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        params: param_count(node, spec),
    }
}

/// Name from the node's `name` field, else from an enclosing
/// variable_declarator/pair (arrow functions), else "(anonymous)".
fn name_of(node: Node<'_>, src: &[u8]) -> String {
    if let Some(name) = node.child_by_field_name("name") {
        return text(name, src);
    }
    if let Some(parent) = node.parent()
        && matches!(parent.kind(), "variable_declarator" | "pair" | "assignment")
        && let Some(name) = parent.child_by_field_name(field_for(parent.kind()))
    {
        return text(name, src);
    }
    "(anonymous)".to_string()
}

fn field_for(parent_kind: &str) -> &'static str {
    match parent_kind {
        "pair" => "key",
        "assignment" => "left",
        _ => "name",
    }
}

/// KNOWN DEFECT (M5-3h blind audit, auditor B; probe-verified
/// 2026-08-14): Go method_declaration's RECEIVER is itself a
/// parameter_list, so the first-of-kind scan below counts the
/// receiver (always 1), never the parameters — `func (t T) add(x
/// int, y string)` keys as `(T) add/1`, colliding arities. The fix
/// (prefer the `parameters` FIELD before the kind scan) changes
/// unitsig/symbols identity keys and cascades into the frozen Go
/// universes (cobra), so it ships with a rev bump + re-freeze batch,
/// not as a drive-by here.
fn param_count(node: Node<'_>, spec: &LangSpec) -> usize {
    let Some(params) = child_of_kinds(node, spec.param_list_kinds) else {
        return 0;
    };
    ast::named_children(params)
        .into_iter()
        .filter(|c| !c.kind().contains("comment"))
        .count()
}

fn child_of_kinds<'t>(node: Node<'t>, kinds: &[&str]) -> Option<Node<'t>> {
    ast::children(node)
        .into_iter()
        .find(|c| kinds.contains(&c.kind()))
}

fn text(node: Node<'_>, src: &[u8]) -> String {
    node.utf8_text(src).unwrap_or("(non-utf8)").to_string()
}
