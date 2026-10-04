//! Resolution ladder (design brief §4): a site walks rungs in order
//! and the FIRST rung producing exactly one in-scope candidate
//! resolves it; more than one candidate at a rung is
//! Unresolved(ambiguous_*) — picking a "best" one would invent a
//! path. External (stdlib / registry / dependency) is a correct
//! terminal answer, not a miss. Every resolved edge records its
//! rung, so precision is attributable per level and a dirty rung can
//! be voted out by data at 2h.
//!
//! All six launch ladders have landed (TS → Py → Rust → Go → Md → Hs),
//! the C family's followed in plan v2.30 step 2, Java's in step 3
//! (java.rs), Lua's and R's in step 4 (r/), HTML's in step 5 (html.rs);
//! a language without rungs must return Unresolved(Unsupported) — an
//! honest ledger row, never a silent skip. Since plan v2.33 wave W2a
//! the Python, Lua, Go and C / C++ rungs live in the core
//! (`resolve/1`, graph/resolve/): `resolve_all` sends their sites in
//! one request and runs the other languages' rungs here. Dispatch
//! carries the site's frozen kind label (the package's
//! `store.site_kinds`): the TS rungs are kind-uniform, Rust's mod_decl
//! and use walk different rungs, and Markdown routes five kinds through
//! one chain.

use crate::scan::lang::Lang;
use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::rc::Rc;

// pub: the walk reads every C-family file's include list with it
pub mod c_head;
pub mod hs;
pub mod java;
// pub: the walk reads every Java header with it (dedup/walkidx.rs)
pub mod html;
pub mod java_header;
mod java_sets;
// pub: the walk hashes every page's id set with it (dedup/walkidx.rs)
pub mod html_head;
// pub: the walk reads every Lua file's package.path templates with it
// (dedup/walkidx.rs)
pub mod lua_path;
pub mod md;
mod paths;
pub mod r;
pub mod rs;
// pub: walkidx feeds pubuse_hash into resolve_key (the slug-hash
// discipline for the binder's cross-file input)
pub mod rs_reexport;
mod rs_tree;
pub mod ts;
// the site outcome vocabulary (a leaf: it reads nothing of this module)
mod outcome;
pub use outcome::{Outcome, Reason, Rung};
// The a8db74a9 Python / Lua / Go / C rungs, frozen byte for byte: the
// differential gate's oracle (tests subrepo unit/graph/ladder/oracle/,
// driven by unit/dedup/ladder_diff/).
#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/frozen.rs"]
pub(crate) mod frozen;

/// What a resolver may consult. Candidate targets MUST come from
/// `files` (the frozen in-scope set); `configs` are the resolver
/// config paths the walk collected (all in resolve_key, store.rs);
/// raw fs access via `root` may only justify External, block a
/// rewrite, read an in-scope candidate's own bytes to refine
/// granularity (Md section slugs, R3 definition tables), or read a
/// walk TERMINAL's own bytes for the one-hop re-export bind — never
/// mint an in-scope candidate. `crate_roots` are the declared Rust
/// crate roots (ce.toml `[graph] crate_roots` ∩ files) the Rust ladder
/// unions with the manifest's — a tree that is a slice of a package
/// elsewhere has no manifest to read. `search_roots` is the declared
/// `[graph.search_roots]` table (language name → directories ∩ the
/// walked tree, plan v2.30): the rung a language's ladder walks after
/// the site's own directory and before its build configuration. `java`
/// is every walked Java file's header (java_header.rs) — the package it
/// declares and the imports it writes, read by the walk so the Java
/// ladder reads no file (plan v2.30 step 3); `lua` the templates the
/// walked Lua files assign to `package.path` (lua_path.rs, step 4).
/// `assets` are the walked files the index never holds — no judged
/// language: images, styles, scripts, fonts, data — the HTML rungs'
/// second candidate set (step 5; the walk lists them and the key
/// hashes them, so a target is still never minted from the
/// filesystem). `includes` is each walked C-family file's include
/// list (c_head.rs), the compile database closure's input (step 5b).
pub struct Scope<'a> {
    pub files: &'a BTreeSet<String>,
    pub assets: &'a BTreeSet<String>,
    pub configs: &'a [String],
    pub root: &'a Path,
    pub memo: &'a Memo,
    pub crate_roots: &'a BTreeSet<String>,
    pub search_roots: &'a BTreeMap<String, BTreeSet<String>>,
    pub java: &'a BTreeMap<String, java_header::Header>,
    pub lua: &'a BTreeSet<lua_path::Template>,
    pub includes: &'a BTreeMap<String, Vec<String>>,
}

