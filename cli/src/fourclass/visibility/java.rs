//! Java: the access modifiers are the export mechanism, read on the
//! declaration node and its enclosing type bodies alone (plan v2.30
//! step 3; design booklet §7 row Java). Every shape named here was
//! probed on tree-sitter-java 0.23.5 (the scripts/tsprobe transcripts)
//! before the reading was written: the keywords are anonymous tokens
//! under an unfielded `modifiers` child.
//!
//! bit 0 — the access a declaration has, spelled or implied:
//!   - `public`, `protected`, `private` as spelled;
//!   - nothing spelled: an interface's or an annotation type's member
//!     is public (the language says so), an enum's constructor is
//!     private (likewise), and everything else is package access —
//!     exported, but only as far as its package (bit 2, D14; the Rust
//!     `pub(crate)` reading). `protected` is exported and restricted
//!     too: a subclass and the package reach it, nothing else does.
//!   - `private` is 0.
//!
//! bit 1 reads the enclosing chain: every enclosing type declaration
//! itself public where IT sits (a package or protected type closes the
//! scope, as a `pub(crate) mod` does), and no body a name cannot leave
//! — a method or initializer block, a constructor body, a lambda, an
//! anonymous class (`new T() { … }`) or an enum constant's body. So a
//! local or anonymous class's members keep their own bit 0 and never
//! get bit 1.

use super::{VIS_RESTRICTED, ancestors, word};
use crate::scan::ast;
use tree_sitter::Node;

/// A declaration's access, spelled or implied (module doc).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Access {
    Public,
    Protected,
    Package,
    Private,
}

/// The named type declarations whose own access opens or closes the
/// scope of what they hold.
const TYPES: [&str; 5] = [
    "class_declaration",
    "interface_declaration",
    "enum_declaration",
    "record_declaration",
    "annotation_type_declaration",
];

/// Bodies a declared name never leaves (module doc, bit 1).
const CLOSED: [&str; 6] = [
    "block",
    "constructor_body",
    "lambda_expression",
    "object_creation_expression",
    "enum_constant",
    "static_initializer",
];

pub(super) fn bits(node: Node<'_>) -> i64 {
    match access(node) {
        Access::Private => 0,
        Access::Public => word(true, scope_open(node)),
        Access::Protected | Access::Package => word(true, scope_open(node)) | VIS_RESTRICTED,
    }
}

/// A declaration's modifier tokens and annotations — the children of
/// its unfielded `modifiers` node. Shared with the convention word's
/// Java arm (mention/conv), which reads the annotations among them.
pub(crate) fn modifiers<'t>(node: Node<'t>) -> Vec<Node<'t>> {
    ast::children(node)
        .into_iter()
        .filter(|c| c.kind() == "modifiers")
        .flat_map(ast::children)
        .collect()
}

/// The access keyword among the declaration's modifiers, else the one
/// its holder implies (module doc, bit 0).
fn access(node: Node<'_>) -> Access {
    let spelled = modifiers(node).into_iter().find_map(|m| match m.kind() {
        "public" => Some(Access::Public),
        "protected" => Some(Access::Protected),
        "private" => Some(Access::Private),
        _ => None,
    });
    if let Some(access) = spelled {
        return access;
    }
    let holder = node.parent().map(|p| p.kind());
    match holder {
        Some("interface_body" | "annotation_type_body") => Access::Public,
        Some("enum_body_declarations") if node.kind() == "constructor_declaration" => {
            Access::Private
        }
        _ => Access::Package,
    }
}

/// The scope chain (module doc, bit 1).
fn scope_open(node: Node<'_>) -> bool {
    ancestors(node).all(|a| {
        let kind = a.kind();
        if CLOSED.contains(&kind) {
            return false;
        }
        !TYPES.contains(&kind) || access(a) == Access::Public
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/fourclass/visibility/tests_java.rs"]
mod tests;
