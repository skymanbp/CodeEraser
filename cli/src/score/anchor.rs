//! §7.2 member-identity anchors in the BASELINE (7.0.0, C-nth): the
//! index's cached unit tables read through the container-chain anchor
//! — the pure function lives in fourclass/anchor.rs with the reasons
//! it replaced `nth`. This module reads the graph `symbols` rows
//! rather than re-segmenting a file (the one-throat rule the unit
//! caches keep), so a baseline member's anchor and the same unit's
//! anchor in the churn ledger, the join and the seam pricer (plan
//! v2.30 step 5b item 29) come from one function over the same (key,
//! start, end) rows. The index's `nth` column is untouched: it is the
//! persisted identity every face prints, not a join key.

use crate::dedup::index::Index;
use crate::fourclass::anchor::{Anchored, anchored};
use anyhow::Result;
use std::collections::{BTreeMap, HashMap};

/// The per-file unit tables of one index snapshot, anchored.
pub struct Anchors {
    by_file: HashMap<String, Vec<Anchored>>,
}

/// One side of a baseline member: (path, key, anchor).
pub type Side = (String, String, String);

/// Cached unit spans per path: (key, start_line, end_line).
pub type UnitSpans = BTreeMap<String, Vec<(String, i64, i64)>>;

/// Every cached unit of every file, grouped by path — the one table
/// both the erase coverage predicate and the §7.2 anchors read (graph
/// symbols are the ONE persisted unit identity: join on the cache,
/// never a re-segmentation).
pub fn units_by_path(idx: &Index) -> Result<UnitSpans> {
    let mut map: UnitSpans = BTreeMap::new();
    for s in crate::graph::symbols::symbol_rows(idx)? {
        map.entry(s.path)
            .or_default()
            .push((s.key, s.start_line, s.end_line));
    }
    Ok(map)
}

impl Anchors {
    /// Every cached unit of every file anchored, off the graph
    /// `symbols` rows the same snapshot refreshed.
    pub fn from_index(idx: &Index) -> Result<Self> {
        Ok(Self {
            by_file: units_by_path(idx)?
                .into_iter()
                .map(|(p, u)| (p, anchored(u)))
                .collect(),
        })
    }

    /// The innermost unit containing the WHOLE `start..=end` span of
    /// `path`, as a member side; a span no single unit contains is the
    /// file's top level (key "", anchor "") — a cross-unit block is not
    /// guessed into either side (the join's UnitMap stance).
    pub fn owner(&self, path: &str, start: usize, end: usize) -> Side {
        let hit = self
            .by_file
            .get(path)
            .into_iter()
            .flatten()
            .filter(|u| u.start <= start as i64 && end as i64 <= u.end)
            .min_by_key(|u| u.end - u.start);
        match hit {
            Some(u) => (path.to_string(), u.key.clone(), u.anchor.clone()),
            None => (path.to_string(), String::new(), String::new()),
        }
    }

    /// The anchor of the `nth` unit standing exactly at (key, start,
    /// end) on `path` — same-span same-key units (closures sharing a
    /// line) are told apart by source order on both sides; None = the
    /// index knows no such unit.
    pub fn of(&self, path: &str, key: &str, start: usize, end: usize, nth: usize) -> Option<&str> {
        self.by_file
            .get(path)?
            .iter()
            .filter(|u| u.key == key && u.start == start as i64 && u.end == end as i64)
            .nth(nth)
            .map(|u| u.anchor.as_str())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/score/anchor.rs"]
mod tests;
