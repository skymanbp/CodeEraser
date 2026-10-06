//! The churn report shape, split from mod.rs at the repo's own
//! 300-line dogfood gate when the M5-3h per-unit ledger landed, and
//! the request the core lays its document out from (plan v2.32 step 5,
//! the R0 pilot): totals are METHODS over the ledger, never stored
//! fields — the conservation-by-construction half of the ledger design
//! — and the document, its console lines and the pairing cap the
//! measurement reads (`document.churn.cochangeFileCap`) are the core's
//! (CE.Churn.Document, CE.Churn.Lines).

use crate::document::{self, Answer, Paths, Request};
use anyhow::Result;

/// One ledger row: lines the window added inside this unit. `key` ""
/// (with anchor "") is the file's top level — `owner()` found no
/// containing unit, which is a real place, not an error; `anchor` is
/// the unit's §7.2 container-chain anchor (fourclass/anchor.rs), the
/// identity a HEAD-side join reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitRow {
    pub path: String,
    pub key: String,
    pub anchor: String,
    pub appended: usize,
    pub rewrote: usize,
}

pub struct Report {
    pub commits: usize,
    /// Per-unit ledger, sorted by (path, key, anchor).
    pub units: Vec<UnitRow>,
    pub surviving: usize,
    pub cochange: Vec<(String, String, usize)>,
    pub skipped_large: usize,
    /// Declared submodules holding judged files whose history is not
    /// this repository's (mod.rs `unhistoried`) — the ledger's named
    /// shortfall, never an unnamed exclusion.
    pub submodules_without_history: Vec<String>,
}

impl Report {
    /// Every added line lands in exactly one ledger row, so the
    /// window totals are sums over it, never separate counters.
    pub fn append_lines(&self) -> usize {
        self.units.iter().map(|u| u.appended).sum()
    }
    pub fn rewrite_lines(&self) -> usize {
        self.units.iter().map(|u| u.rewrote).sum()
    }
    pub fn added_in_window(&self) -> usize {
        self.append_lines() + self.rewrite_lines()
    }
}

/// The churn document and its lines, laid out by the core at `core`:
/// the window's sums, the survivors and the skip count as facts, each
/// co-change pair `[a, b, commits]` over the paths it names, and the
/// paths and submodules the core spells in.
pub fn answer(core: &str, r: &Report, days: u32) -> Result<Answer> {
    let mut paths = Paths::default();
    let cochange: Vec<[i64; 3]> = r
        .cochange
        .iter()
        .map(|(a, b, n)| [paths.id(a), paths.id(b), *n as i64])
        .collect();
    let req = Request::new("churn")
        .range("paths", paths.list.len())
        .range("submodules", r.submodules_without_history.len())
        .fact("days", days)
        .fact("commits", r.commits)
        .fact("appended", r.append_lines())
        .fact("rewrote", r.rewrite_lines())
        .fact("surviving", r.surviving)
        .fact("skipped", r.skipped_large)
        .rows("cochange", cochange);
    let req = req
        .text("path", &paths.list)
        .text("submodule", &r.submodules_without_history);
    document::assemble(core, req)
}
