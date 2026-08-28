//! Rust rungs (design §4 row 3). R1 `mod foo;` builds the module
//! structure: dir/foo.rs | dir/foo/mod.rs, where dir is the declarer's
//! child directory (its own dir for crate roots — Cargo surface — and
//! mod.rs, dir/<stem>/ otherwise). R2 `use crate::…` walks that tree
//! from the covering crate root(s), stopping at the DEEPEST in-scope
//! child module: remaining segments are items or inline modules
//! INSIDE that file — or names on its re-export surface, which the
//! binder answers (R5 below). R3 self::/super:: — the same tree
//! anchored at the site's own file, with inline `mod` depth consumed
//! BEFORE any file climb (the audited interpolate.rs/globset rows: a
//! use inside `#[cfg(test)] mod tests` means super IS the enclosing
//! file); a self:: path in an inline module or matching no child
//! resolves to the file ITSELF (the island red condition: intra-file
//! references come home, never dangle); a bare head declared as a
//! module in the site's own namespace is the same local tree (uniform
//! paths — §4 amendment, plan v2.17 L round step 8, ruling ④), read
//! before any crate name. R4 bare head: builtin crates and declared
//! dependencies (registry or out-of-scope path) ⇒ External; an
//! in-scope package matching by normalized name ('-' → '_') anchors at
//! its lib root and the remaining segments descend its tree — the
//! audit records the definition file, not the crate façade. A crate::
//! walk that stops at BOTH roots of a lib+bin package (the same
//! amendment) is settled by the root whose top level holds the next
//! segment; neither or both still refuse.
//!
//! R5: `#[path = "…"]` remaps answer at R1 (pre-registered, the
//! literal answers at R1): the attribute is read off the per-sweep
//! cached tree; the base is the declaring file's own directory, and
//! inside inline modules child_dir plus the enclosing mod names
//! (rustc reference, both habitats). Macro-generated mod/use never
//! parse as mod_item/use_declaration, so the macro bucket is
//! structurally empty (the Python dynamic precedent). Specs are
//! first-line fragments (sites.rs), but rustfmt folds `use` only at
//! brace groups, so the pre-`{` prefix — all the walk consumes — is
//! complete whenever a brace is present; a fragment ending in `::`
//! with NO brace is a hand-folded cut, refused rather than guessed
//! shallow. Symbol binding landed 2026-08-18 (§4 R5 amendment,
//! user-ratified): a single unambiguous top-level `pub use` binds
//! ONE hop to the definition file — rs_reexport.rs owns the surface
//! facts, rs_use::bound owns the hop, and every refusal (glob,
//! ambiguity, local definition, pub extern crate) keeps the
//! file-level edge. The audited BinaryDetection row's façade answer
//! is thereby repaid at the definition point.

use super::{Outcome, Reason, Scope, Site};
use crate::graph::{cargo, roots};
use std::collections::BTreeSet;

// the use-family rungs ride a #[path] child mount — the very
// construct the ladder learned to read in clearance 1
#[path = "rs_use.rs"]
mod rs_use;

pub fn resolve(site: &Site, scope: &Scope) -> Outcome {
    let ctx = ctx_for(scope, site.from);
    let (pkg, roots_set) = (&ctx.0, &ctx.1);
    match site.kind {
        "mod_decl" => mod_rung(site, roots_set, scope),
        "use" => rs_use::use_rungs(site, pkg.as_ref(), roots_set, scope),
        _ => Outcome::Unresolved(Reason::Unsupported),
    }
}

/// Per-sweep package context (review MED: this pair was re-derived
/// for EVERY site) — one throat, also the binder's anchor for its
/// hop from a terminal file in another directory.
type RsCtx = (Option<cargo::Package>, BTreeSet<String>);
fn ctx_for(scope: &Scope, from: &str) -> std::rc::Rc<RsCtx> {
    scope.memo.cached("rs_ctx", &roots::parent_dir(from), || {
        let pkg = cargo::nearest(scope.root, &roots::parent_dir(from));
        let roots_set = pkg
            .as_ref()
            .map(|p| p.crate_roots(scope.files))
            .unwrap_or_default();
        (pkg, roots_set)
    })
}

