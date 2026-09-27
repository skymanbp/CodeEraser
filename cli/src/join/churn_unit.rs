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
//! number here would be fabricated; [`GRAPH_NULL_IMPORT_GRANULARITY`]
//! rides every emitted row instead, so absence can never read as zero
//! indegree.

use crate::churn;
use crate::dedup::{self, pairs::Block};
use crate::fourclass::units;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Why the unit tier's graph leg is null, as a CODE rather than a
/// sentence (plan v2.15). It used to be 200 characters of English
/// prose riding every emitted row of the report JSON — which i18n.rs
/// declares the machine face and never translates, so no lookup
/// switch could reach it and a zh reader got English. The console
/// meanwhile rendered the SAME fact from its own bilingual template
/// ("graph null (R6 locked)"): one fact, two sources, one of them
/// untranslatable. Measurement emits the code; each face owns the
/// words, exactly as erase's reason codes 0..6 already work.
///
/// Frozen position, like every other verdict code here:
///   1 import_granularity — import granularity has no unit nodes, so
///     symbol-level indegree needs R6 (independent 100-callsite audit
///     >= 0.90, 2026-08-12-m5-2-graph-design.md), not unlocked this
///     milestone. There is no 0: a row without a reason would be the
///     fabricated number this whole design refuses.
pub const GRAPH_NULL_IMPORT_GRANULARITY: i64 = 1;

/// The report identity of a unit: the index's persisted (path, key,
/// nth), what every face prints.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq, Eq)]
pub struct Lines {
    pub appended: usize,
    pub rewrote: usize,
}

/// One Tier U row: a similar unit pair with its churn leg. The graph
/// leg is deliberately NOT a field — it is null for every unit row,
/// and the report prints [`GRAPH_NULL_IMPORT_GRANULARITY`] in its
/// place — the code, not a sentence (plan v2.15).
#[derive(Debug, Serialize)]
pub struct UnitRow {
    pub a: UnitId,
    pub b: UnitId,
    pub tokens: usize,
    pub churn_a: Lines,
    pub churn_b: Lines,
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

/// Assemble the Tier U rows: one per clone block, both sides
/// unit-attributed, churn joined on the ledger identity (path, key,
/// anchor). An absent ledger row means the unit genuinely saw no
/// window edits — a real zero, not a fabricated leg.
pub fn rows(root: &Path, blocks: &[Block], ledger: &churn::Report) -> Vec<UnitRow> {
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
    blocks
        .iter()
        .map(|blk| {
            let a = map.id_of(&blk.a_file, blk.a_start, blk.a_end);
            let b = map.id_of(&blk.b_file, blk.b_start, blk.b_end);
            UnitRow {
                tokens: blk.tokens,
                churn_a: churn_of(&a),
                churn_b: churn_of(&b),
                a: a.id,
                b: b.id,
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/join/churn_unit.rs"]
mod tests;
