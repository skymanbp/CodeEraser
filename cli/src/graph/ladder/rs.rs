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
//! file; a self:: path matching no child resolves to the file ITSELF
//! (the island red condition: intra-file references come home, never
//! dangle). R4 bare head: builtin crates and declared dependencies
//! (registry or out-of-scope path) ⇒ External; an in-scope package
//! matching by normalized name ('-' → '_') resolves to its lib root.
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
//! shallow. The blind spot (newline BEFORE `::`) reads as a complete
//! spec and is repaid at phase 1.5.

use super::{Outcome, Reason, Scope};
use crate::graph::{cargo, roots};
use std::collections::BTreeSet;

/// Crates the toolchain provides without any declaration.
const BUILTIN: [&str; 5] = ["std", "core", "alloc", "proc_macro", "test"];

pub fn resolve(kind: &str, from: &str, spec: &str, scope: &Scope) -> Outcome {
    let pkg = cargo::nearest(scope.root, &roots::parent_dir(from));
    let roots_set = pkg
        .as_ref()
        .map(|p| p.crate_roots(scope.files))
        .unwrap_or_default();
    match kind {
        "mod_decl" => mod_rung(from, spec, &roots_set, scope.files),
        "use" => use_rungs(from, spec, pkg.as_ref(), &roots_set, scope),
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
    match child(from, name, roots_set, files) {
        Child::One(path) => Outcome::Resolved { path, rung: 1 },
        Child::Both => Outcome::Unresolved(Reason::AmbiguousPaths),
        Child::None => Outcome::Unresolved(Reason::OutOfScope),
    }
}

fn use_rungs(
    from: &str,
    spec: &str,
    pkg: Option<&cargo::Package>,
    roots_set: &BTreeSet<String>,
    scope: &Scope,
) -> Outcome {
    let Some(segs) = use_path(spec) else {
        return Outcome::Unresolved(Reason::OutOfScope); // hand-folded fragment
    };
    let Some((head, rest)) = segs.split_first() else {
        return Outcome::Unresolved(Reason::OutOfScope); // `use {…}` group only
    };
    let crate_anchors = covering_roots(from, roots_set);
    match *head {
        "crate" => walk_all(crate_anchors, rest, 2, roots_set, scope.files),
        "self" => walk_all(vec![from.to_string()], rest, 3, roots_set, scope.files),
        "super" => {
            let ups = 1 + rest.iter().take_while(|s| **s == "super").count();
            let anchors = climb(from, ups, roots_set, scope.files);
            walk_all(anchors, &rest[ups - 1..], 3, roots_set, scope.files)
        }
        b if BUILTIN.contains(&b) => Outcome::External { rung: 4 },
        name => extern_rung(name, pkg, scope),
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

/// Walk one segment list from every anchor; distinct terminals from
/// different anchors are ambiguous_root (the Python cross-root
/// stance), a double hit at one step is ambiguous_paths, an empty
/// anchor set (climbed above the crate root) is out of scope.
fn walk_all(
    anchors: Vec<String>,
    segs: &[&str],
    rung: u8,
    roots_set: &BTreeSet<String>,
    files: &BTreeSet<String>,
) -> Outcome {
    let mut hits = BTreeSet::new();
    for anchor in anchors {
        match descend(&anchor, segs, roots_set, files) {
            Ok(target) => {
                hits.insert(target);
            }
            Err(reason) => return Outcome::Unresolved(reason),
        }
    }
    match hits.len() {
        0 => Outcome::Unresolved(Reason::OutOfScope),
        1 => Outcome::Resolved {
            path: hits.pop_first().expect("len checked"),
            rung,
        },
        _ => Outcome::Unresolved(Reason::AmbiguousRoot),
    }
}

/// Descend the convention tree; stopping early is not failure — the
/// remaining segments live inside the deepest matched file.
fn descend(
    anchor: &str,
    segs: &[&str],
    roots_set: &BTreeSet<String>,
    files: &BTreeSet<String>,
) -> Result<String, Reason> {
    let mut cur = anchor.to_string();
    for seg in segs {
        match child(&cur, seg, roots_set, files) {
            Child::One(next) => cur = next,
            Child::Both => return Err(Reason::AmbiguousPaths),
            Child::None => break,
        }
    }
    Ok(cur)
}

enum Child {
    One(String),
    Both,
    None,
}

/// The child-module lookup throat: dir/name.rs | dir/name/mod.rs,
/// both present is rustc's own E0761 ambiguity.
fn child(file: &str, name: &str, roots_set: &BTreeSet<String>, files: &BTreeSet<String>) -> Child {
    let dir = child_dir(file, roots_set);
    let plain = roots::join_dir(&dir, &format!("{name}.rs"));
    let modrs = roots::join_dir(&dir, &format!("{name}/mod.rs"));
    match (files.contains(&plain), files.contains(&modrs)) {
        (true, true) => Child::Both,
        (true, false) => Child::One(plain),
        (false, true) => Child::One(modrs),
        (false, false) => Child::None,
    }
}

/// A crate root or a mod.rs parents children in its OWN directory;
/// any other module file parents them under dir/<stem>/ (2018 style).
fn child_dir(file: &str, roots_set: &BTreeSet<String>) -> String {
    let dir = roots::parent_dir(file);
    if roots_set.contains(file) || is_mod_rs(file) {
        return dir;
    }
    let stem = file
        .rsplit('/')
        .next()
        .unwrap_or(file)
        .trim_end_matches(".rs");
    roots::join_dir(&dir, stem)
}

fn is_mod_rs(file: &str) -> bool {
    file == "mod.rs" || file.ends_with("/mod.rs")
}

/// The crate roots whose module tree can contain `from`: itself when
/// it IS a root, else the roots whose directory is the deepest
/// prefix (src/bin/x/helper.rs belongs to bin x, not to the lib that
/// also covers src/ — deepest-wins is Cargo semantics, not a pick).
fn covering_roots(from: &str, roots_set: &BTreeSet<String>) -> Vec<String> {
    if roots_set.contains(from) {
        return vec![from.to_string()];
    }
    let mut best: Vec<String> = Vec::new();
    let mut best_len = 0usize;
    for root in roots_set {
        let dir = roots::parent_dir(root);
        if !(dir.is_empty() || from.starts_with(&format!("{dir}/"))) {
            continue;
        }
        if best.is_empty() || dir.len() > best_len {
            best = vec![root.clone()];
            best_len = dir.len();
        } else if dir.len() == best_len {
            best.push(root.clone());
        }
    }
    best
}

/// k×super: each step maps every anchor to the file(s) owning its
/// parent directory; a crate root has no parent and drops out. All
/// owners of one directory share one child directory, so divergent
/// climbs can only differ at the terminal — walk_all's check.
fn climb(
    from: &str,
    ups: usize,
    roots_set: &BTreeSet<String>,
    files: &BTreeSet<String>,
) -> Vec<String> {
    let mut cur = BTreeSet::from([from.to_string()]);
    for _ in 0..ups {
        let mut next = BTreeSet::new();
        for f in cur.iter().filter(|f| !roots_set.contains(*f)) {
            let dir = if is_mod_rs(f) {
                roots::parent_dir(&roots::parent_dir(f))
            } else {
                roots::parent_dir(f)
            };
            next.extend(owners(&dir, roots_set, files));
        }
        cur = next;
        if cur.is_empty() {
            break;
        }
    }
    cur.into_iter().collect()
}

/// Files whose child directory is `dir`: dir/mod.rs, the sibling
/// <dir>.rs, and any crate root sitting directly in dir.
fn owners(dir: &str, roots_set: &BTreeSet<String>, files: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let modrs = roots::join_dir(dir, "mod.rs");
    if files.contains(&modrs) {
        out.insert(modrs);
    }
    let sibling = format!("{dir}.rs");
    if !dir.is_empty() && files.contains(&sibling) {
        out.insert(sibling);
    }
    for root in roots_set {
        if roots::parent_dir(root) == dir {
            out.insert(root.clone());
        }
    }
    out
}

/// R4: an in-scope Cargo package by normalized name resolves to its
/// lib root (a member whose root is not a scope file terminates here
/// — the TS workspace precedent); a declared dependency ⇒ External;
/// anything else is out of scope.
fn extern_rung(name: &str, pkg: Option<&cargo::Package>, scope: &Scope) -> Outcome {
    let members = super::members(scope, "Cargo.toml", cargo::package, |p| {
        p.name
            .as_deref()
            .is_some_and(|n| n.replace('-', "_") == name)
    });
    match members.len() {
        1 => match members[0].lib_root(scope.files) {
            Some(path) => Outcome::Resolved { path, rung: 4 },
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
