//! The C family's declarator road (plan v2.30 step 2): what makes a
//! `function_definition` a unit, and how it spells its name. Split
//! from functions.rs, which stays the grammar-neutral extraction root
//! and calls in here for the one grammar family that names a
//! definition through nested declarators instead of a `name` field.
//!
//! tree-sitter-cpp's recovery from an unexpanded macro leaves
//! `function_definition` nodes that are no functions (the fmt/format.h
//! transcript, crosscheck 2026-09-24): `FMT_BEGIN_NAMESPACE` alone on a
//! line before `namespace detail {` reads as a definition named
//! `namespace`; a `struct` folded into a definition's type reads as one
//! with an ERROR node for a declarator; `bigint(const bigint&) =
//! delete;` is the definition kind with a delete_method_clause and no
//! body; and `FMT_CATCH(...) { ... }` inside a body reads as a nested
//! definition. So `defined` asks four things of a node the kind table
//! admitted: a body, a parameter list somewhere in its declarator
//! chain, a name leaf at the chain's end, and no unit-shaped definition
//! enclosing it before the nearest scope opener — a nested definition
//! (a macro block in C++, a GNU nested function in C) is absorbed into
//! its host the way a lambda is, while a local class's member, whose
//! climb meets the class first, stays a unit of its own.

use super::ast;
use tree_sitter::Node;

/// The kinds a declarator chain may end in and still name a function.
/// An ERROR leaf, or a type folded in by recovery, names nothing.
const NAME_LEAVES: [&str; 7] = [
    "identifier",
    "field_identifier",
    "qualified_identifier",
    "destructor_name",
    "operator_name",
    "template_function",
    "operator_cast",
];

/// Ancestors that open a scope of their own: reaching one before a
/// definition means the node is not nested in a function.
const SCOPE_OPENERS: [&str; 8] = [
    "class_specifier",
    "struct_specifier",
    "union_specifier",
    "field_declaration_list",
    "namespace_definition",
    "declaration_list",
    "linkage_specification",
    "translation_unit",
];

/// A node that names itself through a `declarator` field is a unit
/// when it is shaped like a definition and not absorbed; a node with
/// no such field (every other grammar) is left to the kind table.
pub(super) fn defined(node: Node<'_>, src: &[u8]) -> bool {
    node.child_by_field_name("declarator").is_none() || (shaped(node, src) && !absorbed(node, src))
}

/// Body, parameter list, name leaf, and a type or a special member's
/// name — the definition shape.
fn shaped(node: Node<'_>, src: &[u8]) -> bool {
    node.child_by_field_name("body").is_some()
        && chain(node)
            .is_some_and(|(leaf, params)| params.is_some() && NAME_LEAVES.contains(&leaf.kind()))
        && (node.child_by_field_name("type").is_some() || special(node, src))
}

/// A definition with no `type` field is a constructor, a destructor or
/// a conversion operator — the three C++ lets go without a return type,
/// each carrying the class's own name or `~` / `operator` — or an
/// unexpanded macro wearing a parameter list and a block
/// (`FMT_CATCH(...) { }` at class scope), which carries its own. Only
/// a definition INSIDE a class can be told apart that way; one with no
/// owner keeps the benefit of the doubt, since recovery may have
/// folded its class away (format.h 4126: a constructor whose class
/// body was read as a block). C never reaches here: tree-sitter-c
/// reads an implicit-int `main() {}` as an ERROR, not as the
/// definition kind (probe 2026-09-24).
fn special(node: Node<'_>, src: &[u8]) -> bool {
    let Some((owner, base)) = identity(node, src) else {
        return false;
    };
    base.starts_with('~')
        || base.starts_with("operator ")
        || owner.is_none_or(|o| o.rsplit("::").next() == Some(base.as_str()))
}

/// Enclosed by a unit-shaped definition or a lambda before any scope
/// opener: a nested definition, absorbed into its host.
fn absorbed(node: Node<'_>, src: &[u8]) -> bool {
    let mut here = node.parent();
    while let Some(p) = here {
        if SCOPE_OPENERS.contains(&p.kind()) {
            return false;
        }
        if p.kind() == "lambda_expression" {
            return true;
        }
        if p.kind() == "function_definition" {
            return shaped(p, src);
        }
        here = p.parent();
    }
    false
}

