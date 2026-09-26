//! HTML rungs (plan v2.30 step 5; booklet §8, HTML row). A page names
//! what the site serves, so a target is any walked path — a page, a
//! document, a code file, or an asset the index never holds
//! (Scope::assets) — and a directory serves its index page. R1 joins a
//! relative reference onto the document's directory (the directory its
//! `<base href>` names, when it declares one). A root-relative
//! reference (`/x`) needs the deployment root, which no join knows: R2
//! takes it from a declared `[graph.search_roots] html` directory, or
//! from the document's own served URL (html_head.rs: `/how/` on
//! `site/how/index.html` puts the root at `site`); R3, with neither,
//! asks each ancestor directory of the page — the root serves the page,
//! so it is one of them — and answers when exactly one holds the path
//! (two is ambiguous_root, none out_of_scope). An absolute URL on the
//! document's own host is that page's root-relative path; any other
//! host or scheme is External (R5, md's reading). R4 is md's too: a
//! bare fragment claims a section of the page as written, `#` alone
//! the page. A cross-page fragment is validated against the target
//! page's `id` set (a Markdown target's slug set), zero or many
//! matches degrading to the file. The query is dropped (a static site
//! serves one file per path), character and percent escapes decode
//! before any lookup, and an empty reference is the document's own URL
//! (the URL Standard): a page or form reference lands the page, while
//! an empty `src`, `srcset` or asset `<link>` fetches nothing — HTML's
//! img, script and link algorithms return on an empty value — and is
//! the ledger's `empty` row (O60).

use super::html_head::{self as head, Document};
use super::md::{self, slug::percent_decode};
use super::{Outcome, Reason, Rung, Scope, Site};
use crate::graph::roots::{join_dir, join_rel, parent_dir};
use crate::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::Path;
use std::rc::Rc;

pub fn resolve(site: &Site, scope: &Scope) -> Outcome {
    let from = site.from;
    if site.spec.is_empty() {
        return match site.kind {
            "href" | "action" => md::fragment(from, None),
            _ => Outcome::Unresolved(Reason::Empty),
        };
    }
    let spec = head::decode_refs(site.spec);
    let doc = document(from, scope);
    if let Some((host, path)) = head::split_origin(&spec) {
        if doc.host.as_deref() != Some(host.as_str()) {
            return Outcome::External { rung: 5 };
        }
        let path = if path.is_empty() { "/" } else { path };
        return located(from, path, &doc, scope);
    }
    if md::is_scheme(&spec) {
        return Outcome::External { rung: 5 };
    }
    located(from, &spec, &doc, scope)
}

/// A reference without an origin: its fragment split off, its query
/// dropped, the path found (R1–R3) and the fragment settled (R2/R4).
fn located(from: &str, reference: &str, doc: &Document, scope: &Scope) -> Outcome {
    let (path, frag) = match reference.split_once('#') {
        Some((p, f)) => (p, Some(f)),
        None => (reference, None),
    };
    let path = path.split('?').next().unwrap_or("");
    if path.is_empty() {
        return md::fragment(from, frag);
    }
    let found = match path.strip_prefix('/') {
        Some(rel) => rooted(from, rel, doc, scope, &|p| candidate(p, scope)),
        None => relative(from, path, doc, scope),
    };
    match found {
        Ok((target, rung)) => sectioned(target, frag, rung, scope),
        Err(terminal) => terminal,
    }
}

/// A rung's answer: the found path and the rung, or the terminal
/// outcome that stands instead (a refusal, or External for a reference
/// a `<base>` on another host sends off the site).
type Found = Result<(String, Rung), Outcome>;

/// R1: the reference joined onto the document's directory — its
/// `<base href>` first, itself a reference the rungs settle to a
/// directory.
fn relative(from: &str, path: &str, doc: &Document, scope: &Scope) -> Found {
    let dir = match doc.base.as_deref() {
        None => parent_dir(from),
        Some(base) => base_dir(from, base, doc, scope)?,
    };
    let refused = Outcome::Unresolved(Reason::OutOfScope);
    let target = join_rel(&dir, &percent_decode(path)).ok_or(refused.clone())?;
    candidate(&target, scope).map(|t| (t, 1)).ok_or(refused)
}

