//! The provenance table (plan v2.18 step #14, O40; wire 6.4.0): the
//! file entities that exist under the scope but own no continuous
//! row this run — the core reads it beside the committed baseline
//! and answers `ratchet.dropped`, the rows an EXCLUSION explains
//! (a `.ceignore` line, an `exclude` glob) as opposed to a deletion.
//! A row that vanished with its file is `removed` and the violation
//! set shrank; a row that vanished because its file was hidden from
//! the walk is `rows_dropped`, a named fail condition, because that
//! is the one edit that could retire a ceiling in silence.
//!
//! Two universes on purpose: the MEASURED set is the scan's (every
//! ignore file, every exclude, the owner rule), and this table is
//! walked with no ignore file and no exclude — those are the roads
//! being watched — but with the built-in excludes, the secrets table,
//! the hidden rule and the owner pruning, because a file those hide
//! was never a candidate (`scan::walk::collect_unignored`).

use super::baseline::file_entity;
use crate::scan::walk::{self, Walked};
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

/// The entity key of a scope-relative path: the PROJECT-relative
/// one. `ce check cli` measures a scope whose rows the root-only
/// baseline (O30) records under `cli/…`; the prefix is the scope's
/// own path under the project, empty when the two coincide — so a
/// whole-project run spells every key exactly as it always did.
pub(crate) struct Keys {
    prefix: String,
}

impl Keys {
    pub(crate) fn of(root: &Path) -> Self {
        let project = crate::root::project_root(root);
        let here = std::path::absolute(root).unwrap_or_else(|_| root.to_path_buf());
        Keys {
            prefix: walk::rel_str(&project, &here),
        }
    }

    pub(crate) fn key(&self, scope_rel: &str) -> String {
        if self.prefix.is_empty() {
            scope_rel.to_string()
        } else {
            format!("{}/{scope_rel}", self.prefix)
        }
    }
}

/// The ascending, deduplicated file entities present under `root`
/// and absent from `measured` (the file rows this run produced).
/// Candidate = the walker's own file (never a foreign reader's) in a
/// scan language; the language rule is the same `Lang::from_path`
/// the measured walk applies, so the two sets differ ONLY by the
/// ignore roads.
pub(crate) fn present(root: &Path, measured: &BTreeSet<u64>) -> Result<Vec<u64>> {
    let keys = Keys::of(root);
    let walked = walk::collect_unignored(root).map_err(anyhow::Error::msg)?;
    let mut out: Vec<u64> = walked
        .iter()
        .filter(|w| candidate(w))
        .map(|w| file_entity(&keys.key(&walk::rel_str(root, &w.path))))
        .filter(|u| !measured.contains(u))
        .collect();
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

fn candidate(w: &Walked) -> bool {
    !w.foreign && crate::scan::lang::Lang::from_path(&w.path).is_some()
}
