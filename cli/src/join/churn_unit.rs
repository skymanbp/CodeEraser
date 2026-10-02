//! Tier U (unit-level) join assembly: attribute each clone block
//! side to its owning unit — the innermost unit containing the WHOLE
//! span; a span no single unit contains belongs to the file top
//! level (key ""), the same refusal-to-guess the t3 Forest ledger
//! practices — then join the churn ledger on (path, key, anchor), the
//! §7.2 container-chain anchor (fourclass/anchor.rs) that survives a
//! sibling's deletion between the commit and HEAD (plan v2.30 step 5b
//! item 29; the report's own identity stays the index's nth). The
//! graph leg at unit tier is null BY DESIGN: import granularity has
//! no unit nodes (unit indegree is constant 0, design §6.2), so any
//! number here would be fabricated; the reason rides every row of the
//! document as a code the core names (CE.Join.Document's
//! importGranularity, plan v2.15), so absence can never read as zero
//! indegree. Since plan v2.30 step 5b-9 the tier also carries the T3
//! family's pairs, seated by the identity that family already names.
//! The rows here are the measurement and, read back off the document,
//! the console's rows too.

use crate::churn;
use crate::dedup;
use crate::fourclass::units;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The report identity of a unit: the index's persisted (path, key,
/// nth), what every face prints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UnitId {
    pub path: String,
    /// "" = file top level (no single unit contains the span).
    pub key: String,
    pub nth: i64,
}

/// A span's owner: the report identity and the §7.2 anchor the churn
/// ledger is joined on. The anchor never rides the report — the
/// index's nth is the identity every face prints; the anchor is the
/// join key that survives a sibling's deletion.
pub struct Owner {
    pub id: UnitId,
    pub anchor: String,
}

/// Window churn of one entity (lines appended / rewritten).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lines {
    pub appended: usize,
    pub rewrote: usize,
}

/// One Tier U row: a similar unit pair with its churn leg. The graph
/// leg is deliberately NOT a field — it is null for every unit row,
/// and the document carries the reason's code in its place — the
/// code, not a sentence (plan v2.15).
#[derive(Debug, Serialize, Deserialize)]
pub struct UnitRow {
    pub a: UnitId,
    pub b: UnitId,
    #[serde(flatten)]
    pub sim: UnitSim,
    pub churn_a: Lines,
    pub churn_b: Lines,
}

/// The similarity a unit row carries (plan v2.30 step 5b-9): which
/// clone family found the pair, with that family's own metric —
/// `kind` on the wire, the metric fields beside it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UnitSim {
    /// A T1/T2 clone block: its token count.
    T1t2 { tokens: usize },
    /// A T3 near-miss pair: the core's tree edit distance and the two
    /// node counts.
    T3 { ted: i64, n1: i64, n2: i64 },
}

/// One HEAD unit of one file: key, nth, span and anchor, off the same
/// segments + with_nth throat the unitsig/symbols caches persist — a
/// second nth derivation is exactly what that throat forbids — and the
/// same anchor function the ledger keys with.
struct Seat {
    key: String,
    nth: i64,
    start: usize,
    end: usize,
    anchor: String,
}

/// Lazy per-file unit table.
pub struct UnitMap {
    root: PathBuf,
    by_file: HashMap<String, Vec<Seat>>,
}

impl UnitMap {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            by_file: HashMap::new(),
        }
    }

    fn table(&mut self, path: &str) -> &[Seat] {
        let root = &self.root;
        self.by_file
            .entry(path.to_string())
            .or_insert_with(|| load_table(root, path))
    }

    /// The innermost unit containing the WHOLE span, or the file's top
    /// level when no single unit does (a cross-unit span is not
    /// guessed into either side): its report identity and its anchor.
    pub fn id_of(&mut self, path: &str, start: usize, end: usize) -> Owner {
        let hit = self
            .table(path)
            .iter()
            .filter(|s| s.start <= start && end <= s.end)
            .min_by_key(|s| s.end - s.start);
        let (key, nth, anchor) = match hit {
            Some(s) => (s.key.clone(), s.nth, s.anchor.clone()),
            None => (String::new(), 0, String::new()),
        };
        let path = path.to_string();
        Owner {
            id: UnitId { path, key, nth },
            anchor,
        }
    }

    /// The unit itself, by the report identity the T3 family already
    /// carries (5b-9): its anchor for the churn join. A unit the pass
    /// saw that the table no longer holds joins no churn (anchor "").
    pub fn seat(&mut self, path: &str, key: &str, nth: i64) -> Owner {
        let anchor = self
            .table(path)
            .iter()
            .find(|s| s.key == key && s.nth == nth)
            .map_or(String::new(), |s| s.anchor.clone());
        let id = UnitId {
            path: path.to_string(),
            key: key.to_string(),
            nth,
        };
        Owner { id, anchor }
    }
}

fn load_table(root: &Path, path: &str) -> Vec<Seat> {
    let Ok((text, lang)) = dedup::walked_text(root, path) else {
        return Vec::new(); // vanished since the pass: no units to own
    };
    let segs = units::segments(&text, lang);
    let anchors = crate::fourclass::anchor::for_units(&segs);
    units::with_nth(&segs)
        .into_iter()
        .zip(anchors)
        .map(|((u, nth), anchor)| Seat {
            key: u.key.clone(),
            nth,
            start: u.start_line,
            end: u.end_line,
            anchor,
        })
        .collect()
}

/// Assemble the Tier U rows: one per clone block and one per T3 pair
/// (5b-9), both sides unit-attributed, churn joined on the ledger
/// identity (path, key, anchor). An absent ledger row means the unit
/// genuinely saw no window edits — a real zero, not a fabricated leg.
pub fn rows(root: &Path, sim: &crate::score::Similar<'_>, ledger: &churn::Report) -> Vec<UnitRow> {
    let by_id: HashMap<(&str, &str, &str), Lines> = ledger
        .units
        .iter()
        .map(|u| {
            let lines = Lines {
                appended: u.appended,
                rewrote: u.rewrote,
            };
            ((u.path.as_str(), u.key.as_str(), u.anchor.as_str()), lines)
        })
        .collect();
    let churn_of = |o: &Owner| {
        by_id
            .get(&(o.id.path.as_str(), o.id.key.as_str(), o.anchor.as_str()))
            .copied()
            .unwrap_or_default()
    };
    let mut map = UnitMap::new(root);
    let row = |a: Owner, b: Owner, sim: UnitSim| UnitRow {
        sim,
        churn_a: churn_of(&a),
        churn_b: churn_of(&b),
        a: a.id,
        b: b.id,
    };
    let mut out = Vec::new();
    for blk in sim.blocks {
        let a = map.id_of(&blk.a_file, blk.a_start, blk.a_end);
        let b = map.id_of(&blk.b_file, blk.b_start, blk.b_end);
        out.push(row(a, b, UnitSim::T1t2 { tokens: blk.tokens }));
    }
    for &(a, b, m) in &sim.t3.clones {
        let (ua, ub) = (&sim.t3.units[a], &sim.t3.units[b]);
        let a = map.seat(&ua.path, &ua.key, ua.nth);
        let b = map.seat(&ub.path, &ub.key, ub.nth);
        let sim = UnitSim::T3 {
            ted: m.ted,
            n1: m.n1,
            n2: m.n2,
        };
        out.push(row(a, b, sim));
    }
    out
}

#[cfg(test)]
#[path = "../../tests/unit/join/churn_unit.rs"]
mod tests;
