//! TS/TSX: `export` wraps a declaration, so the fact sits ABOVE the
//! node — and how far above is the whole question (plan v2.17 L round
//! piece (2), criterion T3).
//!
//! bit 0 is the disjunction of two independent climbs, each judged on
//! its own:
//!   - the identity-guarded climb: through `lexical_declaration` /
//!     `variable_declaration` freely, through a `variable_declarator`
//!     only when the declarator's name IS this unit's name. The export
//!     mechanism of `export const f = function g() {}` names `f` and
//!     never `g`; an unguarded climb would hand `g` an export it does
//!     not have — a false public-surface row on live code. `export
//!     const f = () => {}` passes because an arrow has no name of its
//!     own and the extractor already names it after the declarator;
//!     `export const dbnc = function dbnc() {}` passes because the
//!     names agree. Equality is on source bytes, both sides read
//!     through the extractor's own `name_of` throat.
//!   - two plain hops: an object-literal member reaches its `export`
//!     through the literal — `method_definition → object →
//!     export_statement`, `export default { m() {} }`. Neither
//!     `object`/`pair` nor `class_body` joins the guarded climb: in
//!     `{ handler: function hh() {} }` the exported name is the key
//!     `handler` and `hh` is not (the first defect on another node),
//!     and a class member is never named by the class's export at
//!     all. A decorator is a sibling of the declaration, not a hop.
//!
//! bit 1 reads the namespace chain: every `internal_module` and every
//! `module` node — the legacy `module N { }` spelling and `declare
//! module` alike — between the declaration and the file root. The declaration
//! (bit 0) and every inner chain member must carry `export`; the
//! OUTERMOST member must too when the file is a module (a top-level
//! import or export makes it one), and need not when the file is a
//! script — a script's top-level namespace merges into the global one,
//! so its exported members really do leave the file. `declare global`
//! has no `internal_module` and is no chain member: a global
//! augmentation's declarations are judged on an empty chain, which
//! under-reports on the safe side.

use super::{ancestors, root_of, word};
use crate::scan::{ast, functions};
use tree_sitter::Node;

pub(super) fn bits(node: Node<'_>, src: &[u8]) -> i64 {
    let exported = climbs_to_export(node, src) || two_hops_to_export(node);
    word(exported, scope_exported(node))
}

/// Declaration wrappers the guarded climb passes through freely.
const CLIMB: [&str; 2] = ["lexical_declaration", "variable_declaration"];

fn climbs_to_export(node: Node<'_>, src: &[u8]) -> bool {
    let me = functions::name_of(node, src);
    for p in ancestors(node) {
        match p.kind() {
            "export_statement" => return true,
            k if CLIMB.contains(&k) => {}
            "variable_declarator" => {
                let same = p
                    .child_by_field_name("name")
                    .is_some_and(|n| src[n.byte_range()] == *me.as_bytes());
                if !same {
                    return false;
                }
            }
            _ => return false,
        }
    }
    false
}

fn two_hops_to_export(node: Node<'_>) -> bool {
    ancestors(node)
        .take(2)
        .any(|p| p.kind() == "export_statement")
}

fn scope_exported(node: Node<'_>) -> bool {
    let chain: Vec<Node> = ancestors(node).filter(|a| is_chain_member(*a)).collect();
    let module_file = is_module_file(root_of(node));
    chain.iter().enumerate().all(|(i, member)| {
        let outermost = i + 1 == chain.len();
        (outermost && !module_file) || carries_export(*member)
    })
}

/// `namespace N { }` is `internal_module`; the legacy `module N { }`
/// spelling and `declare module` are both kind `module` (the `module`
/// keyword itself is an anonymous leaf and never an ancestor).
fn is_chain_member(n: Node<'_>) -> bool {
    matches!(n.kind(), "internal_module" | "module")
}

/// `export namespace N { … }`, or `export declare namespace N { … }`
/// with the `declare` wrapper between the member and its `export`.
fn carries_export(member: Node<'_>) -> bool {
    let mut up = ancestors(member);
    match up.next() {
        Some(p) if p.kind() == "export_statement" => true,
        Some(p) if p.kind() == "ambient_declaration" => {
            up.next().is_some_and(|g| g.kind() == "export_statement")
        }
        _ => false,
    }
}

/// A file with a top-level import or export is a module; anything
/// else is a script whose top-level scope is the global one.
fn is_module_file(root: Node<'_>) -> bool {
    ast::named_children(root)
        .iter()
        .any(|c| matches!(c.kind(), "import_statement" | "export_statement"))
}
