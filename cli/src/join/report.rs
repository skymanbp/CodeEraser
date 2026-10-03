//! The join document read back (plan v2.32 step 4): the core lays it
//! out (join/document.rs, CE.Join.Document) and, since step 5, its
//! console lines (CE.Join.Lines); this reader serves the library's
//! callers that read the document typed. Every code and rank here is
//! the core's or the measurement's.

use super::Pos;
use super::churn_unit::{self, Lines};
use serde::Deserialize;
use serde_json::Value;

/// The document read typed; `doc` is the document itself.
#[derive(Debug, Deserialize)]
pub struct Report {
    pub days: u32,
    pub commits: usize,
    pub files: Vec<FileRow>,
    pub units: Vec<churn_unit::UnitRow>,
    pub degraded: Option<String>,
    #[serde(skip)]
    pub doc: Value,
}

/// One Tier F row: a similar file pair with all three legs and the
/// core's verdict on it.
#[derive(Debug, Deserialize)]
pub struct FileRow {
    pub a: String,
    pub b: String,
    pub blocks: usize,
    pub tokens: usize,
    /// T3 near-miss unit pairs between the two files (5b-9).
    pub near_miss: usize,
    pub graph_a: Option<Pos>,
    pub graph_b: Option<Pos>,
    pub churn_a: Lines,
    pub churn_b: Lines,
    /// None = the pair is outside the churn report's co-change table.
    pub cochange: Option<usize>,
    /// The core's verdict for this pair (2.33.0, H4); None for a
    /// self-pair (the wire's u < v contract cannot carry it) and when
    /// the judgment degraded.
    pub verdict: Option<String>,
    pub severity: Option<i64>,
    pub confidence: Option<i64>,
}

crate::report::bound!(Report);
