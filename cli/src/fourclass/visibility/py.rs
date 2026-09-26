//! Python: the leading-underscore convention IS the language's
//! visibility rule for a name — and at module level `__all__`, when
//! the module declares one in literals, is the export mechanism that
//! names its declarations (plan v2.17 L round step 8, O56): `from m
//! import *` and every API reader honour it, so a module-level
//! declaration is exported exactly when `__all__` lists it (an
//! underscore name it lists included, a public name it omits
//! excluded). Without a readable `__all__` the convention stands.
//! Read off the file's own root, never resolved: the top-level
//! statements `__all__ = [...]` / `= (...)`, `__all__ += [...]`,
//! `__all__.extend([...])` and `__all__.append("x")` union their
//! string literals with their escapes decoded (plan v2.30 step 5b
//! closed the two mutation forms and the escaped entry). Anything
//! else that spells the name as an identifier is unreadable and the
//! convention answers — a non-literal right-hand side (`__all__ =
//! other.__all__`, a comprehension), a rebind under an `if` or a
//! loop, an f-string entry, a `\N{…}` escape — a floor, never an
//! invention (the step-8 review's counterexamples, each of which had
//! narrowed bit 0 on a name the module really exports). A docstring
//! or a comment mentioning `__all__` is prose, not a spelling. Nested
//! declarations (a method, an inner def) are not module names, so
//! `__all__` never speaks for them; dunder names (`__init__`) are
//! protocol, not private.

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
fn public_by_convention(name: &str) -> bool {
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

/// One top-level statement's contribution to `__all__`: a list of
/// entries, or one entry.
enum Edit<'t> {
    Many(Node<'t>),
    One(Node<'t>),
}

/// The module's literal `__all__`, or None when it declares none or
/// declares it unreadably (module doc): every identifier spelling the
/// name must be one the top-level scan consumed — the target of an
/// assignment, the object of an `extend` / `append` call — or the
/// list is unreadable, which errs only toward the convention's wider
/// floor.
fn all_names(root: Node<'_>, src: &[u8]) -> Option<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    let mut read = 0usize;
    for stmt in ast::named_children(root) {
        let Some(edit) = all_edit(stmt, src) else {
            continue;
        };
        read += 1;
        names.extend(match edit {
            Edit::Many(value) => string_literals(value, src)?,
            Edit::One(value) => vec![literal_text(value, src)?],
        });
    }
    (read > 0 && read == spellings(root, src)).then_some(names)
}

/// A top-level `__all__ = …` / `__all__ += …` (the value is the
/// list), `__all__.extend(…)` (the one argument is the list) or
/// `__all__.append(…)` (the one argument is an entry).
fn all_edit<'t>(stmt: Node<'t>, src: &[u8]) -> Option<Edit<'t>> {
    if stmt.kind() != "expression_statement" {
        return None;
    }
    let expr = stmt.named_child(0)?;
    match expr.kind() {
        "assignment" | "augmented_assignment" => {
            let left = expr.child_by_field_name("left")?;
            (left.kind() == "identifier" && text(left, src) == "__all__")
                .then(|| expr.child_by_field_name("right").map(Edit::Many))
                .flatten()
        }
        "call" => call_edit(expr, src),
        _ => None,
    }
}

/// `__all__.extend(list)` / `__all__.append(entry)` with exactly one
/// argument; any other callee or arity is no edit.
fn call_edit<'t>(expr: Node<'t>, src: &[u8]) -> Option<Edit<'t>> {
    let function = expr.child_by_field_name("function")?;
    let object = function.child_by_field_name("object")?;
    if object.kind() != "identifier" || text(object, src) != "__all__" {
        return None;
    }
    let [arg] = ast::entries(expr.child_by_field_name("arguments")?)[..] else {
        return None;
    };
    match text(function.child_by_field_name("attribute")?, src).as_str() {
        "extend" => Some(Edit::Many(arg)),
        "append" => Some(Edit::One(arg)),
        _ => None,
    }
}

/// Every identifier in the file spelling `__all__` — a docstring's or
/// a comment's mention is no identifier.
fn spellings(root: Node<'_>, src: &[u8]) -> usize {
    ast::preorder(root, |_| true, ast::children)
        .into_iter()
        .filter(|n| n.kind() == "identifier" && text(*n, src) == "__all__")
        .count()
}

/// Every string literal of a list or tuple; None when the value is
/// any other shape or any item is not a readable literal.
fn string_literals(value: Node<'_>, src: &[u8]) -> Option<Vec<String>> {
    if !matches!(value.kind(), "list" | "tuple") {
        return None;
    }
    ast::named_children(value)
        .into_iter()
        .map(|item| literal_text(item, src))
        .collect()
}

/// One item as the name it evaluates to: a plain literal's content
/// with its escapes decoded. An f-string (`interpolation` child), an
/// implicit concatenation (no `string` node at all) or an escape no
/// table decodes is None, unreadable.
fn literal_text(item: Node<'_>, src: &[u8]) -> Option<String> {
    if item.kind() != "string" {
        return None;
    }
    let mut out = String::new();
    for part in ast::named_children(item) {
        match part.kind() {
            "string_start" | "string_end" => {}
            "string_content" => out.push_str(&content_text(part, src)?),
            _ => return None,
        }
    }
    (!out.is_empty()).then_some(out)
}

/// A string_content's text with each `escape_sequence` child decoded
/// and the plain runs between them kept; any other child (an
/// interpolation) is unreadable.
fn content_text(part: Node<'_>, src: &[u8]) -> Option<String> {
    let mut out = String::new();
    let mut at = part.start_byte();
    for child in ast::named_children(part) {
        if child.kind() != "escape_sequence" {
            return None;
        }
        out.push_str(std::str::from_utf8(&src[at..child.start_byte()]).ok()?);
        out.push_str(&unescape(
            std::str::from_utf8(&src[child.byte_range()]).ok()?,
        )?);
        at = child.end_byte();
    }
    out.push_str(std::str::from_utf8(&src[at..part.end_byte()]).ok()?);
    Some(out)
}

/// One Python escape sequence's value (the language reference,
/// §2.4.1 "String and Bytes literals"): the single-character escapes,
/// octal, `\x`, `\u` and `\U`; a backslash before a newline is a line
/// continuation and spells nothing, and `\N{name}` needs the Unicode
/// name table — unreadable.
fn unescape(seq: &str) -> Option<String> {
    const SIMPLE: [(&str, char); 10] = [
        ("\\", '\\'),
        ("'", '\''),
        ("\"", '"'),
        ("a", '\x07'),
        ("b", '\x08'),
        ("f", '\x0c'),
        ("n", '\n'),
        ("r", '\r'),
        ("t", '\t'),
        ("v", '\x0b'),
    ];
    let body = seq.strip_prefix('\\')?;
    if matches!(body, "\n" | "\r\n") {
        return Some(String::new());
    }
    if let Some((_, c)) = SIMPLE.iter().find(|(spelling, _)| *spelling == body) {
        return Some(c.to_string());
    }
    let code = match body.as_bytes().first()? {
        b'x' | b'u' | b'U' => u32::from_str_radix(&body[1..], 16).ok()?,
        b'0'..=b'7' => u32::from_str_radix(body, 8).ok()?,
        _ => return None,
    };
    Some(char::from_u32(code)?.to_string())
}
