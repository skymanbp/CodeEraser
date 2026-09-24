//! C and C++: linkage and access are the export mechanisms, both read
//! on the declaration node and its enclosing bodies alone (plan v2.30
//! step 2; design booklet §7 rows C / C++). Every shape named here was
//! probed on tree-sitter-c 0.24.2 / tree-sitter-cpp 0.23.4 (the
//! scripts/tsprobe transcripts) before the reading was written.
//!
//! bit 0, three roads:
//!   - a MEMBER of a class body — the node, or the ancestor that is a
//!     direct child of a `field_declaration_list` (a nested type sits
//!     inside a `field_declaration`, a template member inside a
//!     `template_declaration`) — is exported by the nearest preceding
//!     `access_specifier` sibling, or by the body's own default when
//!     none precedes it: `class` opens private, `struct` / `union` open
//!     public. `protected` is exported and restricted (bit 2): a derived
//!     class reaches it, nothing else does. `static` inside a body is a
//!     class-static member, never internal linkage, and is not read.
//!     The member search stops at a function body: what a body declares
//!     is nobody's member, whatever class the function belongs to.
//!   - a macro (`#define`) is visible from its line to the end of every
//!     translation unit that includes the file, whatever encloses it —
//!     bits 0 and 1 both, and no access specifier applies (the
//!     preprocessor runs before the class exists).
//!   - everything else — a free function, a type, a namespace, an
//!     out-of-class member definition — has external linkage unless a
//!     `storage_class_specifier` spells `static` or an anonymous
//!     `namespace_definition` (no `name`) encloses it. An out-of-class
//!     definition (`void K::m() {}`) cannot see its class's access
//!     specifier from here and reads as exported: the safe side (D13,
//!     the 2026-09-24 ruling — a false "private" is what the erase
//!     verdict must never be handed).
//!
//! bit 1 reads the enclosing chain: no function body (a
//! `function_definition` or a `lambda_expression` — a local type is that
//! body's alone), every namespace on the chain named, and every
//! enclosing class body itself public where IT sits (its own member
//! access, by the same reading). `template_declaration`,
//! `linkage_specification` and a `field_declaration` wrapper are
//! transparent. C has no access specifier, no namespace and no lambda,
//! so the same function reduces to the booklet's C row there: bit 0 =
//! no `static`, bit 1 = no function body.

use super::{VIS_RESTRICTED, ancestors, text, word};
use crate::scan::ast;
use tree_sitter::Node;

/// What an `access_specifier` spells, or what a body opens with.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Access {
    Public,
    Protected,
    Private,
}

/// The preprocessor's own declarations (module doc, second road).
const MACROS: [&str; 2] = ["preproc_def", "preproc_function_def"];

pub(super) fn bits(node: Node<'_>, src: &[u8]) -> i64 {
    if MACROS.contains(&node.kind()) {
        return word(true, true);
    }
    match member_access(node, src) {
        Some(Access::Private) => 0,
        Some(Access::Protected) => word(true, scope_open(node, src)) | VIS_RESTRICTED,
        Some(Access::Public) => word(true, scope_open(node, src)),
        None => word(!internal_linkage(node, src), scope_open(node, src)),
    }
}

/// The access of a class-body member (module doc, first road); None
/// for a node that is no member — outside every class body, or inside
/// a function body, which the climb refuses to cross.
fn member_access(node: Node<'_>, src: &[u8]) -> Option<Access> {
    let mut member = node;
    loop {
        let parent = member.parent()?;
        if parent.kind() == "field_declaration_list" {
            break;
        }
        if matches!(parent.kind(), "function_definition" | "lambda_expression") {
            return None;
        }
        member = parent;
    }
    let mut prev = member.prev_sibling();
    while let Some(p) = prev {
        if p.kind() == "access_specifier" {
            return Some(match text(p, src).as_str() {
                "public" => Access::Public,
                "protected" => Access::Protected,
                _ => Access::Private,
            });
        }
        prev = p.prev_sibling();
    }
    // the body's default: the list's parent is the specifier that owns it
    let holder = member.parent().and_then(|list| list.parent());
    Some(match holder.map(|h| h.kind()) {
        Some("class_specifier") => Access::Private,
        _ => Access::Public,
    })
}

/// `static` on the declaration itself, or an anonymous namespace
/// anywhere above it (module doc, third road).
fn internal_linkage(node: Node<'_>, src: &[u8]) -> bool {
    let is_static = ast::children(node)
        .into_iter()
        .any(|c| c.kind() == "storage_class_specifier" && text(c, src) == "static");
    is_static || ancestors(node).any(|a| anonymous_namespace(a))
}

fn anonymous_namespace(node: Node<'_>) -> bool {
    node.kind() == "namespace_definition" && node.child_by_field_name("name").is_none()
}

/// The scope chain (module doc): bodies close it, an anonymous
/// namespace closes it, a class body is open exactly when it is a
/// public member where it sits or no member at all.
fn scope_open(node: Node<'_>, src: &[u8]) -> bool {
    ancestors(node).all(|a| match a.kind() {
        "function_definition" | "lambda_expression" => false,
        "namespace_definition" => !anonymous_namespace(a),
        "class_specifier" | "struct_specifier" | "union_specifier" => {
            member_access(a, src).is_none_or(|access| access == Access::Public)
        }
        _ => true,
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/fourclass/visibility/tests_c.rs"]
mod tests;
