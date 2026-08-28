//! Python: the leading-underscore convention IS the language's
//! visibility rule for a name — and at module level `__all__`, when
//! the module declares one as a literal, is the export mechanism that
//! names its declarations (plan v2.17 L round step 8, O56): `from m
//! import *` and every API reader honour it, so a module-level
//! declaration is exported exactly when `__all__` lists it (an
//! underscore name it lists included, a public name it omits
//! excluded). Without a readable `__all__` the convention stands.
//! Read off the file's own root, never resolved: the top-level
//! assignments `__all__ = [...]` / `= (...)` and `__all__ += [...]`
//! union their string literals. Anything else that spells the name
//! is unreadable and the convention answers — a non-literal
//! right-hand side (`__all__ = other.__all__`, a comprehension), a
//! mutation (`__all__.extend(...)`, `.append`), a rebind under an
//! `if` or a loop, an f-string or escaped entry — a floor, never an
//! invention (the step-8 review's three counterexamples, each of
//! which had narrowed bit 0 on a name the module really exports).
//! Nested declarations (a method, an inner def) are not module
//! names, so `__all__` never speaks for them; dunder names
//! (`__init__`) are protocol, not private.

use super::{ancestors, name_text, root_of, text};
use crate::scan::ast;
use std::collections::BTreeSet;
use tree_sitter::Node;

pub(super) fn exported(node: Node<'_>, src: &[u8]) -> bool {
    let Some(name) = name_text(node, src) else {
        return false;
    };
    let module_level =
        !ancestors(node).any(|a| matches!(a.kind(), "class_definition" | "function_definition"));
    match (module_level, all_names(root_of(node), src)) {
        (true, Some(all)) => all.contains(&name),
        _ => public_by_convention(&name),
    }
}

/// The convention on one name: a leading underscore marks it
/// internal, a dunder is protocol.
pub(super) fn public_by_convention(name: &str) -> bool {
    !name.starts_with('_') || name.starts_with("__") && name.ends_with("__")
}

/// The scope chain: a `def` inside a `def` is the outer one's local,
/// and a method of a `_Private` class is as hidden as the class —
/// public by the same convention at every enclosing class.
pub(super) fn scope_open(node: Node<'_>, src: &[u8]) -> bool {
    ancestors(node).all(|a| match a.kind() {
        "function_definition" => false,
        "class_definition" => name_text(a, src).is_some_and(|n| public_by_convention(&n)),
        _ => true,
    })
}

/// The module's literal `__all__`, or None when it declares none or
/// declares it unreadably (module doc): every spelling of the name
/// the top-level scan did not consume — a mutation, a guarded or
/// looped rebind, a mention in a docstring — makes the list
/// unreadable, which errs only toward the convention's wider floor.
fn all_names(root: Node<'_>, src: &[u8]) -> Option<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    let mut read = 0usize;
    for stmt in ast::named_children(root) {
        let Some(rhs) = all_assignment(stmt, src) else {
            continue;
        };
        read += 1;
        names.extend(string_literals(rhs, src)?);
    }
    let spelled = src.windows(7).filter(|w| *w == b"__all__").count();
    (read > 0 && read == spelled).then_some(names)
}

/// The right-hand side of a top-level `__all__ = …` / `__all__ += …`.
fn all_assignment<'t>(stmt: Node<'t>, src: &[u8]) -> Option<Node<'t>> {
    if stmt.kind() != "expression_statement" {
        return None;
    }
    let assign = stmt.named_child(0)?;
    if !matches!(assign.kind(), "assignment" | "augmented_assignment") {
        return None;
    }
    let left = assign.child_by_field_name("left")?;
    (left.kind() == "identifier" && text(left, src) == "__all__")
        .then(|| assign.child_by_field_name("right"))
        .flatten()
}

/// Every string literal of a list or tuple; None when the value is
/// any other shape or any item is not a plain literal.
fn string_literals(value: Node<'_>, src: &[u8]) -> Option<Vec<String>> {
    if !matches!(value.kind(), "list" | "tuple") {
        return None;
    }
    ast::named_children(value)
        .into_iter()
        .map(|item| literal_text(item, src))
        .collect()
}

/// One item as the name it evaluates to: a plain literal's content.
/// An f-string (`interpolation` child) or an escape sequence (a named
/// child of the content) is not decoded here — None, unreadable —
/// and an implicit concatenation is not a `string` node at all.
fn literal_text(item: Node<'_>, src: &[u8]) -> Option<String> {
    if item.kind() != "string" {
        return None;
    }
    let mut out = String::new();
    for part in ast::named_children(item) {
        match part.kind() {
            "string_start" | "string_end" => {}
            "string_content" if part.named_child_count() == 0 => out.push_str(&text(part, src)),
            _ => return None,
        }
    }
    (!out.is_empty()).then_some(out)
}
