//! The flow document read back (plan v2.32 step 3): the core lays
//! the document out (document/1, CE.Flow.Document), face.rs binds it,
//! and the console and the exit code read these structs off the bound
//! value — the same value the machine faces print.

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FindingFace {
    pub path: String,
    pub unit: String,
    pub kind: String,
    pub line: u32,
    #[serde(rename = "lineEnd")]
    pub line_end: u32,
    pub var: Option<String>,
    pub judged: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RefusedFace {
    pub path: String,
    pub unit: String,
    pub reason: String,
}

/// The bound document, read.
#[derive(Deserialize)]
pub struct Report {
    pub counts: BTreeMap<String, u64>,
    pub findings: Vec<FindingFace>,
    pub refused: Vec<RefusedFace>,
    pub degraded: Option<String>,
}

impl Report {
    /// The findings the gate reads: judged ones, whatever `--kind`
    /// showed (the filter shapes the listing, never the verdict).
    pub fn judged(&self) -> u64 {
        self.counts.get("judged").copied().unwrap_or(0)
    }
}
