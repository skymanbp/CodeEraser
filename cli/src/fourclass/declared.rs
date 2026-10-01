//! Which names a registered declaration node declares (plan v2.30 step
//! 5b-6, the declaration domain widened by rule). The kind tables in
//! fourclass::kinds admit a node; this module answers the two questions
//! units.rs then asks of it — does it declare HERE, and which name nodes
//! does it bind — off the node and its ancestors in this file alone,
//! the way the visibility word is read (visibility/).
//!
//! The 2026-09-27 ruling, one clause per language:
//!   - Go: a `const_spec` / `var_spec` at package level (kinds::
//!     PACKAGE_LEVEL, step 5b item 25).
//!   - Java: a `static` field and an interface's or annotation type's
//!     constant (kinds::JAVA_FIELDS). An instance field is every
//!     object's own and stays out; an enum constant is reached by
//!     `values()` / `valueOf` without its name being spelled, a road
//!     the mention criterion cannot see, so it stays out as well (a row
//!     for it would be unsafe on the advisory face).
//!   - C / C++: a `declaration` at file scope that defines a variable
//!     (kinds::C_VARIABLE) — one unit per declarator that names no
//!     function (a prototype `int f(int);` is the header's own spelling
//!     of a name, booklet §6) and is no bare `extern` (a reference to a
//!     definition elsewhere, C11 6.9.2). A class member is the class's
//!     (`field_declaration`, never keyed); a static member's
//!     out-of-class definition `int K::count = 0;` is the file-scope
//!     declaration the rule admits, keyed as it is spelled.
//!   - TypeScript: a `const` / `let` / `var` at module level (kinds::
//!     TS_LEXICAL) — under the program, an `export` / `declare` wrapper
//!     or a namespace / module / `declare global` body — one unit per
//!     bound identifier, a destructuring pattern's included. A
//!     declarator whose function value the extractor already named
//!     after it (`const f = () => {}` is the unit `f/0`,
//!     functions::name_of) declares no second symbol; a class's static
//!     field is the class's own.
//!
//! Every shape read here was probed on the pinned grammars before the
//! reading was written (tree-sitter-java 0.23.5, -c 0.24.2, -cpp 0.23.4,
//! -typescript 0.23.2; the step's probe transcripts).

use super::kinds;
use super::visibility::java_modifiers;
use crate::scan::ast::ancestors;
use crate::scan::lang::Lang;
use crate::scan::{ast, binding, declarator, functions, spec};
use tree_sitter::Node;

/// The name nodes a registered declaration keys by — every `name`
/// field it carries (a Go `var a, b int` carries two), the leaf of a C
/// typedef's declarator chain, or the per-language reading below; none
/// where the node is not a declaration of its own (`declares`).
pub(super) fn keys<'t>(node: Node<'t>, table: &[&str], src: &[u8], lang: Lang) -> Vec<Node<'t>> {
    if !table.contains(&node.kind()) || !declares(node) {
        return Vec::new();
    }
    match node.kind() {
        "type_definition" => declarator::chain(node)
            .map(|(leaf, _)| leaf)
            .into_iter()
            .collect(),
        k if k == kinds::shared().c_variable => c_variables(node, src),
        k if kinds::shared().java_fields.contains(&k) => binding::fielded(node, "declarator")
            .into_iter()
            .filter_map(|d| d.child_by_field_name("name"))
            .collect(),
        k if kinds::shared().ts_lexical.contains(&k) => ts_bindings(node, src, lang),
        _ => binding::fielded(node, "name"),
    }
}

/// Whether a registered kind declares here: an instance of a family
/// names the family, never a new type (kinds::REDECLARING); a C
/// `struct K x;` spells K by the node kind that declares it and only
/// the form with a body declares (kinds::BODIED); a Go const or var
/// spec declares a cross-file name at package level alone; a Java
/// field when `static`; a C declaration at file scope; a TS lexical
/// form at module level (module doc).
fn declares(node: Node<'_>) -> bool {
    let kind = node.kind();
    let redeclares = node
        .parent()
        .is_some_and(|p| kinds::shared().redeclaring.contains(&p.kind()));
    let bodiless =
        kinds::shared().bodied.contains(&kind) && node.child_by_field_name("body").is_none();
    let local = kinds::shared().package_level.contains(&kind) && !package_level(node);
    let instance = kind == "field_declaration" && !java_static(node);
    let scoped = kind == kinds::shared().c_variable && !c_file_scope(node);
    let block = kinds::shared().ts_lexical.contains(&kind) && !ts_module_level(node);
    !(redeclares || bodiless || local || instance || scoped || block)
}

/// Go: the spec's grandparent is the file root — the `const` / `var`
/// declaration sits directly under `source_file`.
fn package_level(node: Node<'_>) -> bool {
    node.parent()
        .and_then(|p| p.parent())
        .is_some_and(|g| g.parent().is_none())
}

// ---- Java ----

/// `static` among the declaration's modifier tokens.
fn java_static(node: Node<'_>) -> bool {
    java_modifiers(node).iter().any(|m| m.kind() == "static")
}

// ---- C / C++ ----

