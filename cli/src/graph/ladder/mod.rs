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
//! ledger row, never a silent skip.

use crate::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::Path;

pub mod ts;

/// Which rung answered (1-based per the design §4 table); stored on
/// every edge — ammunition for the per-level cut table.
pub type Rung = u8;

/// Unresolved reasons — the frozen design §4 vocabulary. `Dynamic`
/// and `Macro` are reserved for the Python/Rust batches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Dynamic,
    AmbiguousPaths,
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

/// Dispatch one site to its language ladder. `from` is the source
/// file repo-relative with forward slashes.
pub fn resolve(lang: Lang, from: &str, spec: &str, scope: &Scope) -> Outcome {
    match lang {
        Lang::TypeScript | Lang::Tsx => ts::resolve(from, spec, scope),
        _ => Outcome::Unresolved(Reason::Unsupported),
    }
}
