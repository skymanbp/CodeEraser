//! Markdown rungs (design §4 row 5). Doc targets resolve at FILE and
//! SECTION granularity. R1 joins a relative target against the
//! linking file's directory; a directory holding in-scope files
//! resolves as a package — the audited fourclass row: a tree
//! reference collapsed to one file would be a guess. R2 validates a
//! cross-file anchor against the target's ATX slug set by GitHub
//! rules (lowercase, punctuation dropped, spaces to hyphens, -N
//! duplicate suffixes); zero or multiple matches degrade to the file
//! (ResolvedSection { slug: None }). R3 reference links substitute
//! the in-file definition (CommonMark: labels case-fold and the
//! FIRST definition wins — spec, not a guess) and rerun the chain
//! relabeled rung 3; an unused definition is External — the design's
//! ledger-visible exclusion, because a definition that renders
//! nothing must not keep its target alive. R4 a bare fragment is an
//! in-file section claim taken AS WRITTEN, never validated: raw-HTML
//! anchors (`<h3 name=…>`) are legal GitHub targets we do not model
//! (the audited FAQ.md #complete row) — validating against ATX-only
//! slugs would refuse real anchors. R5 any URI scheme (http/https/
//! mailto per the design row) is External; site-root (/x) and
//! protocol-relative (//x) forms are domain-relative in rendered
//! contexts, never repo-relative.
//!
//! Honest limits: slugs are a char-level approximation of
//! rendered-text slugging (markdown syntax inside a heading drifts
//! the slug), and the heading walk is fence- and comment-aware but
//! not indented-code-aware — every approximation failure degrades an
//! anchor to file level, never invents a section. Cross-file
//! staleness is closed at the key (M5 close, repaying the 2f wiring
//! debt): the ONLY target-content fact this ladder consults is the
//! slug set (anchor() below), so every md file's slug_hash is a
//! resolve_key input — a heading edit anywhere shifts the key and
//! the phase-2 sweep re-validates every anchor.

use super::{Outcome, Reason, Scope};
use crate::graph::md::{content_lines, detect, ref_definition};
use crate::graph::roots;
use std::collections::{BTreeMap, BTreeSet};

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
    let Some(target) = roots::join_rel(&roots::parent_dir(from), path) else {
        return Outcome::Unresolved(Reason::OutOfScope);
    };
    if scope.files.contains(&target) {
        return match frag {
            Some(f) if target.ends_with(".md") && !f.is_empty() => anchor(target, f, scope),
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

/// R2: the fragment against the target's ATX slug set; anything but
/// exactly one match degrades to the file (slug: None).
fn anchor(target: String, frag: &str, scope: &Scope) -> Outcome {
    let text = std::fs::read_to_string(scope.root.join(&target)).unwrap_or_default();
    let hits = slug_set(&text).into_iter().filter(|s| s == frag).count();
    let slug = (hits == 1).then(|| frag.to_string());
    Outcome::ResolvedSection {
        path: target,
        slug,
        rung: 2,
    }
}

/// R3: substitute the definition and rerun the chain, relabeled —
/// the rung that answered is the reference machinery.
fn ref_link(from: &str, spec: &str, scope: &Scope) -> Outcome {
    let (defs, _) = refs(from, scope);
    match defs.get(&fold(spec)) {
        Some(target) => relabel(link(from, target, scope)),
        None => Outcome::Unresolved(Reason::OutOfScope),
    }
}

/// A definition site: used ⇒ its target resolves like a link (the
/// ref_link edge simply duplicates it); unused ⇒ External (module
/// header). A duplicate label's later definition is inert under
/// first-wins, so it lands in the unused arm.
fn ref_def(from: &str, spec: &str, scope: &Scope) -> Outcome {
    let (defs, used) = refs(from, scope);
    let live = defs
        .iter()
        .any(|(label, t)| t == spec && used.contains(label));
    if live {
        relabel(link(from, spec, scope))
    } else {
        Outcome::External { rung: 5 }
    }
}

fn refs(from: &str, scope: &Scope) -> (BTreeMap<String, String>, BTreeSet<String>) {
    let text = std::fs::read_to_string(scope.root.join(from)).unwrap_or_default();
    ref_table(&text)
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

/// The slug set folded to one resolve_key input (module header):
/// order-sensitive on purpose — duplicate -N suffixes shift with
/// order, and the set IS what anchor() consults.
pub fn slug_hash(text: &str) -> u64 {
    let mut buf = Vec::new();
    for slug in slug_set(text) {
        buf.extend_from_slice(slug.as_bytes());
        buf.push(b'\n');
    }
    crate::dedup::tokens::fnv1a(&buf)
}

/// GitHub-slugged ATX headings in document order, -N suffixes for
/// duplicates; fence- and comment-aware via the detector's walk.
fn slug_set(text: &str) -> Vec<String> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut out = Vec::new();
    for (_, line, mask) in content_lines(text) {
        if mask.first().copied().unwrap_or(false) {
            continue;
        }
        let Some(head) = atx_heading(line.trim_start()) else {
            continue;
        };
        let base = slugify(head);
        let n = seen.entry(base.clone()).or_insert(0usize);
        let slug = if *n == 0 { base } else { format!("{base}-{n}") };
        *n += 1;
        out.push(slug);
    }
    out
}

/// ATX heading text: 1-6 leading #, then a space or the end; the
/// space-separated closing sequence strips (CommonMark).
fn atx_heading(trimmed: &str) -> Option<&str> {
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &trimmed[hashes..];
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    Some(rest.trim().trim_end_matches('#').trim_end())
}

/// GitHub slug: lowercase, keep letters/digits/_/-, spaces become
/// hyphens, everything else drops (verified against the audited
/// Hangul, backtick and parenthesis ground-truth rows).
fn slugify(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            out.extend(c.to_lowercase());
        } else if c == ' ' {
            out.push('-');
        }
    }
    out
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
