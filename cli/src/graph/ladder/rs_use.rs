//! The use-family rungs (R2 crate / R3 self+super+local module / R4
//! extern) and the R5 binder hookup — split from rs.rs at the 300
//! gate as a `#[path]` CHILD module: the parent's private items stay
//! reachable and the mount is exactly the construct the clearance-1
//! slice taught the ladder to see (dogfood by construction).

use super::{ctx_for, inline_depth};
use crate::graph::cargo;
use crate::graph::ladder::rs_reexport;
use crate::graph::ladder::rs_tree::{
    Hits, cached_tree, climb, covering_chain, covering_roots, descend, mod_named, settle, walk_all,
    walk_hits,
};
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
    let Some((global, segs)) = use_path(site.spec) else {
        return Outcome::Unresolved(Reason::OutOfScope); // hand-folded fragment
    };
    let (out, used, walked) = use_walk(site, &segs, global, pkg, roots_set, scope);
    bound(scope, walked, out, used)
}

/// The module-path prefix of a use spec, and whether it is the global
/// form (`::foo::Bar` — a crate name outright, never a local module;
/// the step-8 review: a same-named local module used to capture it).
/// None = a fragment cut mid-path by a hand fold (module header) —
/// refuse, never guess.
fn use_path(spec: &str) -> Option<(bool, Vec<&str>)> {
    let cut = [spec.find('{'), spec.find('*'), spec.find(" as ")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(spec.len());
    let prefix = spec[..cut].trim_end();
    if prefix.ends_with("::") && cut == spec.len() {
        return None;
    }
    let segs = prefix
        .trim_end_matches("::")
        .split("::")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    Some((prefix.starts_with("::"), segs))
}

/// The BIND-FREE walk shared by real use sites and the binder's one
/// hop — the design's ≤1 bound is structural because this function
/// never consults a re-export surface. Returns (outcome, consumed,
/// the sub-slice actually walked) so the binder can name the first
/// unconsumed segment.
fn use_walk<'a>(
    site: &Site,
    segs: &'a [&'a str],
    global: bool,
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
        _ if global => {
            let (o, u) = extern_rung(head, rest, pkg, roots_set, scope);
            (o, u, rest)
        }
        "crate" => {
            let anchors = covering_roots(from, roots_set);
            let (o, u) = crate_walk(anchors, rest, roots_set, scope);
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
        name => match local_module(site, name, rest, roots_set, scope) {
            Some((o, u)) => (o, u, rest),
            None => {
                let (o, u) = extern_rung(name, rest, pkg, roots_set, scope);
                (o, u, rest)
            }
        },
    }
}

/// R2 with the same-package tie-break (§4 amendment, step 8, ruling
/// ④): a lib root and a bin root of one package both cover a shared
/// module, and a walk that stopped at the roots refused as
/// ambiguous_root — `use crate::Thing` from that module could name
/// either root's item. The first unconsumed segment settles it: the
/// root that defines or imports that name at top level is the one
/// rustc compiles the site against; both or neither still refuse.
fn crate_walk(
    anchors: Vec<String>,
    segs: &[&str],
    roots_set: &BTreeSet<String>,
    scope: &Scope,
) -> (Outcome, usize) {
    let hits = match walk_hits(anchors, segs, roots_set, scope.files) {
        Ok(hits) => hits,
        Err(reason) => return (Outcome::Unresolved(reason), 0),
    };
    let terminals: BTreeSet<&str> = hits.iter().map(|(p, _)| p.as_str()).collect();
    if terminals.len() < 2 {
        return settle(hits, 2);
    }
    let owners: Hits = hits
        .iter()
        .filter(|(path, used)| {
            segs.get(*used)
                .is_some_and(|name| rs_reexport::owns(scope, path, name))
        })
        .cloned()
        .collect();
    if owners.is_empty() {
        return (Outcome::Unresolved(Reason::AmbiguousRoot), 0);
    }
    settle(owners, 2)
}

