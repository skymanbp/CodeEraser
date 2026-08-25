//! The names an import brings into scope (plan v2.14, ADR-008 clause
//! three). One half of the symbol-edge input: this module reads the
//! BOUND NAMES off an import site's own syntax — which names an
//! import binds is a syntactic fact, not a guess, and that is the
//! whole reason symbol edges may ride import edges at all.
//!
//! What this module deliberately does NOT decide: whether a bound
//! name identifies a DECLARATION in the target file. Four adversarial
//! reviews (2026-08-24) showed the leaf of an import path is
//! syntactically indistinguishable between a declaration, a submodule
//! and a re-export — Python's `from . import certs` (a submodule)
//! has the shape of `from . import X` (declared in __init__.py), and
//! Rust's `use crate::{churn, dedup}` (modules) has the shape of
//! `use crate::config::Config` (an item). So every name here is a
//! CANDIDATE; the symbols table of the resolved target decides, and a
//! candidate that misses is a module or a re-export and yields no
//! symbol edge. One edge short beats one edge wrong — minting on
//! syntax alone is what killed R6.

use crate::scan::ast::{children, named_children};
use crate::scan::lang::Lang;
use tree_sitter::Node;

/// One name an import brings in. `local` is what the citing file
/// calls it; `target` is the name to look for in the imported file.
/// They differ under an alias, and BOTH are needed: use sites read
/// the local name, the symbols lookup reads the target name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub local: String,
    pub target: String,
}

impl Binding {
    fn same(name: String) -> Self {
        Binding {
            target: name.clone(),
            local: name,
        }
    }
}

/// Nesting cap for Rust use trees. Real code nests two or three deep;
/// the cap exists so a pathological file cannot recurse without end.
const MAX_USE_DEPTH: usize = 8;

/// Candidate bindings of one detected site. The node is whatever the
/// site table emitted at: a statement for most kinds, and for Python
/// `import a, b` the per-target CHILD (sites.rs EachImportTarget).
pub fn of_site(node: Node<'_>, src: &[u8], lang: Lang) -> Vec<Binding> {
    match (lang, node.kind()) {
        (Lang::Python, "import_from_statement") => py_from(node, src),
        // `import a.b [as c]` binds a module object, never a name
        // declared inside the target — no candidate to offer.
        (Lang::Python, _) => Vec::new(),
        (Lang::TypeScript | Lang::Tsx, "import_statement" | "export_statement") => {
            ts_specifiers(node, src)
        }
        (Lang::Rust, "use_declaration") => match node.child_by_field_name("argument") {
            Some(arg) => rust_use(arg, src, 0),
            None => Vec::new(),
        },
        (Lang::Haskell, "import") => hs_import(node, src),
        _ => Vec::new(),
    }
}

fn text(node: Node<'_>, src: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(&src[node.byte_range()])
        .trim()
        .to_string();
    (!s.is_empty()).then_some(s)
}

/// Python `from m import a, b as c`. The `name` field is MULTIPLE, so
/// every name-fielded child is a binding; `module_name` is a separate
/// field and never one, and a `wildcard_import` or an interleaved
/// `comment` carries no identifier at all.
fn py_from(node: Node<'_>, src: &[u8]) -> Vec<Binding> {
    let mut cursor = node.walk();
    node.children_by_field_name("name", &mut cursor)
        .filter_map(|child| match child.kind() {
            "dotted_name" => text(child, src).map(Binding::same),
            "aliased_import" => Some(Binding {
                local: text(child.child_by_field_name("alias")?, src)?,
                target: text(child.child_by_field_name("name")?, src)?,
            }),
            _ => None,
        })
        .collect()
}

