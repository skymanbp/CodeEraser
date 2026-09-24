//! Java's `type_ref` sites (plan v2.30 step 3; booklet §8, register
//! D17): every place a Java file names a type — a type position (a
//! `type_identifier`, or a qualified `a.b.C` as ONE site whose segments
//! open none), an annotation's name, a record pattern's type, the
//! service types of a module's `uses` / `provides` directives (a
//! provider is often named nowhere else), and in expression position a
//! receiver spelled like a type (`Util.f()`, `Mode.FAST`, `Util::f`) or
//! a class named through its package (`a.b.C.f()`) — Java's naming
//! convention (JLS 6.1) is the only syntax telling a class from a
//! variable there. Excluded are the names the file itself declares:
//! its types at every depth and its type parameters, which reach no
//! other file, and — for an expression, where a variable obscures a
//! type of the same name (JLS 6.4.2) — its variables; `var` is no type
//! (JLS 14.4.1). The exclusion reads this file alone, so detection
//! stays resolution-free. Without these sites a class reached only from
//! its own package, which needs no import, would read unreferenced.

use super::{RawSite, site};
use crate::scan::ast;
use crate::scan::lang::Lang;
use crate::scan::spec;
use std::collections::BTreeSet;
use tree_sitter::Node;

/// The names a file declares, by JLS 6.5's two namespaces.
struct Own {
    types: BTreeSet<String>,
    vars: BTreeSet<String>,
}

/// Every type reference of one file, in document order.
pub(super) fn type_refs(root: Node<'_>, src: &[u8]) -> Vec<RawSite> {
    let own = Own::of(root, src);
    ast::preorder(root, |_| true, parts)
        .into_iter()
        .flat_map(|node| references(node, src, &own))
        .filter_map(|named| {
            let spec = spec_of(named, src)?;
            let head = spec.split('.').next().unwrap_or("").trim();
            let foreign = spec != "var" && !own.types.contains(head);
            foreign.then(|| site("type_ref", named, spec))
        })
        .collect()
}

/// A node's children for the walk — except that a qualified type is one
/// site, so of its parts only those naming other types are walked: a
/// generic segment's type arguments and a type annotation.
fn parts(node: Node<'_>) -> Vec<Node<'_>> {
    if node.kind() != "scoped_type_identifier" {
        return ast::children(node);
    }
    ast::named_children(node)
        .into_iter()
        .flat_map(|part| match part.kind() {
            "scoped_type_identifier" => parts(part),
            "generic_type" => ast::named_children(part)
                .into_iter()
                .filter(|g| g.kind() == "type_arguments")
                .collect(),
            "marker_annotation" | "annotation" => vec![part],
            _ => Vec::new(),
        })
        .collect()
}

/// The nodes naming a type at `node`.
fn references<'t>(node: Node<'t>, src: &[u8], own: &Own) -> Vec<Node<'t>> {
    let field = |name: &str| node.child_by_field_name(name).into_iter().collect();
    match node.kind() {
        "type_identifier" | "scoped_type_identifier" => vec![node],
        "marker_annotation" | "annotation" => field("name"),
        "uses_module_directive" => field("type"),
        "provides_module_directive" => ast::named_children(node),
        // a generic record pattern's type is a generic_type, walked on
        "record_pattern" => ast::named_children(node)
            .into_iter()
            .take(1)
            .filter(|t| t.kind() != "generic_type")
            .collect(),
        "method_invocation" | "field_access" | "method_reference" => accessed(node, src, own),
        _ => Vec::new(),
    }
}

/// A member access's type: a receiver spelled like a type that no
/// variable of the file obscures, or — a field access only — a class
/// named through a lowercase package chain.
fn accessed<'t>(node: Node<'t>, src: &[u8], own: &Own) -> Vec<Node<'t>> {
    let object = match node.kind() {
        "method_reference" => node.named_child(0),
        _ => node.child_by_field_name("object"),
    };
    let Some(object) = object else {
        return Vec::new();
    };
    let free = |n: Node<'_>| !own.vars.contains(n.utf8_text(src).unwrap_or(""));
    if object.kind() == "identifier" && typed(object, src) && free(object) {
        return vec![object];
    }
    let qualified = node.kind() == "field_access"
        && node
            .child_by_field_name("field")
            .is_some_and(|f| typed(f, src))
        && package_head(object, src).is_some_and(free);
    if qualified { vec![node] } else { Vec::new() }
}

/// The head of a dotted chain of lowercase names (`a.b.c`) — a package
/// name's shape in expression position.
fn package_head<'t>(node: Node<'t>, src: &[u8]) -> Option<Node<'t>> {
    let lower = |n: Node<'_>| n.kind() == "identifier" && !typed(n, src);
    match node.kind() {
        "identifier" => lower(node).then_some(node),
        "field_access" if lower(node.child_by_field_name("field")?) => {
            package_head(node.child_by_field_name("object")?, src)
        }
        _ => None,
    }
}

/// Spelled like a type (JLS 6.1): the first letter past any `$` / `_`
/// is upper case.
fn typed(ident: Node<'_>, src: &[u8]) -> bool {
    ident
        .utf8_text(src)
        .ok()
        .and_then(|t| t.trim_start_matches(['$', '_']).chars().next())
        .is_some_and(char::is_uppercase)
}

/// A site's spec: its first line, up to a generic segment's type
/// arguments — `Outer<String>.Inner` names a member type of Outer, which
/// lives in Outer's file — so the spec stays a substring of its line.
fn spec_of(named: Node<'_>, src: &[u8]) -> Option<String> {
    let text = named.utf8_text(src).ok()?;
    let name = text.split(['\n', '<']).next()?.trim();
    let name = name.trim_end_matches('.').trim_end();
    (!name.is_empty()).then(|| name.to_string())
}

impl Own {
    fn of(root: Node<'_>, src: &[u8]) -> Self {
        let types = spec::spec(Lang::Java).owner_kinds;
        let text = |n: Node<'_>| n.utf8_text(src).ok().map(str::to_string);
        let mut own = Own {
            types: BTreeSet::new(),
            vars: BTreeSet::new(),
        };
        for node in ast::preorder(root, |_| true, ast::children) {
            match node.kind() {
                "type_parameter" => own.types.extend(
                    ast::named_children(node)
                        .into_iter()
                        .find(|c| c.kind() == "type_identifier")
                        .and_then(text),
                ),
                kind if types.contains(&kind) => own
                    .types
                    .extend(node.child_by_field_name("name").and_then(text)),
                _ => own
                    .vars
                    .extend(variables(node).into_iter().filter_map(text)),
            }
        }
        own
    }
}

/// The variables a node declares (JLS 4.12.3's kinds: fields and
/// locals, parameters of every sort, enum constants, resources and
/// pattern bindings).
fn variables(node: Node<'_>) -> Vec<Node<'_>> {
    match node.kind() {
        "variable_declarator"
        | "formal_parameter"
        | "catch_formal_parameter"
        | "resource"
        | "enum_constant"
        | "instanceof_expression" => node.child_by_field_name("name").into_iter().collect(),
        "inferred_parameters" => ast::named_children(node),
        "lambda_expression" => node
            .child_by_field_name("parameters")
            .filter(|p| p.kind() == "identifier")
            .into_iter()
            .collect(),
        "record_pattern_component" => ast::named_children(node)
            .into_iter()
            .filter(|c| c.kind() == "identifier")
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/sites/java.rs"]
mod tests;
