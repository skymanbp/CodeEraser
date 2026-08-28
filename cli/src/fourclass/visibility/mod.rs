//! Symbol visibility as a LOCAL SYNTACTIC FACT (plan v2.14, ADR-008
//! clause three; a three-bit word since plan v2.17 L round piece (2)).
//! Exported-ness is the public/private judgment axis the graph family
//! has always declared and never had a producer for: `symbols.flags`
//! was reserved-and-zero from 2g until v2.14 because "storing a guess
//! would be inventing entry-point facts" (graph/store.rs). Nothing
//! here guesses — every bit is read off the declaration itself, in the
//! file that declares it, with no resolution and no cross-file lookup.
//!
//! The word, one principle per bit:
//!   - bit 0 `VIS_EXPORTED`: the language's export mechanism, read on
//!     this declaration node alone, names THIS declaration's own
//!     identifier. `export const f` names `f` and `export class K { m }`
//!     does not name `m` (ts.rs); Rust `pub` sits on the item; Go's
//!     initial capital and Python's leading underscore are the name's
//!     own convention; a Haskell header exports what its list says,
//!     or every TOP-LEVEL binding when it has no list (hs.rs).
//!   - bit 1 `VIS_SCOPE_EXPORTED`: bit 0 holds AND the scopes between
//!     the declaration and the file root let the name out as well —
//!     still a measurement of this file's AST, never a resolution: the
//!     Rust inline `mod` chain is plain `pub` all the way up, the TS
//!     namespace chain is exported at every level that must be (ts.rs
//!     states the module/script split), no Python `def` encloses the
//!     declaration and every enclosing class is public. Go and Haskell
//!     put nothing between a declaration and its file, so their bit 1
//!     mirrors bit 0. The bit implies bit 0 by construction, which is
//!     what lets a consumer mask "exported and scope-exported" as one
//!     word.
//!   - bit 2 `VIS_RESTRICTED`: Rust `pub(crate)` / `pub(super)` /
//!     `pub(in …)` / `pub(self)` — exported (bit 0 stays 1) but only
//!     as far as the named scope.
//!
//! Only bit 0 crosses the wire: graph/symwire.rs masks the stored word
//! down to it, so the core's verdict axis does not move with the wider
//! word, and bits 1 and 2 travel with the L round's `unmentioned`
//! table alone.
//!
//! Boundaries, stated rather than papered over:
//!   - Python's `__all__` is consulted since L round step 8 (py.rs):
//!     a literal list names the module's exports outright; a module
//!     that builds `__all__` dynamically, or re-exports a name it does
//!     not declare, is still judged by its declarations under the
//!     underscore convention. The bit is a floor, never an invention.
//!   - Haskell's export list is read from the module header when one
//!     is present, lexed by the language's own comment rules
//!     (hs_lex.rs); a header without a list exports every top-level
//!     binding, and no header is `module Main(main)` as GHC applies
//!     it — the language's rules, not assumptions (hs.rs).
//!   - Nested items carry their OWN bit 0 (a `pub fn` inside a private
//!     `mod` is exported by its own declaration); what the enclosing
//!     scopes do to it is bit 1's question, answered from the same
//!     file, and reachability across files stays with the graph.

mod hs;
mod hs_lex;
mod py;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_hs;
mod ts;

use crate::scan::ast;
use crate::scan::lang::Lang;
use tree_sitter::Node;

/// Frozen bit positions for `symbols.flags`. Bit 0 is the one the
/// graph's verdict codes have always meant by "public".
pub const VIS_EXPORTED: i64 = 1 << 0;
/// Bit 1: the enclosing scopes let the name out too (module doc).
pub const VIS_SCOPE_EXPORTED: i64 = 1 << 1;
/// Bit 2: exported only as far as a named scope (module doc).
pub const VIS_RESTRICTED: i64 = 1 << 2;