/// The memo's slot table, aliased so the shape reads once.
type Slots = RefCell<HashMap<(&'static str, String), Rc<dyn Any>>>;

/// Per-sweep memo (M5-close review MED: every site re-parsed its
/// configs and re-derived root sets — O(sites × files) with a
/// tree-sitter or TOML parse per site). One keyed any-cache carried
/// by the Scope: a sweep sees a QUIESCENT tree (the resolve_key gate
/// / phase-1.5 dirty set), so caching derived structures for one
/// sweep's lifetime cannot serve stale answers.
#[derive(Default)]
pub struct Memo(Slots);

impl Memo {
    /// The one cache throat: compute once per (namespace, key). A
    /// namespace always stores one concrete type; the downcast
    /// therefore only misses on a programming error, and falling
    /// through to a rebuild keeps even that case correct.
    pub fn cached<T: 'static>(
        &self,
        ns: &'static str,
        key: &str,
        build: impl FnOnce() -> T,
    ) -> Rc<T> {
        if let Some(hit) = self.0.borrow().get(&(ns, key.to_string()))
            && let Ok(t) = hit.clone().downcast::<T>()
        {
            return t;
        }
        let made = Rc::new(build());
        self.0
            .borrow_mut()
            .insert((ns, key.to_string()), made.clone() as Rc<dyn Any>);
        made
    }
}

/// One reference site as the ladder consumes it — the CachedSite
/// projection that travels the dispatcher. `kind` is the frozen
/// label; `from` is repo-relative with forward slashes; `line` is
/// the 1-based source line — only Rust consumes it (inline-module
/// depth anchors self/super), the other ladders are line-free.
pub struct Site<'a> {
    pub kind: &'a str,
    pub from: &'a str,
    pub spec: &'a str,
    pub line: usize,
}

/// Dispatch one site to its language ladder (`resolve_all` of one).
pub fn resolve(lang: Lang, site: &Site, scope: &Scope) -> Result<Outcome, String> {
    let mut out = resolve_all(&[(lang, site)], scope)?;
    Ok(out.pop().expect("one site in, one outcome out"))
}

/// Dispatch sites to their language ladders, outcomes in site order:
/// the languages the core holds go in one resolve/1 request, the rest
/// walk this side's rungs. An empty specifier is refused here by name
/// before any ladder sees it — a bare-package rung or a package-root
/// lookup would otherwise read `""` as a name; Markdown and HTML keep
/// their own reading of an empty target (the document itself; html.rs
/// tells a page reference from an asset fetch of nothing).
pub fn resolve_all(sites: &[(Lang, &Site)], scope: &Scope) -> Result<Vec<Outcome>, String> {
    let empty = |lang: Lang, site: &Site| {
        site.spec.is_empty() && !matches!(lang, Lang::Markdown | Lang::Html)
    };
    let core: Vec<(Lang, &Site)> = sites
        .iter()
        .filter(|(lang, site)| !empty(*lang, site) && super::resolve::in_core(*lang))
        .copied()
        .collect();
    let mut answered = super::resolve::outcomes(&core, scope)?.into_iter();
    Ok(sites
        .iter()
        .map(|(lang, site)| {
            if empty(*lang, site) {
                Outcome::Unresolved(Reason::Empty)
            } else if super::resolve::in_core(*lang) {
                answered.next().expect("one outcome per core site")
            } else {
                here(*lang, site, scope)
            }
        })
        .collect())
}

/// The rungs this side still runs.
fn here(lang: Lang, site: &Site, scope: &Scope) -> Outcome {
    match lang {
        Lang::TypeScript | Lang::Tsx => ts::resolve(site.from, site.spec, scope),
        Lang::Rust => rs::resolve(site, scope),
        Lang::Markdown => md::resolve(site, scope),
        Lang::Haskell => hs::resolve(site.from, site.spec, scope),
        Lang::Java => java::resolve(site, scope),
        Lang::R => r::resolve(site, scope),
        Lang::Html => html::resolve(site, scope),
        // The sentinel is never walked, and the scan-only arm (plan
        // v2.5) is never indexed — if either ever arrives, the honest
        // answer is the documented no-rungs stance, never a guess.
        _ => Outcome::Unresolved(Reason::Unsupported),
    }
}

/// Shared workspace-member throat for the R4 rungs: the in-scope
/// configs of one basename, parsed once per sweep (the memo), and
/// filtered by name — each caller judges the hit count (1 = the
/// member, more = its own ambiguity reason).
pub(crate) fn members<T: Clone + 'static>(
    scope: &Scope,
    basename: &'static str,
    load: impl Fn(&Path, &str) -> Option<T>,
    keep: impl Fn(&T) -> bool,
) -> Vec<T> {
    scope
        .configs
        .iter()
        .filter(|c| c.rsplit('/').next() == Some(basename))
        .filter_map(|c| {
            scope
                .memo
                .cached(basename, c, || load(scope.root, c))
                .as_ref()
                .clone()
        })
        .filter(|t| keep(t))
        .collect()
}
