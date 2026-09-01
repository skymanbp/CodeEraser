//! The erase plan's data model — types only, importing nothing from
//! its siblings, so every other erase file can reference it DOWNWARD
//! (the house convention: children never `super::`, shared shapes
//! live in a leaf — an intra-module `use super::` web is an import
//! cycle by the graph family's own axis-6 measure, and this module
//! got charged for its first draft).

use serde::Serialize;
use std::collections::BTreeMap;

/// JSON output schema id; bump on shape change (plan §7.1). 0.2.0
/// (plan v2.25): each row carries `sites`, the unresolved reference
/// sites in its language — the number behind `language_unresolved`.
pub const SCHEMA_ID: &str = "ce.erase-plan/0.2.0";

/// Frozen class positions — the wire's `class` field (erase/1).
/// Class 3 is dead_file on the CONFIDENCE road (2.32.0, H3): its
/// trust fact is the graph family's own per-row confidence. Class 0
/// was the same candidate family on the local-count road; it was
/// superseded at 2.32.0 and RETIRED at 4.0.0 when the grace window
/// closed. Its position stays frozen and never renders — the core
/// refuses a class-0 row by name, and renumbering the survivors
/// would move three other frozen codes to save one array slot.
pub const CLASS_NAMES: [&str; 4] = ["(retired)", "verbatim_doc", "t1_twin", "dead_file"];

/// Frozen reason positions — the wire's `reason` field (erase/1).
/// Position 6 arrived at 6.1.0 with the RG10 firewall: a dead file
/// whose verdict is `unref_public` or `unreach_public` is refused by
/// name, because the four-way dead code exists precisely so an
/// exported API cannot be treated as plain dead.
pub const REASON_NAMES: [&str; 7] = [
    "eraseable",
    "language_unresolved",
    "not_full_segment",
    "bytes_differ",
    "copy_not_dead",
    "unit_not_covered",
    "public_surface",
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
    /// Unresolved reference sites in the row's language — the fact
    /// behind `language_unresolved`, carried for the reader (FIELD-TEST,
    /// plan v2.25); the wire sees it only where a class's facts do.
    pub sites: i64,
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
    /// Unresolved sites in this row's language (Candidate::sites).
    pub sites: i64,
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
