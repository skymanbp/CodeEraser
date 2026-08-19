//! The use-family rungs (R2 crate / R3 self+super / R4 extern) and
//! the R5 binder hookup — split from rs.rs at the 300 gate as a
//! `#[path]` CHILD module: the parent's private items stay reachable
//! and the mount is exactly the construct the clearance-1 slice
//! taught the ladder to see (dogfood by construction).

use super::{ctx_for, inline_depth};
use crate::graph::cargo;
use crate::graph::ladder::rs_tree::{climb, covering_roots, descend, walk_all};
use crate::graph::ladder::{Outcome, Reason, Scope, Site};
use std::collections::BTreeSet;

/// Crates the toolchain provides without any declaration.
const BUILTIN: [&str; 5] = ["std", "core", "alloc", "proc_macro", "test"];

pub(super) fn use_rungs(
    site: &Site,
    pkg: Option<&cargo::Package>,
    roots_set: &BTreeSet<String>,
    scope: &Scope,
) -> Outcome {
    let Some(segs) = use_path(site.spec) else {
        return Outcome::Unresolved(Reason::OutOfScope); // hand-folded fragment
    };
    let (out, used, walked) = use_walk(site, &segs, pkg, roots_set, scope);
    bound(scope, walked, out, used)
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

/// The BIND-FREE walk shared by real use sites and the binder's one
/// hop — the design's ≤1 bound is structural because this function
/// never consults a re-export surface. Returns (outcome, consumed,
/// the sub-slice actually walked) so the binder can name the first
/// unconsumed segment.
fn use_walk<'a>(
    site: &Site,
    segs: &'a [&'a str],
    pkg: Option<&cargo::Package>,
    roots_set: &BTreeSet<String>,
    scope: &Scope,
) -> (Outcome, usize, &'a [&'a str]) {
    let from = site.from;
    let Some((head, rest)) = segs.split_first() else {
        // `use {…}` group only
        return (Outcome::Unresolved(Reason::OutOfScope), 0, segs);
    };
    match *head {
        "crate" => {
            let anchors = covering_roots(from, roots_set);
            let (o, u) = walk_all(anchors, rest, 2, roots_set, scope.files);
            (o, u, rest)
        }
        "self" => {
            if inline_depth(scope, from, site.line) > 0 {
                // self:: inside an inline module: every remaining
                // segment is an item of THIS file — descending would
                // mistake a sibling file module for the inline item
                let out = Outcome::Resolved {
                    path: from.to_string(),
                    rung: 3,
                };
                return (out, rest.len(), rest);
            }
            let (o, u) = walk_all(vec![from.to_string()], rest, 3, roots_set, scope.files);
            (o, u, rest)
        }
        "super" => super_walk(site, rest, roots_set, scope),
        b if BUILTIN.contains(&b) => (Outcome::External { rung: 4 }, 0, rest),
        name => {
            let (o, u) = extern_rung(name, rest, pkg, roots_set, scope);
            (o, u, rest)
        }
    }
}

/// The super:: arm (split from use_walk at the E01 fn gate): ups
/// consume inline-module depth BEFORE any file climb (the audited
/// interpolate.rs/globset rows).
fn super_walk<'a>(
    site: &Site,
    rest: &'a [&'a str],
    roots_set: &BTreeSet<String>,
    scope: &Scope,
) -> (Outcome, usize, &'a [&'a str]) {
    let from = site.from;
    let ups = 1 + rest.iter().take_while(|s| **s == "super").count();
    let tail = &rest[ups - 1..];
    let depth = inline_depth(scope, from, site.line);
    if ups <= depth {
        let (o, u) = walk_all(vec![from.to_string()], tail, 3, roots_set, scope.files);
        return (o, u, tail);
    }
    let anchors = climb(from, ups - depth, roots_set, scope.files);
    let (o, u) = walk_all(anchors, tail, 3, roots_set, scope.files);
    (o, u, tail)
}

/// §4 R5 as amended 2026-08-18: a single-terminal walk that left
/// segments unconsumed consults the terminal's re-export surface for
/// ONE hop and answers the DEFINITION file (the frozen GT's stance);
/// an unbound, ambiguous or self-pointing hop keeps the file-level
/// edge — refinement only, never a downgrade, never a guess.
fn bound(scope: &Scope, walked: &[&str], out: Outcome, used: usize) -> Outcome {
    let Outcome::Resolved { path, rung } = &out else {
        return out;
    };
    if used >= walked.len() {
        return out;
    }
    let Some(hop_segs) = crate::graph::ladder::rs_reexport::binds_to(scope, path, walked[used])
    else {
        return out;
    };
    let seg_refs: Vec<&str> = hop_segs.iter().map(String::as_str).collect();
    let hop_site = Site {
        kind: "use",
        from: path,
        spec: "",
        line: 1, // flatten is top-level-only, so depth 0 is exact
    };
    let ctx = ctx_for(scope, path);
    let (hop_out, _, _) = use_walk(&hop_site, &seg_refs, ctx.0.as_ref(), &ctx.1, scope);
    match hop_out {
        Outcome::Resolved { path: g, .. } if g != *path => Outcome::ResolvedVia {
            path: g,
            rung: *rung,
        },
        _ => out,
    }
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
) -> (Outcome, usize) {
    let members = crate::graph::ladder::members(scope, "Cargo.toml", cargo::package, |p| {
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
                    Ok((path, used)) => (Outcome::Resolved { path, rung: 4 }, used),
                    Err(reason) => (Outcome::Unresolved(reason), 0),
                }
            }
            None => (Outcome::Unresolved(Reason::OutOfScope), 0),
        },
        0 => {
            let declared = pkg.is_some_and(|p| p.deps.iter().any(|d| d.replace('-', "_") == name));
            if declared {
                return (Outcome::External { rung: 4 }, 0);
            }
            (Outcome::Unresolved(Reason::OutOfScope), 0)
        }
        _ => (Outcome::Unresolved(Reason::AmbiguousWorkspace), 0),
    }
}