/// R3 for a bare head (§4 amendment, step 8, ruling ④ — uniform
/// paths): a module declared in the site's own namespace — the
/// file's top level, or the innermost inline `mod` body enclosing the
/// site — is read before any crate name. `use gitio::git` beside
/// `mod gitio;` names the local module, and rustc itself refuses a
/// head that is both a local module and an extern crate (E0659), so
/// reading the module first invents nothing. A bodied `mod` keeps
/// every remaining segment in this file; a declaration mounts through
/// the mod rung (so a `#[path]` remap holds) and the rest descends
/// from there. None = no such module: the head is a crate name.
fn local_module(
    site: &Site,
    head: &str,
    rest: &[&str],
    roots_set: &BTreeSet<String>,
    scope: &Scope,
) -> Option<(Outcome, usize)> {
    let parsed = cached_tree(scope, site.from);
    let (text, tree) = parsed.as_ref().as_ref()?;
    let decl = namespace_mod(tree, text, site.line.saturating_sub(1), head)?;
    if decl.child_by_field_name("body").is_some() {
        let here = Outcome::Resolved {
            path: site.from.to_string(),
            rung: 3,
        };
        return Some((here, rest.len()));
    }
    let mount = Site {
        kind: "mod_decl",
        from: site.from,
        spec: head,
        line: decl.start_position().row + 1,
    };
    let mounted = super::mod_rung(&mount, roots_set, scope);
    let Outcome::Resolved { path, .. } = mounted else {
        return Some((mounted, 0));
    };
    Some(match descend(&path, rest, roots_set, scope.files) {
        Ok((path, used)) => (Outcome::Resolved { path, rung: 3 }, used),
        Err(reason) => (Outcome::Unresolved(reason), 0),
    })
}

/// The `mod_item` named `head` declared in the namespace enclosing
/// `row`: the innermost bodied mod's declaration list on the covering
/// chain, else the file's root — a module declared one namespace up
/// is not in scope without `super::` (rustc's own rule).
fn namespace_mod<'t>(
    tree: &'t tree_sitter::Tree,
    src: &str,
    row: usize,
    head: &str,
) -> Option<tree_sitter::Node<'t>> {
    let namespace = covering_chain(tree, row)
        .iter()
        .rev()
        .find(|c| c.kind() == "mod_item" && c.child_by_field_name("body").is_some())
        .and_then(|m| m.child_by_field_name("body"))
        .unwrap_or_else(|| tree.root_node());
    crate::scan::ast::children(namespace)
        .into_iter()
        .find(|c| mod_named(*c, src, head))
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
    let Some((hop_segs, row)) = rs_reexport::binds_to(scope, path, walked[used]) else {
        return out;
    };
    // a global `pub use ::foo::Bar` carries an empty first segment
    let global = hop_segs.first().is_some_and(String::is_empty);
    let seg_refs: Vec<&str> = hop_segs
        .iter()
        .skip(usize::from(global))
        .map(String::as_str)
        .collect();
    let hop_site = Site {
        kind: "use",
        from: path,
        spec: "",
        // the pub use's own line: flatten is top-level-only, and a
        // fixed line 1 read whatever bodied `mod` opened the file as
        // the hop's namespace (the step-8 review's shadow block)
        line: row + 1,
    };
    let ctx = ctx_for(scope, path);
    let (hop_out, _, _) = use_walk(&hop_site, &seg_refs, global, ctx.0.as_ref(), &ctx.1, scope);
    match hop_out {
        Outcome::Resolved { path: g, .. } if g != *path => Outcome::ResolvedVia {
            path: g,
            rung: *rung,
        },
        _ => out,
    }
}

/// R4: a crate the toolchain provides ⇒ External; an in-scope Cargo
/// package by normalized name anchors at its lib root and the
/// remaining segments DESCEND its module tree — the audit records the
/// definition file, not the crate façade (EVAL-SET 判例: 取定义点; a
/// member whose root is not a scope file terminates here — the TS
/// workspace precedent); a declared dependency ⇒ External; anything
/// else is out of scope. Read AFTER the local namespace (§4 amendment
/// ⑬ ①, the step-8 review): `test` and `alloc` are not in the extern
/// prelude unless declared, so `mod test; use test::Helper` names the
/// module, and a real `std` clash is rustc's own E0659 — nothing this
/// order can invent.
fn extern_rung(
    name: &str,
    rest: &[&str],
    pkg: Option<&cargo::Package>,
    roots_set: &BTreeSet<String>,
    scope: &Scope,
) -> (Outcome, usize) {
    if BUILTIN.contains(&name) {
        return (Outcome::External { rung: 4 }, 0);
    }
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