/// Walks a node's `declarator` field down to the leaf and returns it
/// with the LAST function_declarator passed — the one carrying the
/// function's own parameters (`int (*maker(int n))(int)` wraps `maker`
/// in a function_declarator, a pointer_declarator and a parenthesized
/// one before the returned pointer's own list). Every chain kind but
/// two carries a `declarator` field (AST-probed: function / pointer /
/// array / init declarators); parenthesized_declarator and
/// reference_declarator hold their inner declarator as an unnamed
/// child. A conversion operator (`operator bool() const`) is an
/// operator_cast leaf whose abstract_function_declarator child is its
/// list. None where the node has no declarator field.
pub(crate) fn chain(node: Node<'_>) -> Option<(Node<'_>, Option<Node<'_>>)> {
    let mut here = node.child_by_field_name("declarator")?;
    let mut params = None;
    loop {
        match here.kind() {
            "function_declarator" => params = Some(here),
            "operator_cast" => return Some((here, here.child_by_field_name("declarator"))),
            _ => {}
        }
        let inner = here.child_by_field_name("declarator").or_else(|| {
            matches!(
                here.kind(),
                "parenthesized_declarator" | "reference_declarator"
            )
            .then(|| ast::named_children(here).into_iter().next())
            .flatten()
        });
        match inner {
            Some(inner) => here = inner,
            None => return Some((here, params)),
        }
    }
}

/// (owner, base) of a node that names itself through a declarator
/// chain; None where the node has no `declarator` field. A leaf that
/// spells its own qualifier (`K::b`) is the authority; a bare leaf
/// takes the enclosing class / struct / union chain.
pub(crate) fn identity(node: Node<'_>, src: &[u8]) -> Option<(Option<String>, String)> {
    let (leaf, _) = chain(node)?;
    if leaf.kind() == "qualified_identifier" {
        return Some(split_qualified(leaf, src));
    }
    let mut base = bare(leaf, src);
    if dropped_tilde(node, src) {
        base.insert(0, '~');
    }
    Some((class_chain(node, src), base))
}

/// `FMT_CONSTEXPR20 ~K() { }`: the macro reads as the return type and
/// the parser drops the `~` into an ERROR child between it and the
/// declarator (format.h 984), so the destructor would spell its class's
/// name. A lone `~` in that position is the tilde put back.
fn dropped_tilde(node: Node<'_>, src: &[u8]) -> bool {
    ast::children(node)
        .into_iter()
        .any(|c| c.is_error() && c.utf8_text(src) == Ok("~"))
}

/// `A::B::c` parses as qualified_identifier{scope: A, name:
/// qualified_identifier{scope: B, name: c}} (AST-probed): the scopes
/// joined are the owner, the innermost name is the base. A leading
/// `::` (the global qualifier) has no scope and owns nothing.
fn split_qualified(node: Node<'_>, src: &[u8]) -> (Option<String>, String) {
    let mut scopes = Vec::new();
    let mut here = node;
    while here.kind() == "qualified_identifier" {
        if let Some(scope) = here.child_by_field_name("scope") {
            scopes.push(bare(scope, src));
        }
        match here.child_by_field_name("name") {
            Some(name) => here = name,
            None => break,
        }
    }
    let owner = (!scopes.is_empty()).then(|| scopes.join("::"));
    (owner, bare(here, src))
}

/// The named class_specifier / struct_specifier / union_specifier
/// ancestors, outermost first. Namespaces are deliberately not in the
/// chain: a namespace body is a lexical scope bare names resolve in,
/// not a member scope (spec_c.rs call_member_scopes), so a definition
/// inside one spells the same name it would at the top level.
fn class_chain(node: Node<'_>, src: &[u8]) -> Option<String> {
    let mut names = Vec::new();
    let mut here = node.parent();
    while let Some(p) = here {
        if matches!(
            p.kind(),
            "class_specifier" | "struct_specifier" | "union_specifier"
        ) && let Some(name) = p.child_by_field_name("name")
        {
            names.push(bare(name, src));
        }
        here = p.parent();
    }
    if names.is_empty() {
        return None;
    }
    names.reverse();
    Some(names.join("::"))
}

/// A name node's spelling: the identifier of a template_type or
/// template_function — `Box<T>::b` and the specialization `spec<int>`
/// spell `Box::b` and `spec`, since the arguments are not the name the
/// language looks up and a partial specialization's list runs over
/// lines — `operator ` + the type for a conversion operator, and
/// otherwise the text with its whitespace runs collapsed, where an
/// operator_name keeps one space before a word (`operator new[]`) and
/// none before a symbol (`operator ()`).
fn bare(node: Node<'_>, src: &[u8]) -> String {
    let spelled = match node.kind() {
        "template_type" | "template_function" => {
            text(node.child_by_field_name("name").unwrap_or(node), src)
        }
        "operator_cast" => {
            let ty = node
                .child_by_field_name("type")
                .map_or(String::new(), |t| text(t, src));
            format!("operator {ty}")
        }
        _ => text(node, src),
    };
    match spelled.strip_prefix("operator ") {
        Some(rest) if !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_') => {
            format!("operator{rest}")
        }
        _ => spelled,
    }
}

/// A node's text with whitespace runs collapsed to one space.
fn text(node: Node<'_>, src: &[u8]) -> String {
    node.utf8_text(src)
        .unwrap_or("(non-utf8)")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
