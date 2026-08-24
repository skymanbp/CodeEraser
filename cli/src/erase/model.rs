//! The erase plan's data model — types only, importing nothing from
//! its siblings, so every other erase file can reference it DOWNWARD
//! (the house convention: children never `super::`, shared shapes
//! live in a leaf — an intra-module `use super::` web is an import
//! cycle by the graph family's own axis-6 measure, and this module
//! got charged for its first draft).

use serde::Serialize;
use std::collections::BTreeMap;

/// JSON output schema id; bump on shape change (plan §7.1).
pub const SCHEMA_ID: &str = "ce.erase-plan/0.1.0";

/// Frozen class positions — the wire's `class` field (erase/1).
/// Class 3 is dead_file on the CONFIDENCE road (2.32.0, H3): same
/// candidate family, its trust fact now the graph family's own
/// per-row confidence; class 0 is superseded, judged unchanged for
/// the grace window.
pub const CLASS_NAMES: [&str; 4] = ["dead_file", "verbatim_doc", "t1_twin", "dead_file"];

/// Frozen reason positions — the wire's `reason` field (erase/1).
pub const REASON_NAMES: [&str; 6] = [
    "eraseable",
    "language_unresolved",
    "not_full_segment",
    "bytes_differ",
    "copy_not_dead",
    "unit_not_covered",
];

/// One candidate as measured: the dense facts the wire carries plus
/// the labels the wire deliberately does not (row index is identity;
/// the CLI re-labels the verdicts on return).
pub struct Candidate {
    pub class: usize,
    pub facts: [i64; 4],
    pub path: String,
    /// 1-based inclusive line span; None = the whole file.
    pub span: Option<(i64, i64)>,
    pub provenance: String,
}

/// One judged plan row.
#[derive(Serialize, Clone)]
pub struct Row {
    pub class: &'static str,
    pub eraseable: bool,
    pub reason: &'static str,
    pub path: String,
    pub span: Option<(i64, i64)>,
    pub provenance: String,
    /// fnv1a64 of the target file's bytes at plan time — apply
    /// refuses a file that moved since planning (plans are not
    /// portable across edits).
    pub hash: u64,
}

#[derive(Serialize)]
pub struct Counts {
    pub candidates: usize,
    pub eraseable: usize,
    pub advisory: usize,
    /// Findings OUTSIDE the three deterministic-safe classes, named
    /// in aggregate: the plan hands the reader their family commands
    /// instead of pretending they have a safe erase.
    pub out_of_class: BTreeMap<&'static str, usize>,
}

#[derive(Serialize)]
pub struct Plan {
    pub rows: Vec<Row>,
    pub counts: Counts,
}

impl Plan {
    /// The rows apply will act on, in plan order.
    pub fn eraseable(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(|r| r.eraseable)
    }
}