/// The directory a `<base href>` names in the tree: a relative base
/// beside the document, a root-relative or same-host absolute one
/// under the deployment root (R2/R3 as for a reference), the directory
/// part of either — out of scope when no walked file lives under it; a
/// base on another host sends every relative reference off the site,
/// so those are External.
fn base_dir(from: &str, base: &str, doc: &Document, scope: &Scope) -> Result<String, Outcome> {
    let dir_of = |p: &str| p.rsplit_once('/').map_or("", |(d, _)| d).to_string();
    let exists = |d: &str| directory(d, scope).then(|| d.to_string());
    let under_root = |rel: &str| rooted(from, &dir_of(rel), doc, scope, &exists).map(|(d, _)| d);
    if let Some((host, path)) = head::split_origin(base) {
        if doc.host.as_deref() != Some(host.as_str()) {
            return Err(Outcome::External { rung: 5 });
        }
        return under_root(path.trim_start_matches('/'));
    }
    if md::is_scheme(base) {
        return Err(Outcome::External { rung: 5 });
    }
    match base.strip_prefix('/') {
        Some(rel) => under_root(rel),
        None => join_rel(&parent_dir(from), &dir_of(base))
            .and_then(|d| exists(&d))
            .ok_or(Outcome::Unresolved(Reason::OutOfScope)),
    }
}

/// R2/R3 for a root-relative path: under the declared `html` roots
/// (two answering is ambiguous_root), else under the root the page's
/// own URL derives, else under the one ancestor directory of the page
/// that holds it. `found` is what "holds it" means — a walked file or
/// index page (candidate) for a reference, a directory for a base.
fn rooted(
    from: &str,
    rel: &str,
    doc: &Document,
    scope: &Scope,
    found: &dyn Fn(&str) -> Option<String>,
) -> Found {
    let rel = percent_decode(rel);
    let under = |dir: &str| join_rel(dir, &rel).and_then(|p| found(&p));
    if let Some(dirs) = scope.search_roots.get("html") {
        return unique(dirs.iter().filter_map(|d| under(d)).collect(), 2);
    }
    if let Some(root) = &doc.root {
        return unique(under(root).into_iter().collect(), 2);
    }
    let dir = parent_dir(from);
    let ancestors = dir
        .match_indices('/')
        .map(|(i, _)| &dir[..i])
        .chain([dir.as_str(), ""]);
    unique(ancestors.filter_map(under).collect(), 3)
}

/// A rung's distinct candidates as its answer: one answers, two is one
/// path in two roots, none leaves the reference out of scope.
fn unique(hits: BTreeSet<String>, rung: Rung) -> Found {
    let mut hits = hits.into_iter();
    match (hits.next(), hits.next()) {
        (Some(path), None) => Ok((path, rung)),
        (Some(_), Some(_)) => Err(Outcome::Unresolved(Reason::AmbiguousRoot)),
        (None, _) => Err(Outcome::Unresolved(Reason::OutOfScope)),
    }
}

/// The walked path a tree path serves: itself, or its index page when
/// it is a directory (the tree root included) — never a file the walk
/// did not see.
fn candidate(path: &str, scope: &Scope) -> Option<String> {
    if scope.files.contains(path) || scope.assets.contains(path) {
        return Some(path.to_string());
    }
    ["index.html", "index.htm"]
        .iter()
        .map(|index| join_dir(path, index))
        .find(|p| scope.files.contains(p))
}

/// Whether walked files live under a directory (the tree root always).
fn directory(dir: &str, scope: &Scope) -> bool {
    if dir.is_empty() {
        return true;
    }
    let prefix = format!("{dir}/");
    scope
        .files
        .iter()
        .chain(scope.assets)
        .any(|f| f.starts_with(&prefix))
}

/// The fragment on a found target: a page's `id` set (or a Markdown
/// document's slug set) validates it — exactly one match lands the
/// section, anything else the file (R2, md's reading); a fragment on
/// any other target is a view detail, dropped.
fn sectioned(target: String, frag: Option<&str>, rung: Rung, scope: &Scope) -> Outcome {
    let Some(frag) = frag.filter(|f| !f.is_empty()) else {
        return Outcome::Resolved { path: target, rung };
    };
    match Lang::from_path(Path::new(&target)) {
        Some(Lang::Markdown) => md::anchor(target, frag, scope),
        Some(Lang::Html) => {
            let ids = scope
                .memo
                .cached("html_ids", &target, || head::ids(&text_of(scope, &target)));
            let frag = percent_decode(frag);
            let hits = ids.iter().filter(|id| **id == frag).count();
            Outcome::ResolvedSection {
                path: target,
                slug: (hits == 1).then_some(frag),
                rung: 2,
            }
        }
        _ => Outcome::Resolved { path: target, rung },
    }
}

/// The document's own facts, read once per sweep (the memo).
fn document(from: &str, scope: &Scope) -> Rc<Document> {
    scope
        .memo
        .cached("html_doc", from, || head::read(from, &text_of(scope, from)))
}

fn text_of(scope: &Scope, path: &str) -> String {
    std::fs::read_to_string(scope.root.join(path)).unwrap_or_default()
}
