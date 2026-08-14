//! Tier U (unit-level) join assembly: attribute each clone block
//! side to its owning unit — the innermost unit containing the WHOLE
//! span; a span no single unit contains belongs to the file top
//! level (key ""), the same refusal-to-guess the t3 Forest ledger
//! practices — then join the churn ledger on (path, key, nth). The
//! graph leg at unit tier is null BY DESIGN: import granularity has
//! no unit nodes (unit indegree is constant 0, design §6.2), so any
//! number here would be fabricated; [`GRAPH_CAVEAT`] rides every
//! emitted row instead.

use crate::churn;
use crate::dedup::{self, pairs::Block};
use crate::fourclass::units;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Why the unit tier's graph leg is null — printed on every row, so
/// absence can never read as zero indegree.
pub const GRAPH_CAVEAT: &str = "graph leg is import-granularity; symbol-level indegree needs \
     R6 (independent 100-callsite audit >= 0.90, 2026-08-12-m5-2-graph-design.md) — \
     not unlocked this milestone";

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct UnitId {
    pub path: String,
    /// "" = file top level (no single unit contains the span).
    pub key: String,
    pub nth: i64,
}

/// Window churn of one entity (lines appended / rewritten).
#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq, Eq)]
pub struct Lines {
    pub appended: usize,
    pub rewrote: usize,
}

/// One Tier U row: a similar unit pair with its churn leg. The graph
/// leg is deliberately NOT a field — it is null for every unit row,
/// and the report prints [`GRAPH_CAVEAT`] in its place.
#[derive(Debug, Serialize)]
pub struct UnitRow {
    pub a: UnitId,
    pub b: UnitId,
    pub tokens: usize,
    pub churn_a: Lines,
    pub churn_b: Lines,
}

/// Lazy per-file unit table (key, nth, start, end) from the same
/// segments + with_nth throat the unitsig/symbols caches persist —
/// a second nth derivation is exactly what that throat forbids.
pub struct UnitMap {
    root: PathBuf,
    by_file: HashMap<String, Vec<(String, i64, usize, usize)>>,
}

impl UnitMap {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            by_file: HashMap::new(),
        }
    }

    fn table(&mut self, path: &str) -> &[(String, i64, usize, usize)] {
        let root = &self.root;
        self.by_file
            .entry(path.to_string())
            .or_insert_with(|| load_table(root, path))
    }

    /// Identity of the innermost unit containing the WHOLE span, or
    /// the file's top level when no single unit does (a cross-unit
    /// span is not guessed into either side).
    pub fn id_of(&mut self, path: &str, start: usize, end: usize) -> UnitId {
        let hit = self
            .table(path)
            .iter()
            .filter(|(_, _, s, e)| *s <= start && end <= *e)
            .min_by_key(|(_, _, s, e)| e - s);
        match hit {
            Some((key, nth, _, _)) => UnitId {
                path: path.to_string(),
                key: key.clone(),
                nth: *nth,
            },
            None => UnitId {
                path: path.to_string(),
                key: String::new(),
                nth: 0,
            },
        }
    }
}

fn load_table(root: &Path, path: &str) -> Vec<(String, i64, usize, usize)> {
    let Ok((text, lang)) = dedup::walked_text(root, path) else {
        return Vec::new(); // vanished since the pass: no units to own
    };
    let segs = units::segments(&text, lang);
    units::with_nth(&segs)
        .into_iter()
        .map(|(u, nth)| (u.key.clone(), nth, u.start_line, u.end_line))
        .collect()
}

/// Assemble the Tier U rows: one per clone block, both sides
/// unit-attributed, churn joined on the ledger identity. An absent
/// ledger row means the unit genuinely saw no window edits — a real
/// zero, not a fabricated leg.
pub fn rows(root: &Path, blocks: &[Block], ledger: &churn::Report) -> Vec<UnitRow> {
    let by_id: HashMap<(&str, &str, i64), Lines> = ledger
        .units
        .iter()
        .map(|u| {
            let lines = Lines {
                appended: u.appended,
                rewrote: u.rewrote,
            };
            ((u.path.as_str(), u.key.as_str(), u.nth), lines)
        })
        .collect();
    let churn_of = |u: &UnitId| {
        by_id
            .get(&(u.path.as_str(), u.key.as_str(), u.nth))
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
                a,
                b,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two same-key methods across impl blocks: the span inside the
    /// SECOND gets nth 1 (the with_nth throat, not a re-derivation),
    /// and a span crossing both impls refuses into the top level.
    #[test]
    fn spans_attribute_to_innermost_or_refuse_to_toplevel() {
        let root = crate::testutil::scratch("join-unitmap");
        std::fs::write(
            root.join("x.rs"),
            "impl A {\n    fn add(&self) {\n        let a = 1;\n    }\n}\n\
             impl B {\n    fn add(&self) {\n        let b = 2;\n    }\n}\n",
        )
        .expect("x.rs");
        let mut map = UnitMap::new(&root);
        let first = map.id_of("x.rs", 2, 4);
        assert_eq!((first.key.as_str(), first.nth), ("add/1", 0));
        let second = map.id_of("x.rs", 7, 9);
        assert_eq!((second.key.as_str(), second.nth), ("add/1", 1));
        let crossing = map.id_of("x.rs", 4, 7);
        assert_eq!(crossing.key, "", "cross-unit span is not guessed");
        std::fs::remove_dir_all(&root).ok();
    }
}
