//! Resolution ladder (design brief §4): a site walks rungs in order
//! and the FIRST rung producing exactly one in-scope candidate
//! resolves it; more than one candidate at a rung is
//! Unresolved(ambiguous_*) — picking a "best" one would invent a
//! path. External (stdlib / registry / dependency) is a correct
//! terminal answer, not a miss. Every resolved edge records its
//! rung, so precision is attributable per level and a dirty rung can
//! be voted out by data at 2h.
//!
//! Per-language rungs land one batch at a time (TS first); languages
//! without a ladder yet return Unresolved(Unsupported) — an honest
//! ledger row, never a silent skip. Dispatch carries the site's
//! frozen kind label (store::KINDS): the TS/Py rungs are
//! kind-uniform, but Rust's mod_decl and use walk different rungs,
//! and the Markdown kinds will differ more.

use crate::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::Path;

pub mod go;
pub mod py;
pub mod rs;
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
    /// Exactly one in-scope PACKAGE directory (Go): the node identity
    /// is (pkg_dir, "") and granularity is package — collapsing to a
    /// single file would be a guess (design §4 row 4).
    ResolvedPackage {
        dir: String,
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
/// raw fs access via `root` may only justify External or block a
/// rewrite — never mint an in-scope candidate.
pub struct Scope<'a> {
    pub files: &'a BTreeSet<String>,
    pub configs: &'a [String],
    pub root: &'a Path,
}

/// Dispatch one site to its language ladder. `kind` is the site's
/// frozen label; `from` is the source file repo-relative with
/// forward slashes.
pub fn resolve(lang: Lang, kind: &str, from: &str, spec: &str, scope: &Scope) -> Outcome {
    match lang {
        Lang::TypeScript | Lang::Tsx => ts::resolve(from, spec, scope),
        Lang::Python => py::resolve(from, spec, scope),
        Lang::Rust => rs::resolve(kind, from, spec, scope),
        Lang::Go => go::resolve(from, spec, scope),
        Lang::Markdown => Outcome::Unresolved(Reason::Unsupported),
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
