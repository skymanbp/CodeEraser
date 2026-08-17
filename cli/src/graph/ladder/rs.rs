//! Rust rungs (design §4 row 3). R1 `mod foo;` builds the module
//! structure: dir/foo.rs | dir/foo/mod.rs, where dir is the declarer's
//! child directory (its own dir for crate roots — Cargo surface — and
//! mod.rs, dir/<stem>/ otherwise). R2 `use crate::…` walks that tree
//! from the covering crate root(s), stopping at the DEEPEST in-scope
//! child module: remaining segments are items or inline modules
//! INSIDE that file, so the deepest file is the true target — which
//! also lands `pub use` edges on the re-exporter (zero hops meets the
//! design's ≤1 bound; the via_reexport flag needs symbol binding,
//! 2g). R3 self::/super:: — the same tree anchored at the site's own
//! file, with inline `mod` depth consumed BEFORE any file climb (the
//! audited interpolate.rs/globset rows: a use inside `#[cfg(test)]
//! mod tests` means super IS the enclosing file); a self:: path in an
//! inline module or matching no child resolves to the file ITSELF
//! (the island red condition: intra-file references come home, never
//! dangle). R4 bare head: builtin crates and declared dependencies
//! (registry or out-of-scope path) ⇒ External; an in-scope package
//! matching by normalized name ('-' → '_') anchors at its lib root
//! and the remaining segments descend its tree — the audit records
//! the definition file, not the crate façade.
//!
//! R5 honesty: `#[path]` remaps need attribute text only the AST has
//! — until phase 1.5 hands content over they land Unresolved (recall
//! loss, never a wrong edge); macro-generated mod/use never parse as
//! mod_item/use_declaration, so the macro bucket is structurally
//! empty (the Python dynamic precedent). Specs are first-line
//! fragments (sites.rs), but rustfmt folds `use` only at brace
//! groups, so the pre-`{` prefix — all the walk consumes — is
//! complete whenever a brace is present; a fragment ending in `::`
//! with NO brace is a hand-folded cut, refused rather than guessed
//! shallow. Symbol binding stays future work (the audited
//! BinaryDetection re-export row is a KNOWN wrong): a name
//! re-exported by the façade lands on the façade file.

use super::rs_tree::{climb, covering_roots, descend, walk_all};
use super::{Outcome, Reason, Scope, Site};
use crate::graph::{cargo, roots};
use std::collections::BTreeSet;

/// Crates the toolchain provides without any declaration.
const BUILTIN: [&str; 5] = ["std", "core", "alloc", "proc_macro", "test"];

pub fn resolve(site: &Site, scope: &Scope) -> Outcome {
    // per-sweep: sites in one directory share the nearest-Cargo.toml
    // walk-up + parse and the O(files) crate-root scan (review MED —
    // this pair was re-derived for EVERY site)
    let ctx = scope
        .memo
        .cached("rs_ctx", &roots::parent_dir(site.from), || {
            let pkg = cargo::nearest(scope.root, &roots::parent_dir(site.from));
            let roots_set = pkg
                .as_ref()
                .map(|p| p.crate_roots(scope.files))
                .unwrap_or_default();
            (pkg, roots_set)
        });
    let (pkg, roots_set) = (&ctx.0, &ctx.1);
    match site.kind {
        "mod_decl" => mod_rung(site.from, site.spec, roots_set, scope.files),
        "use" => use_rungs(site, pkg.as_ref(), roots_set, scope),
        _ => Outcome::Unresolved(Reason::Unsupported),
    }
}

/// R1: one child lookup — the shared throat all tree walks use.
fn mod_rung(
    from: &str,
    name: &str,
    roots_set: &BTreeSet<String>,
    files: &BTreeSet<String>,
) -> Outcome {
    use super::rs_tree::{Child, child};
    match child(from, name, roots_set, files) {
        Child::One(path) => Outcome::Resolved { path, rung: 1 },
        Child::Both => Outcome::Unresolved(Reason::AmbiguousPaths),
        Child::None => Outcome::Unresolved(Reason::OutOfScope),
    }
}