/// Holders a file-scope declaration climbs through to the translation
/// unit: a namespace or linkage body and `extern "C"` itself, a
/// template head, and the preprocessor's conditionals, which group
/// declarations without scoping them.
const C_TRANSPARENT: [&str; 9] = [
    "declaration_list",
    "namespace_definition",
    "linkage_specification",
    "template_declaration",
    "preproc_if",
    "preproc_ifdef",
    "preproc_elif",
    "preproc_elifdef",
    "preproc_else",
];

/// The leaves a variable's declarator chain may end in: a name, an
/// out-of-class definition's qualified name, or a structured binding.
const C_VARIABLE_LEAVES: [&str; 3] = [
    "identifier",
    "qualified_identifier",
    "structured_binding_declarator",
];

/// The first ancestor that is no transparent holder is the translation
/// unit itself — not a function body, a class body or a loop head.
fn c_file_scope(node: Node<'_>) -> bool {
    ancestors(node)
        .find(|a| !C_TRANSPARENT.contains(&a.kind()))
        .is_some_and(|a| a.kind() == "translation_unit")
}

/// The variables one file-scope declaration defines: each `declarator`
/// field's chain leaf, skipping a declarator that names a function and,
/// under `extern`, one without an initializer; a structured binding
/// (`auto [x, y] = …`) binds each of its identifiers.
fn c_variables<'t>(node: Node<'t>, src: &[u8]) -> Vec<Node<'t>> {
    let extern_ = ast::children(node)
        .into_iter()
        .any(|c| c.kind() == "storage_class_specifier" && &src[c.byte_range()] == b"extern");
    binding::fielded(node, "declarator")
        .into_iter()
        .filter(|d| !extern_ || d.kind() == "init_declarator")
        .map(|d| declarator::chain_from(d).0)
        .filter(|leaf| C_VARIABLE_LEAVES.contains(&leaf.kind()) && !names_a_function(*leaf))
        .flat_map(|leaf| match leaf.kind() {
            "structured_binding_declarator" => ast::named_children(leaf)
                .into_iter()
                .filter(|c| c.kind() == "identifier")
                .collect(),
            _ => vec![leaf],
        })
        .collect()
}

/// Parentheses aside, the leaf's own wrapper is a function declarator:
/// the declarator names a function (`int f(int)`, `int *f(int)`, the
/// `signal` prototype), where a pointer or array wrapper names a
/// variable of function-pointer type (`int (*fp)(int)`).
fn names_a_function(leaf: Node<'_>) -> bool {
    ancestors(leaf)
        .find(|p| p.kind() != "parenthesized_declarator")
        .is_some_and(|p| p.kind() == "function_declarator")
}

// ---- TypeScript ----

/// Bodies whose declarations a file can name: a namespace's or a
/// module's, and a `declare global` block's.
const TS_OPEN_BODIES: [&str; 3] = ["internal_module", "module", "ambient_declaration"];

/// Module level: past the `export` / `declare` wrappers, the holder is
/// the program itself or an open body — a function body, a loop head,
/// a branch or a bare block binds locals.
fn ts_module_level(node: Node<'_>) -> bool {
    let holder =
        ancestors(node).find(|a| !matches!(a.kind(), "export_statement" | "ambient_declaration"));
    match holder.map(|h| h.kind()) {
        Some("program") => true,
        Some("statement_block") => holder
            .and_then(|h| h.parent())
            .is_some_and(|body| TS_OPEN_BODIES.contains(&body.kind())),
        _ => false,
    }
}

/// The identifiers the declaration binds, one per declarator the
/// function extractor has not already claimed.
fn ts_bindings<'t>(node: Node<'t>, src: &[u8], lang: Lang) -> Vec<Node<'t>> {
    let fn_kinds = spec::spec(lang).fn_kinds;
    ast::named_children(node)
        .into_iter()
        .filter(|d| d.kind() == "variable_declarator" && !taken_by_a_function(*d, src, fn_kinds))
        .filter_map(|d| d.child_by_field_name("name"))
        .flat_map(pattern_names)
        .collect()
}

/// `const f = () => {}`: the extractor names the arrow after the
/// declarator (functions::name_of), so the declaration is the unit
/// `f/0` already; `const g = function h() {}` keeps `h` for the
/// function and `g` is a binding of its own.
fn taken_by_a_function(decl: Node<'_>, src: &[u8], fn_kinds: &[&str]) -> bool {
    let Some(name) = decl.child_by_field_name("name") else {
        return false;
    };
    decl.child_by_field_name("value").is_some_and(|v| {
        fn_kinds.contains(&v.kind())
            && functions::name_of(v, src).as_bytes() == &src[name.byte_range()]
    })
}

/// The identifiers a binding pattern binds: the pattern itself when it
/// is a name, else the leaves of its object / array shape — a pair's
/// value, a defaulted binding's left side (the right is an expression),
/// a rest's target, a shorthand property as itself.
fn pattern_names(node: Node<'_>) -> Vec<Node<'_>> {
    match node.kind() {
        "identifier" | "shorthand_property_identifier_pattern" => vec![node],
        "object_pattern" | "array_pattern" | "rest_pattern" => ast::named_children(node)
            .into_iter()
            .flat_map(pattern_names)
            .collect(),
        "pair_pattern" => node
            .child_by_field_name("value")
            .map(pattern_names)
            .unwrap_or_default(),
        "object_assignment_pattern" | "assignment_pattern" => node
            .child_by_field_name("left")
            .map(pattern_names)
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}
