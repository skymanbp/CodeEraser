//! Resolution ladder (design brief §4): a site walks rungs in order
//! and the FIRST rung producing exactly one in-scope candidate
//! resolves it; more than one candidate at a rung is
//! Unresolved(ambiguous_*) — picking a "best" one would invent a
//! path. External (stdlib / registry / dependency) is a correct
//! terminal answer, not a miss. Every resolved edge records its
//! rung, so precision is attributable per level and a dirty rung can
//! be voted out by data at 2h.
//!
//! All six ladders have landed (TS → Py → Rust → Go → Md → Hs); a
//! future language without rungs must return Unresolved(Unsupported)
//! — an honest ledger row, never a silent skip. Dispatch carries the
//! site's frozen kind label (store::KINDS): the TS/Py rungs are
//! kind-uniform, Rust's mod_decl and use walk different rungs, and
//! Markdown routes five kinds through one chain.

use crate::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::Path;

pub mod go;
pub mod hs;
// pub: the regen_tables drift check re-derives the BOOT table from
// the toolchain (M5-close audit D8 — reproducible regeneration)
pub mod hs_boot;
pub mod md;
pub mod py;
pub mod rs;
mod rs_tree;
pub mod ts;

/// Which rung answered (1-based per the design §4 table); stored on
/// every edge — ammunition for the per-level cut table.
pub type Rung = u8;

/// Unresolved reasons — the frozen design §4 vocabulary. `Dynamic`
/// and `Macro` stay structurally empty so far: the site detector
/// never opens dynamic imports or macro output (py.rs / rs.rs module
/// headers state each mechanism).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Dynamic,
    AmbiguousPaths,
    AmbiguousRoot,
    AmbiguousWorkspace,
    AmbiguousExports,
    Macro,
    ConfigDepth,
    OutOfScope,
    Unsupported,
}

/// Terminal state of one site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Exactly one in-scope target (repo-relative, forward slashes).
    Resolved {
        path: String,
        rung: Rung,
    },
    /// Exactly one in-scope PACKAGE directory (Go import, Markdown
    /// directory link): the node identity is (pkg_dir, "") and
    /// granularity is package — collapsing to a single file would be
    /// a guess (design §4 row 4).
    ResolvedPackage {
        dir: String,
        rung: Rung,
    },
    /// Markdown doc target (design §4 row 5): node identity is
    /// (path, slug), granularity section. `slug: None` is the
    /// anchored link whose section claim could not be confirmed
    /// against the target's ATX slug set — the design's "degrade to
    /// file level + ambiguous_anchor": the edge lands file-level
    /// (dst_unit ""), and the refusal to guess a section stays
    /// visible here instead of vanishing into a plain Resolved.
    ResolvedSection {
        path: String,
        slug: Option<String>,
        rung: Rung,
    },
    /// Outside the corpus by design (registry dep, node_modules).
    External {
        rung: Rung,
    },
    Unresolved(Reason),
}

/// What a resolver may consult. Candidate targets MUST come from
/// `files` (the frozen in-scope set); `configs` are the resolver
/// config paths the walk collected (all in resolve_key, store.rs);
/// raw fs access via `root` may only justify External, block a
/// rewrite, or read an in-scope candidate's own bytes to refine
/// granularity (Md section slugs, R3 definition tables) — never
/// mint an in-scope candidate.
pub struct Scope<'a> {
    pub files: &'a BTreeSet<String>,
    pub configs: &'a [String],
    pub root: &'a Path,
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

/// Dispatch one site to its language ladder.
pub fn resolve(lang: Lang, site: &Site, scope: &Scope) -> Outcome {
    match lang {
        Lang::TypeScript | Lang::Tsx => ts::resolve(site.from, site.spec, scope),
        Lang::Python => py::resolve(site.from, site.spec, scope),
        Lang::Rust => rs::resolve(site, scope),
        Lang::Go => go::resolve(site.from, site.spec, scope),
        Lang::Markdown => md::resolve(site.kind, site.from, site.spec, scope),
        Lang::Haskell => hs::resolve(site.from, site.spec, scope),
        // The sentinel is never walked at all; if it ever is, the
        // honest answer is the documented no-rungs stance, never a
        // guess.
        Lang::LangUnknown => Outcome::Unresolved(Reason::Unsupported),
    }
}

/// Shared workspace-member throat for the R4 rungs: the in-scope
/// configs of one basename, parsed and filtered by name — each
/// caller judges the hit count (1 = the member, more = its own
/// ambiguity reason).
pub(crate) fn members<T>(
    scope: &Scope,
    basename: &str,
    load: impl Fn(&Path, &str) -> Option<T>,
    keep: impl Fn(&T) -> bool,
) -> Vec<T> {
    scope
        .configs
        .iter()
        .filter(|c| c.rsplit('/').next() == Some(basename))
        .filter_map(|c| load(scope.root, c))
        .filter(|t| keep(t))
        .collect()
}
