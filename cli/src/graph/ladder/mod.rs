//! Resolution ladder (design brief §4): a site walks rungs in order
//! and the FIRST rung producing exactly one in-scope candidate
//! resolves it; more than one candidate at a rung is
//! Unresolved(ambiguous_*) — picking a "best" one would invent a
//! path. External (stdlib / registry / dependency) is a correct
//! terminal answer, not a miss. Every resolved edge records its
//! rung, so precision is attributable per level and a dirty rung can
//! be voted out by data at 2h.
//!
//! Every rung of every language with a ladder runs in the core
//! (`resolve/1`, graph/resolve/; plan v2.33 wave W2a and W2-text stages
//! A–H). This side detects the sites and reads what the rungs read — the
//! configuration files as text, a Rust file's syntax-tree facts
//! (rs_cst.rs), a document's anchors and definitions and a page's head
//! facts (graph/resolve/markdown.rs, html_head.rs) — and `resolve_all`
//! sends a sweep's sites in one request over the process's one core
//! session (corelink), asking again only for the facts the next resolve
//! level names: one round per level (stage Z). Dispatch carries the
//! site's frozen kind label (the package's `store.site_kinds`; Markdown
//! routes five kinds through one chain); a language without rungs
//! returns Unresolved(Unsupported) — an honest ledger row, never a silent
//! skip.

use crate::scan::lang::Lang;
use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::rc::Rc;

// pub: the walk reads every C-family file's include list with it
pub mod c_head;
// pub: the walk reads every Java header with it (dedup/walkidx.rs)
pub mod java_header;
// pub: the walk hashes every page's id set with it (dedup/walkidx.rs);
// the request reads every page's head facts with it
// (graph/resolve/markdown.rs)
pub mod html_head;
// pub: the walk reads every Lua file's package.path templates with it
// (dedup/walkidx.rs)
pub mod lua_path;
pub mod md;
// pub: walkidx feeds pubuse_hash into resolve_key (the slug-hash
// discipline for the binder's cross-file input); the request reads the
// two CST facts with it (graph/resolve/request.rs)
pub mod rs_cst;
// the site outcome vocabulary (a leaf: it reads nothing of this module)
mod outcome;
pub use outcome::{Outcome, Reason, Rung};
// The a8db74a9 Python / Lua / Go / C rungs, the c96ab3f6 R rungs, the
// 27d0d56d Java rungs, the fa83a48d Haskell rungs, the dd0eec61 TS rungs,
// the 1324c927 Rust rungs, the 3b7234eb Markdown rungs and the a0cb6e13
// HTML rungs, frozen byte for byte: the differential
// gate's oracle (tests subrepo unit/graph/ladder/oracle/, driven by
// unit/dedup/ladder_diff/). The frozen Rust tree walk and re-export
// surface stand at their old paths, where the frozen rungs read them.
#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/frozen.rs"]
pub(crate) mod frozen;
#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/oracle/rs_reexport.rs"]
pub(crate) mod rs_reexport;
#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/oracle/rs_tree.rs"]
mod rs_tree;
#[cfg(test)]
pub(crate) use frozen::members;

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
/// declares, the imports it writes and its type declarations, read by
/// the walk so the Java rungs read no file (plan v2.30 step 3; the
/// request carries them to the core's rungs since v2.33 W2-text stage C); `lua` the templates the
/// walked Lua files assign to `package.path` (lua_path.rs, step 4).
/// `assets` are the walked files the index never holds — no judged
/// language: images, styles, scripts, fonts, data — the HTML and
/// Markdown rungs' second candidate set (step 5; the walk lists them and the key
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
/// the 1-based source line — the core's Rust rungs read it (the
/// syntax tree's answers at that row: inline-module depth anchors
/// self/super) and its Java rungs (the header's import on that line,
/// the types enclosing it); the others are line-free.
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
/// the languages the core holds go in one resolve/1 request, any other
/// is refused as unsupported. An empty specifier is refused here by name
/// before any ladder sees it — a bare-package rung or a package-root
/// lookup would otherwise read `""` as a name; Markdown and HTML keep
/// their own reading of an empty target (the document itself; the HTML
/// rungs tell a page reference from an asset fetch of nothing).
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
                // The sentinel is never walked, and the scan-only arm
                // (plan v2.5) is never indexed — if either ever arrives,
                // the honest answer is the documented no-rungs stance,
                // never a guess.
                Outcome::Unresolved(Reason::Unsupported)
            }
        })
        .collect())
}
