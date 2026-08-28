//! Markdown rungs (design §4 row 5). Doc targets resolve at FILE and
//! SECTION granularity. R1 joins a relative target against the
//! linking file's directory; a directory holding in-scope files
//! resolves as a package — the audited fourclass row: a tree
//! reference collapsed to one file would be a guess. R2 validates a
//! cross-file anchor, percent-decoded, against the target's anchor
//! set (md_slug.rs: rendered-text slugs of the ATX headings by GitHub
//! rules — lowercase, punctuation dropped, spaces to hyphens, -N
//! duplicate suffixes — plus raw-HTML anchor ids verbatim); zero or
//! multiple matches degrade to the file (ResolvedSection { slug:
//! None }). R3 reference links substitute
//! the in-file definition (CommonMark: labels case-fold and the
//! FIRST definition wins — spec, not a guess) and rerun the chain
//! relabeled rung 3; an unused definition is deliberately EXCLUDED
//! from liveness — a definition that renders nothing must not keep
//! its target alive (user decision D3). Since 2.29.0 (H1 slice 16,
//! the executed post-1.0 disposition) the exclusion is the CORE's:
//! an in-scope unused definition resolves and travels as an INERT
//! edge (EDGE_REFDEF_UNUSED) that liveness drops beside the asset
//! kind, and an unresolvable one is an ordinary miss — the borrowed
//! External category retired with the channel that replaced it
//! (Outcome::ResolvedInert). R4 a bare fragment is an
//! in-file section claim taken AS WRITTEN, never validated: a claim
//! on the linking document itself is an in-file edge whatever it
//! names (the audited FAQ.md #complete row's raw-HTML anchor is one
//! such — modeled for cross-file lookups since step 8, still not a
//! reason to refuse an in-file claim). R5 any URI scheme (http/https/
//! mailto per the design row) is External; site-root (/x) and
//! protocol-relative (//x) forms are domain-relative in rendered
//! contexts, never repo-relative.
//!
//! Honest limits (plan v2.17 L round step 8, O57, closed four: the
//! slug is the rendered heading's, the heading walk is
//! indented-code-aware, raw-HTML anchors enter the set, percent
//! escapes decode — md_slug.rs): what remains — setext headings,
//! attribute forms the one-line tag reader does not parse — degrades
//! an anchor to file level, never invents a section. Cross-file
//! staleness is closed at the key (M5 close, repaying the 2f wiring
//! debt): the ONLY target-content fact this ladder consults is the
//! anchor set (anchor() below), so every md file's slug_hash is a
//! resolve_key input — a heading edit anywhere shifts the key and
//! the phase-2 sweep re-validates every anchor.

use super::{Outcome, Reason, Scope};
use crate::graph::md::{content_lines, detect, ref_definition};
use crate::graph::roots;
use slug::{percent_decode, slug_set};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[path = "md_slug.rs"]
mod slug;
pub use slug::slug_hash;

pub fn resolve(kind: &str, from: &str, spec: &str, scope: &Scope) -> Outcome {
    match kind {
        "link" | "image" => link(from, spec, scope),
        "ref_link" => ref_link(from, spec, scope),
        "ref_def" => ref_def(from, spec, scope),
        "url" => Outcome::External { rung: 5 },
        _ => Outcome::Unresolved(Reason::Unsupported),
    }
}

/// R1/R2/R5 chain for a direct target — link and image share it (the
/// asset exclusion of images is edge-kind wiring, not resolution).
fn link(from: &str, spec: &str, scope: &Scope) -> Outcome {
    if is_scheme(spec) {
        return Outcome::External { rung: 5 };
    }
    if spec.starts_with('/') {
        return Outcome::Unresolved(Reason::OutOfScope);
    }
    let (path, frag) = match spec.split_once('#') {
        Some((p, f)) => (p, Some(f)),
        None => (spec, None),
    };
    if path.is_empty() {
        return fragment(from, frag);
    }
    // decoded AFTER the split: an encoded `%23` is a literal `#` in
    // the file name, never a fragment delimiter
    let Some(target) = roots::join_rel(&roots::parent_dir(from), &percent_decode(path)) else {
        return Outcome::Unresolved(Reason::OutOfScope);
    };
    if scope.files.contains(&target) {
        return match frag {
            Some(f) if crate::graph::md::is_md_path(&target) && !f.is_empty() => {
                anchor(target, f, scope)
            }
            _ => Outcome::Resolved {
                path: target,
                rung: 1,
            },
        };
    }
    directory(target, scope)
}

/// R4: a bare fragment names a section of the linking document, as
/// written — there is nothing to validate against (module header).
fn fragment(from: &str, frag: Option<&str>) -> Outcome {
    match frag {
        Some(f) if !f.is_empty() => Outcome::ResolvedSection {
            path: from.to_string(),
            slug: Some(f.to_string()),
            rung: 4,
        },
        _ => Outcome::Resolved {
            path: from.to_string(),
            rung: 4,
        },
    }
}