/// Markdown sections: every heading is an anchor any document may
/// link to, so a section is public by construction — there is no
/// private heading to distinguish it from — and nothing scopes a
/// heading, so bit 1 mirrors bit 0 as it does for Go and Haskell.
pub const MARKDOWN_VIS: i64 = VIS_EXPORTED | VIS_SCOPE_EXPORTED;

/// The visibility word of one declaration node.
pub fn bits(node: Node<'_>, src: &[u8], lang: Lang) -> i64 {
    match lang {
        Lang::Rust => rust_bits(node, src),
        Lang::TypeScript | Lang::Tsx => ts::bits(node, src),
        Lang::Python => word(py::exported(node, src), py::scope_open(node, src)),
        Lang::Go => word(go_exported(node, src), go_scope_open(node)),
        Lang::Haskell => word(hs::exported(node, src), true),
        _ => 0,
    }
}

/// Bits 0 and 1 from the two facts they stand on: bit 1 is only ever
/// set beside bit 0 (module doc).
fn word(exported: bool, scope_open: bool) -> i64 {
    match (exported, scope_open) {
        (false, _) => 0,
        (true, false) => VIS_EXPORTED,
        (true, true) => VIS_EXPORTED | VIS_SCOPE_EXPORTED,
    }
}

/// Rust: `pub` on the item is bit 0; anything narrower than plain
/// `pub` is bit 2 on top; the inline `mod` chain being plain `pub`
/// all the way up is bit 1.
fn rust_bits(node: Node<'_>, src: &[u8]) -> i64 {
    let Some(modifier) = visibility_modifier(node) else {
        return 0;
    };
    let restricted = if text(modifier, src) == "pub" {
        0
    } else {
        VIS_RESTRICTED
    };
    word(true, rust_scope_open(node, src)) | restricted
}

fn visibility_modifier(node: Node<'_>) -> Option<Node<'_>> {
    ast::children(node)
        .into_iter()
        .find(|c| c.kind() == "visibility_modifier")
}

/// Rust's scope chain: every inline `mod` between the item and the
/// file root is plain `pub` (a `pub(crate) mod` closes the scope the
/// way bit 2 narrows an item), and no function body encloses the
/// item — an item declared inside a body is visible to that body
/// alone, the safe-side reading of a case the criterion leaves
/// unstated.
fn rust_scope_open(node: Node<'_>, src: &[u8]) -> bool {
    ancestors(node).all(|a| match a.kind() {
        "mod_item" => visibility_modifier(a).is_some_and(|m| text(m, src) == "pub"),
        "function_item" | "closure_expression" => false,
        _ => true,
    })
}

/// Go: an identifier's first rune decides. Upper case is exported —
/// the whole rule, checked on the declaration's own name.
fn go_exported(node: Node<'_>, src: &[u8]) -> bool {
    match name_text(node, src) {
        Some(name) => name.chars().next().is_some_and(char::is_uppercase),
        None => false,
    }
}

/// Go's scope chain: a function body closes it — a `type` declared
/// inside a func is visible to that body alone, whatever its case
/// (the Rust and Python arms' reading of a body-local item; the
/// step-8 review found the Go arm storing bit 1 on it).
fn go_scope_open(node: Node<'_>) -> bool {
    !ancestors(node).any(|a| {
        matches!(
            a.kind(),
            "function_declaration" | "method_declaration" | "func_literal"
        )
    })
}

/// The parent chain, innermost first, ending at the file root (shared
/// with the mention category word, mention/conv, which reads the same
/// chain for enclosing classes, ambient blocks and Rust attributes).
pub(crate) fn ancestors(node: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    std::iter::successors(node.parent(), |n| n.parent())
}

fn root_of(node: Node<'_>) -> Node<'_> {
    ancestors(node).last().unwrap_or(node)
}

fn name_text(node: Node<'_>, src: &[u8]) -> Option<String> {
    Some(text(node.child_by_field_name("name")?, src))
}

fn text(node: Node<'_>, src: &[u8]) -> String {
    String::from_utf8_lossy(&src[node.byte_range()]).into_owned()
}
