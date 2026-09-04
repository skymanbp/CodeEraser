//! What ce.toml adds to the measurement (plan v2.27 step 3): the
//! `[tombstone]` table's declared ledgers and vocabulary, compiled
//! once per run and handed to `measure`. The hooks and the audits
//! build one from the loaded config; the replay and a tree without
//! the table measure with `Policy::default()` — one instrument, the
//! table only naming what a repository knows about itself.

use crate::config::Config;
use crate::scan::globs::{self, Inclusions};
use std::path::Path;

#[derive(Default)]
pub struct Policy {
    /// `[tombstone] ledger`, compiled; None = nothing declared.
    ledger: Option<Inclusions>,
    /// `[tombstone] terms`, lower-cased the way spellings are keyed.
    terms: Vec<String>,
}

impl Policy {
    /// The policy a loaded config declares. Its globs passed the load
    /// throat (`Config::globs_fault`), so a set that fails to compile
    /// here is unreachable and read as nothing declared — a hook is
    /// no place for a panic.
    pub fn of(root: &Path, cfg: &Config) -> Policy {
        let t = &cfg.tombstone;
        let ledger = (!t.ledger.is_empty())
            .then(|| globs::compile_inclusions(root, &t.ledger, "[tombstone] ledger").ok())
            .flatten();
        Policy {
            ledger,
            terms: t.terms.iter().map(|w| w.to_lowercase()).collect(),
        }
    }

    /// Whether `rel` was declared to hold the changelog role.
    pub fn declared(&self, rel: &str) -> bool {
        self.ledger
            .as_ref()
            .is_some_and(|set| globs::selected(set, rel))
    }

    /// Whether `word` — one word of a spelling, as `names` keys it —
    /// is the repository's own vocabulary.
    pub fn term(&self, word: &str) -> bool {
        self.terms.iter().any(|t| t == word)
    }
}