/// R1's directory arm: a tree reference resolves as a package while
/// in-scope files live under it.
fn directory(target: String, scope: &Scope) -> Outcome {
    let prefix = if target.is_empty() {
        String::new()
    } else {
        format!("{target}/")
    };
    if scope.files.iter().any(|f| f.starts_with(&prefix)) {
        Outcome::ResolvedPackage {
            dir: target,
            rung: 1,
        }
    } else {
        Outcome::Unresolved(Reason::OutOfScope)
    }
}

/// R2: the fragment, percent-decoded, against the target's anchor
/// set; anything but exactly one match degrades to the file (slug:
/// None).
fn anchor(target: String, frag: &str, scope: &Scope) -> Outcome {
    // per-sweep: one read + slug pass per TARGET file, not one per
    // anchored link pointing at it (review MED)
    let slugs = scope.memo.cached("md_slugs", &target, || {
        let text = std::fs::read_to_string(scope.root.join(&target)).unwrap_or_default();
        slug_set(&text)
    });
    let frag = percent_decode(frag);
    let hits = slugs.iter().filter(|s| **s == frag).count();
    let slug = (hits == 1).then_some(frag);
    Outcome::ResolvedSection {
        path: target,
        slug,
        rung: 2,
    }
}

/// R3: substitute the definition and rerun the chain, relabeled —
/// the rung that answered is the reference machinery.
fn ref_link(from: &str, spec: &str, scope: &Scope) -> Outcome {
    let table = refs(from, scope);
    match table.0.get(&fold(spec)) {
        Some(target) => relabel(link(from, target, scope)),
        None => Outcome::Unresolved(Reason::OutOfScope),
    }
}

/// A definition site: used ⇒ its target resolves like a link (the
/// ref_link edge simply duplicates it); unused ⇒ the target still
/// resolves but travels INERT (H1 slice 16, 2.29.0) — the core owns
/// the liveness exclusion, and an unresolvable unused definition is
/// an ordinary miss now, never the borrowed External category. A
/// duplicate label's later definition is inert under first-wins, so
/// it lands in the unused arm.
fn ref_def(from: &str, spec: &str, scope: &Scope) -> Outcome {
    let table = refs(from, scope);
    let (defs, used) = (&table.0, &table.1);
    let live = defs
        .iter()
        .any(|(label, t)| t == spec && used.contains(label));
    let resolved = relabel(link(from, spec, scope));
    if live {
        return resolved;
    }
    match resolved {
        Outcome::Resolved { path, rung } | Outcome::ResolvedSection { path, rung, .. } => {
            Outcome::ResolvedInert { path, rung }
        }
        Outcome::ResolvedPackage { dir, rung } => Outcome::ResolvedInert { path: dir, rung },
        other => other,
    }
}

/// Per-sweep: every ref site in one file used to re-read it AND
/// re-run the full detector walk (review MED).
fn refs(from: &str, scope: &Scope) -> Rc<(BTreeMap<String, String>, BTreeSet<String>)> {
    scope.memo.cached("md_refs", from, || {
        let text = std::fs::read_to_string(scope.root.join(from)).unwrap_or_default();
        ref_table(&text)
    })
}

/// The in-file reference surface: first-win definitions by folded
/// label, plus every label a reference link actually uses — built on
/// the detector's own walk so the masking cannot drift (a fenced or
/// commented definition must resolve nothing).
fn ref_table(text: &str) -> (BTreeMap<String, String>, BTreeSet<String>) {
    let mut defs: BTreeMap<String, String> = BTreeMap::new();
    for (_, line, mask) in content_lines(text) {
        if !mask.first().copied().unwrap_or(false)
            && let Some((label, target)) = ref_definition(line)
        {
            defs.entry(fold(label))
                .or_insert_with(|| target.to_string());
        }
    }
    let used = detect(text)
        .into_iter()
        .filter(|s| s.kind == "ref_link")
        .map(|s| fold(&s.spec))
        .collect();
    (defs, used)
}

/// CommonMark label normalization: case fold + whitespace collapse.
fn fold(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn relabel(outcome: Outcome) -> Outcome {
    match outcome {
        Outcome::Resolved { path, .. } => Outcome::Resolved { path, rung: 3 },
        Outcome::ResolvedSection { path, slug, .. } => Outcome::ResolvedSection {
            path,
            slug,
            rung: 3,
        },
        Outcome::ResolvedPackage { dir, .. } => Outcome::ResolvedPackage { dir, rung: 3 },
        Outcome::External { .. } => Outcome::External { rung: 3 },
        unresolved => unresolved,
    }
}

/// An RFC 3986 scheme head (or a protocol-relative // form) leaves
/// the corpus before any join runs — join_rel would silently
/// normalize "//host/x" into a bogus in-tree path.
fn is_scheme(spec: &str) -> bool {
    if spec.starts_with("//") {
        return true;
    }
    let head = spec.split(['/', '#']).next().unwrap_or("");
    let Some((scheme, _)) = head.split_once(':') else {
        return false;
    };
    scheme
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

#[cfg(test)]
#[path = "md_tests.rs"]
mod tests;