/// TS/TSX `import { a, b as c } from "m"` and the re-export form
/// `export { a as b } from "m"`. Both hang their specifiers under one
/// clause; a `comment` between specifiers is a named child too, so
/// the kind test is on the specifier, never on position.
fn ts_specifiers(node: Node<'_>, src: &[u8]) -> Vec<Binding> {
    let mut out = Vec::new();
    let mut stack: Vec<Node<'_>> = named_children(node);
    while let Some(child) = stack.pop() {
        match child.kind() {
            "import_specifier" | "export_specifier" => {
                let Some(target) = child.child_by_field_name("name").and_then(|n| text(n, src))
                else {
                    continue;
                };
                let local = child
                    .child_by_field_name("alias")
                    .and_then(|n| text(n, src))
                    .unwrap_or_else(|| target.clone());
                out.push(Binding { local, target });
            }
            // the clause wrappers: import_clause / named_imports /
            // export_clause. Descend; anything else (the source
            // string, a namespace import, a default identifier)
            // carries no target-side name.
            "import_clause" | "named_imports" | "export_clause" => {
                stack.extend(named_children(child));
            }
            _ => {}
        }
    }
    out.reverse(); // document order: the stack visited them backwards
    out
}

/// Rust use trees. Every leaf that names something is a candidate;
/// `use crate::{a, b}` offers `a` and `b` exactly as
/// `use crate::a::B` offers `B`, and the symbols lookup is what tells
/// a module apart from an item afterwards.
fn rust_use(node: Node<'_>, src: &[u8], depth: usize) -> Vec<Binding> {
    if depth > MAX_USE_DEPTH {
        return Vec::new();
    }
    match node.kind() {
        "identifier" | "type_identifier" => {
            text(node, src).map(Binding::same).into_iter().collect()
        }
        "scoped_identifier" => node
            .child_by_field_name("name")
            .and_then(|n| text(n, src))
            .map(Binding::same)
            .into_iter()
            .collect(),
        "use_as_clause" => rust_alias(node, src, depth),
        "use_list" => named_children(node)
            .into_iter()
            .flat_map(|c| rust_use(c, src, depth + 1))
            .collect(),
        "scoped_use_list" => match node.child_by_field_name("list") {
            Some(list) => rust_use(list, src, depth + 1),
            None => Vec::new(),
        },
        // use_wildcard, `self`, `crate`, `super`: no target-side name
        _ => Vec::new(),
    }
}

/// `use a::b as c` — the alias is local, the path's leaf is the name
/// the target declares. `use x as _` binds nothing nameable.
fn rust_alias(node: Node<'_>, src: &[u8], depth: usize) -> Vec<Binding> {
    let Some(path) = node.child_by_field_name("path") else {
        return Vec::new();
    };
    let mut target = rust_use(path, src, depth + 1);
    let Some(first) = target.pop() else {
        return Vec::new();
    };
    match node.child_by_field_name("alias").and_then(|n| text(n, src)) {
        Some(alias) if alias != "_" => vec![Binding {
            local: alias,
            target: first.target,
        }],
        _ => Vec::new(),
    }
}

/// Haskell `import M (a, b)` — an explicit list names exactly those.
/// `import M hiding (a)` names the COMPLEMENT of a list, which is not
/// an enumerable candidate set, so it offers none; a headerless
/// `import M` offers none either (it brings in everything M exports,
/// again not enumerable from this file's syntax).
fn hs_import(node: Node<'_>, src: &[u8]) -> Vec<Binding> {
    // `hiding` is an ANONYMOUS token in this grammar (AST-probed
    // against the pinned tree-sitter-haskell), so it must be looked
    // for among ALL children — named_children filters it out, which
    // is how a first draft let a hiding-import offer candidates.
    if children(node).iter().any(|c| c.kind() == "hiding") {
        return Vec::new();
    }
    let Some(list) = named_children(node)
        .into_iter()
        .find(|c| c.kind() == "import_list")
    else {
        return Vec::new();
    };
    named_children(list)
        .into_iter()
        .filter_map(|entry| hs_entry(entry, src))
        .collect()
}

/// One import-list entry: a bare name, or a type with its
/// constructors (`Tree (..)`) whose binding is the type's own name.
fn hs_entry(entry: Node<'_>, src: &[u8]) -> Option<Binding> {
    let head = match entry.kind() {
        "import_name" | "import_item" => named_children(entry).into_iter().next()?,
        _ => entry,
    };
    let name = text(head, src)?;
    // an operator section like `(!)` and a constructor list like
    // `(..)` are punctuation, not names
    let first = name.chars().next()?;
    (first.is_alphabetic() || first == '_').then(|| Binding::same(name))
}

#[cfg(test)]
#[path = "bindings_tests.rs"]
mod tests;
