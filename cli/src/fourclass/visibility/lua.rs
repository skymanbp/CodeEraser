//! Lua visibility (plan v2.30 step 4, booklet §7), read on the
//! declaration alone. `local` hides a name from every other file —
//! `local function f` and `local f = function() end` — and every other
//! named function is reachable from outside: a global (`function f()`,
//! `f = function() end`) or a table field (`function M.f()`, `M:f`,
//! `{ f = function() end }` — the table a module returns is how
//! `require` hands out its functions). An anonymous function value
//! names nothing to export. Bit 1: no function body encloses the
//! declaration. Bit 2 never: Lua has no narrower export.

use super::{ancestors, word};
use crate::scan::binding;
use tree_sitter::Node;

pub(super) fn bits(node: Node<'_>, src: &[u8]) -> i64 {
    let exported = match binding::of(node, src) {
        Some(b) => !b.is_local(),
        // an unbound declaration is a global or a table field; an
        // unbound value is anonymous
        None => node.kind() == "function_declaration",
    };
    let enclosed =
        ancestors(node).any(|a| matches!(a.kind(), "function_declaration" | "function_definition"));
    word(exported, !enclosed)
}

#[cfg(test)]
#[path = "../../../tests/unit/fourclass/visibility/tests_lua.rs"]
mod tests;
