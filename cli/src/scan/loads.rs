//! The import family's two readings (v2.35 imports lane): its
//! declarations (`import_declaration` below) and its load statements.
//! A load statement is a statement that is nothing
//! but loads bound to names — Lua `local x = require("m")`, R
//! `library(x)`, CommonJS `const x = require("m")`, Python
//! `m = importlib.import_module("x")`. With the import declarations
//! (`[scan] import_kinds`) they are name plumbing, and the clone
//! tokenizer gives them no token (dedup/tokens.rs). The shapes are the
//! package's (`[scan] loads`, CE.Lang.<Language>); a load is a call the
//! call-site table opens a site for (scan/call_site.rs) — the one
//! reading of the graph's call sites. The target side of an assignment
//! is not inspected: `M.x = require("x")` is a load statement.

use super::ast;
use super::call_site;
use super::lang::Lang;
use super::spec::LangSpec;
use tree_sitter::Node;

/// Whether `node` is a declaration of the import family: a NAMED node
/// matching an `import_kinds` entry — `<kind>`, `<kind> <field>` (it
/// has a child under that field: TS `export_statement source`, the
/// re-export) or `<kind> !<field>` (it has none: Rust `mod_item !body`,
/// the module mount). Haskell's `foreign import` keyword is an unnamed
/// node of kind `import` and is no declaration.
pub fn import_declaration(node: Node<'_>, spec: &LangSpec) -> bool {
    node.is_named()
        && spec.import_kinds.iter().any(|entry| {
            let (kind, field) = entry.split_once(' ').unwrap_or((entry, ""));
            kind == node.kind()
                && match field.strip_prefix('!') {
                    Some(absent) => node.child_by_field_name(absent).is_none(),
                    None => field.is_empty() || node.child_by_field_name(field).is_some(),
                }
        })
}

/// Whether `node` — a child of a statement list (`loads.statement_lists`)
/// — is a load statement: one of the shapes holds for it.
pub fn is_load(node: Node<'_>, src: &[u8], lang: Lang, spec: &LangSpec) -> bool {
    spec.loads
        .shapes
        .iter()
        .any(|shape| holds(node, shape, src, lang))
}

/// A shape is `<statement kind> <position>…`: every node a position
/// reaches continues the path and at least one must, and every node at
/// the path's end is a call that opens a site. A shape with no position
/// is the call itself.
fn holds(node: Node<'_>, shape: &str, src: &[u8], lang: Lang) -> bool {
    let mut steps = shape.split(' ');
    if steps.next() != Some(node.kind()) {
        return false;
    }
    let mut level = vec![node];
    for step in steps {
        let mut next = Vec::new();
        for n in &level {
            let reached = at(*n, step);
            if reached.is_empty() {
                return false;
            }
            next.extend(reached);
        }
        level = next;
    }
    level
        .iter()
        .all(|end| call_site::opens_site(*end, src, lang))
}

/// A position, read the way the cognitive tables read a body position
/// (`coc_nesting_kinds`): the children under that field, else the named
/// children of that kind.
fn at<'t>(node: Node<'t>, step: &str) -> Vec<Node<'t>> {
    let mut cursor = node.walk();
    let fielded: Vec<Node<'t>> = node.children_by_field_name(step, &mut cursor).collect();
    if !fielded.is_empty() {
        return fielded;
    }
    ast::named_children(node)
        .into_iter()
        .filter(|c| c.kind() == step)
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/scan/loads.rs"]
mod tests;
