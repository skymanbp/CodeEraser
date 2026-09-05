//! §7.2 member-identity anchors (7.0.0, C-nth): what tells two
//! same-key units of one file apart in the BASELINE. Until 7.0.0 it was
//! `nth`, the occurrence order by start line — deleting an EARLIER
//! sibling shifted every survivor's nth, and the ratchet read one
//! removal plus one addition for a clone nobody touched (the §7.2
//! degradation the store and the baseline both recorded). The anchor is
//! the CONTAINER CHAIN instead: the keys of the units enclosing this
//! one, outermost first, hashed — `impl A { fn add }` and `impl B { fn
//! add }` differ by their impl, and deleting one leaves the other's
//! anchor exactly where it was. Every unit is anchored the same way, a
//! top-level one included (the empty chain hashes to one constant), so
//! neither a body edit nor a deletion elsewhere in the file ever moves
//! an identity; two same-key units under the SAME chain (a redefinition
//! — a compile error in Rust, the later wins in Python) are told apart
//! by their order under it, the one shape the old degradation survives,
//! stated here — two closures on ONE line are that shape too, and the
//! scanner's k-th same-span function is the index's k-th (`of`). The
//! index's `nth` column is untouched: the join and churn ledgers still key on
//! it, and this module reads the index's unit table rather than
//! re-segmenting a file (the one-throat rule the unit caches keep).

use crate::dedup::index::Index;
use crate::dedup::tokens::fnv1a;
use anyhow::Result;
use std::collections::{BTreeMap, HashMap};

/// One unit of one file with its §7.2 anchor.
pub struct Anchored {
    pub key: String,
    pub start: i64,
    pub end: i64,
    pub anchor: String,
}

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

/// Anchor one file's units (key, start, end): the hex fnv1a of the
/// container chain, then `#n` = the unit's start-line order among the
/// same-key units under that same chain (0 for every unit but a
/// redefinition's later copies). Pure and order-free — the input may
/// arrive in any order, the anchors depend on spans and keys alone.
pub fn anchored(units: Vec<(String, i64, i64)>) -> Vec<Anchored> {
    let chains: Vec<String> = units.iter().map(|u| chain_of(&units, u)).collect();
    let mut order: Vec<usize> = (0..units.len()).collect();
    order.sort_by_key(|&i| (units[i].1, units[i].2, units[i].0.as_str()));
    let mut groups: HashMap<(&str, &str), Vec<usize>> = HashMap::new();
    for &i in &order {
        groups
            .entry((units[i].0.as_str(), chains[i].as_str()))
            .or_default()
            .push(i);
    }
    let anchor = |i: usize| -> String {
        let group = &groups[&(units[i].0.as_str(), chains[i].as_str())];
        let n = group
            .iter()
            .position(|&j| j == i)
            .expect("every unit sits in its own group");
        format!("{:016x}#{n}", fnv1a(chains[i].as_bytes()))
    };
    units
        .iter()
        .enumerate()
        .map(|(i, (key, start, end))| Anchored {
            key: key.clone(),
            start: *start,
            end: *end,
            anchor: anchor(i),
        })
        .collect()
}

/// The keys of every unit STRICTLY enclosing `u`, outermost first,
/// NUL-joined — equal spans enclose nothing (two units on one line
/// are siblings, never each other's container).
fn chain_of(units: &[(String, i64, i64)], u: &(String, i64, i64)) -> String {
    let mut outer: Vec<&(String, i64, i64)> = units
        .iter()
        .filter(|c| c.1 <= u.1 && u.2 <= c.2 && (c.1, c.2) != (u.1, u.2))
        .collect();
    outer.sort_by_key(|c| std::cmp::Reverse(c.2 - c.1));
    outer
        .iter()
        .map(|c| c.0.as_str())
        .collect::<Vec<_>>()
        .join("\0")
}

#[cfg(test)]
#[path = "../../tests/unit/score/anchor.rs"]
mod tests;
