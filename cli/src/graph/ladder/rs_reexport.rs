//! The Rust re-export surface (§4 R5 as amended 2026-08-18: `pub
//! use` binds ≤1 hop to the DEFINITION file). This module answers the
//! surface QUESTIONS about one file — which names its top-level pub
//! uses bind, and to which full paths; the hop itself runs in
//! rs_bind.rs through the bind-free walk, so the ≤1 bound is
//! structural, and the tree reading is the child rs_surface.rs. Since
//! plan v2.30 step 5b the surface also carries what the amendment
//! first refused: a `pub use p::*` glob is followed when the module it
//! names exports the name (`exports` — one hop, a glob inside that
//! module is not followed), and a `pub extern crate x [as y]` binds
//! `x` / `y` to the crate as a global path (`::x`). Refusals stay the
//! honest half: a name F defines ITSELF at top level never hops
//! (definition wins), >1 matching entry — or >1 glob carrying the
//! name — keeps the file-level edge (picking would invent a path).
//! Since step 8 the same file also answers the crate rung's tie-break
//! (`owns`), so the hashed projection covers every top-level fact a
//! walk consults: the pub-use surface with its globs, the private use
//! bindings, and the item names with their visibility.

use super::Scope;
use super::rs_tree::cached_tree;
use std::rc::Rc;

#[path = "rs_surface.rs"]
mod surface;
use surface::{Entry, GLOB, toplevel_defs, use_entries};

/// What `f`'s surface says about `name` (binds_to): one entry binds it
/// to a full path — with the declaration's row, the hop site's own
/// line, so its namespace is read at the file's top level — or no
/// entry names it and these globs, each the path before its `*` with
/// its row, may carry it.
pub(super) enum Bind {
    Named(Vec<String>, usize),
    Globbed(Vec<(Vec<String>, usize)>),
}

/// The surface's answer for `name`; None = no bind (defined locally,
/// bound by two entries, no glob either, or no parsable surface).
pub(super) fn binds_to(scope: &Scope, f: &str, name: &str) -> Option<Bind> {
    let parsed = cached_tree(scope, f);
    let (text, tree) = parsed.as_ref().as_ref()?;
    if defines_toplevel(tree, text, name) {
        return None;
    }
    let entries = pub_surface(scope, f, text, tree);
    let mut hits = entries.iter().filter(|(n, _, _)| n == name);
    match (hits.next(), hits.next()) {
        (Some(hit), None) => Some(Bind::Named(hit.1.clone(), hit.2)),
        // ambiguous re-export: keep the file-level edge
        (Some(_), Some(_)) => None,
        (None, _) => {
            let globs: Vec<(Vec<String>, usize)> = entries
                .iter()
                .filter(|(n, _, _)| n == GLOB)
                .map(|(_, path, row)| (path.clone(), *row))
                .collect();
            (!globs.is_empty()).then_some(Bind::Globbed(globs))
        }
    }
}

/// Whether `f` exports `name` from its top level — a pub item it
/// defines, or a pub use binding the name — what a `pub use f::*` in
/// another file carries (rs_bind::globbed). A glob inside `f` is not
/// followed: one hop.
pub(super) fn exports(scope: &Scope, f: &str, name: &str) -> bool {
    parsed_says(scope, f, |text, tree| {
        let pub_item = toplevel_defs(tree, text)
            .iter()
            .any(|(n, is_pub)| n == name && *is_pub);
        pub_item
            || pub_surface(scope, f, text, tree)
                .iter()
                .any(|(n, _, _)| n == name)
    })
}

/// Whether `f` holds `name` in its top-level namespace — defined
/// there, or imported by any top-level `use` (private ones too: a
/// `crate::Thing` from a module resolves through a root's private
/// import just as well). The crate rung's tie-break between two roots
/// of one package (rs_use.rs, step 8).
pub(super) fn owns(scope: &Scope, f: &str, name: &str) -> bool {
    parsed_says(scope, f, |text, tree| {
        defines_toplevel(tree, text, name)
            || use_entries(tree, text, false)
                .iter()
                .any(|(n, _, _)| n == name)
    })
}

/// A question about `f`'s parsed text and tree, false when the file
/// does not parse — the one prelude `exports` and `owns` share (two
/// copies were the clone gate's catch).
fn parsed_says(scope: &Scope, f: &str, says: impl Fn(&str, &tree_sitter::Tree) -> bool) -> bool {
    let parsed = cached_tree(scope, f);
    parsed
        .as_ref()
        .as_ref()
        .is_some_and(|(text, tree)| says(text.as_str(), tree))
}

/// The flattened pub-use surface, memoized per sweep (the md_slugs
/// shape): (bound name, full path segments, row) per leaf entry.
fn pub_surface(scope: &Scope, f: &str, text: &str, tree: &tree_sitter::Tree) -> Rc<Vec<Entry>> {
    scope
        .memo
        .cached("rs_pubuse", f, || pub_entries(tree, text))
}

/// ONE iteration over the top-level pub use declarations — shared by
/// the surface and its hash so the two projections cannot drift (a
/// second inline copy of this loop was the census's catch).
fn pub_entries(tree: &tree_sitter::Tree, text: &str) -> Vec<Entry> {
    use_entries(tree, text, true)
}

/// F defining `name` at top level wins over any re-export of the
/// same name — the walk's own "items INSIDE that file" invariant.
fn defines_toplevel(tree: &tree_sitter::Tree, src: &str, name: &str) -> bool {
    toplevel_defs(tree, src).iter().any(|(n, _)| n == name)
}

/// The surface folded to one resolve_key input (the md slug_hash
/// sibling): the hashed projection IS the consulted projection —
/// the pub-use bindings (names and full paths, document order; a glob
/// under its `*` slot, an extern crate under its bound name), then
/// every top-level use binding's name and every top-level item name
/// with its visibility mark, the facts `owns`, `binds_to` and
/// `exports` read. A private-use or item rename costs one spurious
/// sweep; a missing fact would cost a permanently wrong edge —
/// keys.rs's own trade for the TS facts.
pub fn pubuse_hash(text: &str) -> u64 {
    let Some(grammar) = crate::scan::lang::Lang::Rust.grammar() else {
        return 0;
    };
    let Some(tree) = crate::scan::ast::parse(text, &grammar) else {
        return 0;
    };
    let mut buf = Vec::new();
    for (n, p, _) in pub_entries(&tree, text) {
        buf.extend_from_slice(n.as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(p.join("::").as_bytes());
        buf.push(b'\n');
    }
    let bindings = use_entries(&tree, text, false)
        .into_iter()
        .map(|(n, _, _)| n);
    for name in std::iter::once(String::new()).chain(bindings) {
        buf.extend_from_slice(name.as_bytes());
        buf.push(b'\n');
    }
    for (name, is_pub) in std::iter::once((String::new(), false)).chain(toplevel_defs(&tree, text))
    {
        buf.extend_from_slice(name.as_bytes());
        buf.push(if is_pub { b'+' } else { b'-' });
        buf.push(b'\n');
    }
    crate::dedup::tokens::fnv1a(&buf)
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/rs_reexport.rs"]
mod tests;