/// R1: an explicit `#[path]` remap wins outright; otherwise one
/// child lookup under the convention base — the shared throat all
/// tree walks use.
fn mod_rung(site: &Site, roots_set: &BTreeSet<String>, scope: &Scope) -> Outcome {
    use super::rs_tree::{Child, child_in};
    if let Some(target) = path_attr(scope, site.from, site.line, site.spec) {
        let base = path_base(scope, site, roots_set);
        return match roots::join_rel(&base, &target) {
            Some(path) if scope.files.contains(&path) => Outcome::Resolved { path, rung: 1 },
            _ => Outcome::Unresolved(Reason::OutOfScope),
        };
    }
    match child_in(&conv_base(scope, site, roots_set), site.spec, scope.files) {
        Child::One(path) => Outcome::Resolved { path, rung: 1 },
        Child::Both => Outcome::Unresolved(Reason::AmbiguousPaths),
        Child::None => Outcome::Unresolved(Reason::OutOfScope),
    }
}

/// The bodied `mod` names enclosing `line`, outermost first — empty
/// at file level or when the file does not parse.
fn inline_mods_at(scope: &Scope, from: &str, line: usize) -> Vec<String> {
    let parsed = super::rs_tree::cached_tree(scope, from);
    match parsed.as_ref() {
        Some((text, tree)) => super::rs_tree::inline_mods(tree, text, line.saturating_sub(1)),
        None => Vec::new(),
    }
}

/// The directory a convention `mod x;` mounts under (rustc reference,
/// both habitats): the declarer's child_dir — own dir for mod-rs and
/// crate roots, dir/<stem> otherwise — then one directory per
/// enclosing bodied `mod`: `mod inner { mod deep; }` in src/lib.rs
/// loads src/inner/deep.rs. ONE authority for the `#[path]` inline
/// base and the convention lookup — until the step-8 review's
/// counterexample the lookup read the file-level directory alone and
/// mounted the inline declaration one level too shallow.
fn conv_base(scope: &Scope, site: &Site, roots_set: &BTreeSet<String>) -> String {
    let base = super::rs_tree::child_dir(site.from, roots_set);
    inline_mods_at(scope, site.from, site.line)
        .iter()
        .fold(base, |b, m| roots::join_dir(&b, m))
}

/// The directory a `#[path]` literal resolves against: file-level =
/// the declaring file's OWN directory — never the convention
/// child_dir, which would land one level too deep; inside inline
/// modules the convention base itself.
fn path_base(scope: &Scope, site: &Site, roots_set: &BTreeSet<String>) -> String {
    if inline_mods_at(scope, site.from, site.line).is_empty() {
        return roots::parent_dir(site.from);
    }
    conv_base(scope, site, roots_set)
}

/// How many inline `mod x { … }` bodies enclose the site line —
/// parsed with the real grammar: a brace count would be lied to by
/// string literals (the audited glue.rs CODE constant). Only a
/// BODIED mod opens a nested scope; `mod x;` is a declaration.
fn inline_depth(scope: &Scope, from: &str, line: usize) -> usize {
    let parsed = super::rs_tree::cached_tree(scope, from);
    let Some((_, tree)) = parsed.as_ref() else {
        return 0;
    };
    super::rs_tree::covering_chain(tree, line.saturating_sub(1))
        .iter()
        .filter(|c| c.kind() == "mod_item" && c.child_by_field_name("body").is_some())
        .count()
}

/// The `#[path = "…"]` remap on the mod_decl at `line` named `name`,
/// if any. Attributes are preceding SIBLINGS of the item (mod_item
/// has no attribute member in the grammar) and they stack — the
/// self-repo mounts all sit behind a #[cfg(test)] — so the walk
/// steps back over consecutive attribute_items.
fn path_attr(scope: &Scope, from: &str, line: usize, name: &str) -> Option<String> {
    use super::rs_tree::{attr_path_value, cached_tree, mod_item_at};
    let parsed = cached_tree(scope, from);
    let (text, tree) = parsed.as_ref().as_ref()?;
    let node = mod_item_at(tree, text, line.saturating_sub(1), name)?;
    let mut prev = node.prev_sibling();
    while let Some(p) = prev {
        if p.kind() != "attribute_item" {
            break;
        }
        if let Some(v) = attr_path_value(p, text) {
            return Some(v);
        }
        prev = p.prev_sibling();
    }
    None
}