fn use_rungs(
    site: &Site,
    pkg: Option<&cargo::Package>,
    roots_set: &BTreeSet<String>,
    scope: &Scope,
) -> Outcome {
    let from = site.from;
    let Some(segs) = use_path(site.spec) else {
        return Outcome::Unresolved(Reason::OutOfScope); // hand-folded fragment
    };
    let Some((head, rest)) = segs.split_first() else {
        return Outcome::Unresolved(Reason::OutOfScope); // `use {…}` group only
    };
    match *head {
        "crate" => walk_all(
            covering_roots(from, roots_set),
            rest,
            2,
            roots_set,
            scope.files,
        ),
        "self" => {
            if inline_depth(scope, from, site.line) > 0 {
                // self:: inside an inline module: every remaining
                // segment is an item of THIS file — descending would
                // mistake a sibling file module for the inline item
                return Outcome::Resolved {
                    path: from.to_string(),
                    rung: 3,
                };
            }
            walk_all(vec![from.to_string()], rest, 3, roots_set, scope.files)
        }
        "super" => {
            let ups = 1 + rest.iter().take_while(|s| **s == "super").count();
            let tail = &rest[ups - 1..];
            // super consumes inline-module depth BEFORE any file
            // climb (the audited interpolate.rs/globset rows)
            let depth = inline_depth(scope, from, site.line);
            if ups <= depth {
                return walk_all(vec![from.to_string()], tail, 3, roots_set, scope.files);
            }
            let anchors = climb(from, ups - depth, roots_set, scope.files);
            walk_all(anchors, tail, 3, roots_set, scope.files)
        }
        b if BUILTIN.contains(&b) => Outcome::External { rung: 4 },
        name => extern_rung(name, rest, pkg, roots_set, scope),
    }
}

/// How many inline `mod x { … }` bodies enclose the site line —
/// parsed with the real grammar: a brace count would be lied to by
/// string literals (the audited glue.rs CODE constant). Only a
/// BODIED mod opens a nested scope; `mod x;` is a declaration.
fn inline_depth(scope: &Scope, from: &str, line: usize) -> usize {
    // per-sweep parse cache: every self/super site in one file used
    // to re-read AND re-parse it with tree-sitter (review MED); the
    // text rides along so the tree's provenance stays in one place
    let parsed = scope.memo.cached("rs_tree", from, || {
        let text = std::fs::read_to_string(scope.root.join(from)).ok()?;
        let grammar = crate::scan::lang::Lang::Rust.grammar()?;
        let tree = crate::scan::ast::parse(&text, &grammar)?;
        Some((text, tree))
    });
    let Some((_, tree)) = parsed.as_ref() else {
        return 0;
    };
    let row = line.saturating_sub(1);
    let (mut node, mut depth) = (tree.root_node(), 0);
    'down: loop {
        for c in crate::scan::ast::children(node) {
            if c.start_position().row <= row && row <= c.end_position().row {
                if c.kind() == "mod_item" && c.child_by_field_name("body").is_some() {
                    depth += 1;
                }
                node = c;
                continue 'down;
            }
        }
        return depth;
    }
}

/// The module-path prefix of a use spec. None = a fragment cut
/// mid-path by a hand fold (module header) — refuse, never guess.
fn use_path(spec: &str) -> Option<Vec<&str>> {
    let cut = [spec.find('{'), spec.find('*'), spec.find(" as ")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(spec.len());
    let prefix = spec[..cut].trim_end();
    if prefix.ends_with("::") && cut == spec.len() {
        return None;
    }
    Some(
        prefix
            .trim_end_matches("::")
            .split("::")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

/// R4: an in-scope Cargo package by normalized name anchors at its
/// lib root and the remaining segments DESCEND its module tree —
/// the audit records the definition file, not the crate façade
/// (EVAL-SET 判例: 取定义点; a member whose root is not a scope file
/// terminates here — the TS workspace precedent); a declared
/// dependency ⇒ External; anything else is out of scope.
fn extern_rung(
    name: &str,
    rest: &[&str],
    pkg: Option<&cargo::Package>,
    roots_set: &BTreeSet<String>,
    scope: &Scope,
) -> Outcome {
    let members = super::members(scope, "Cargo.toml", cargo::package, |p| {
        p.name
            .as_deref()
            .is_some_and(|n| n.replace('-', "_") == name)
    });
    match members.len() {
        1 => match members[0].lib_root(scope.files) {
            Some(root) => {
                let mut anchored = roots_set.clone();
                anchored.insert(root.clone());
                match descend(&root, rest, &anchored, scope.files) {
                    Ok(path) => Outcome::Resolved { path, rung: 4 },
                    Err(reason) => Outcome::Unresolved(reason),
                }
            }
            None => Outcome::Unresolved(Reason::OutOfScope),
        },
        0 => {
            let declared = pkg.is_some_and(|p| p.deps.iter().any(|d| d.replace('-', "_") == name));
            if declared {
                return Outcome::External { rung: 4 };
            }
            Outcome::Unresolved(Reason::OutOfScope)
        }
        _ => Outcome::Unresolved(Reason::AmbiguousWorkspace),
    }
}
