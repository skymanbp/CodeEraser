//! Metric data model + shared AST walking helpers.

pub mod events;
pub mod naming;
pub mod size;
pub mod vocab;
pub mod walk;

pub use walk::own_nodes;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FnMetrics {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub lines: usize,
    pub params: usize,
    /// The three complexity numbers are the core's (plan v2.30 step
    /// 7b ③): `measure` leaves them 0 and `scan::settle` writes
    /// the values the core derived from `events` and judged with, so a
    /// measured-only tree reports no complexity at all rather than a
    /// second reading of it.
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub max_nesting: u32,
    /// Name conforms to the language's convention (readability §4.1)
    /// — the core's verdict over the facts below (CE.Scan.Cost
    /// .conforms), written by `scan::settle` off the code-6 level;
    /// `measure` alone leaves it true (no verdict read).
    pub name_ok: bool,
    /// The five naming facts bound for the wire ([lang, style,
    /// upper, under, test] — naming::facts). Skipped: wire shape,
    /// not report vocabulary (schema §7.1 unchanged).
    #[serde(skip)]
    pub naming: [i64; 5],
    /// The structural event stream the core folds the three numbers
    /// from (stated by metrics::events in metrics::vocab's terms).
    /// Skipped: wire shape, not report vocabulary (schema §7.1
    /// unchanged).
    #[serde(skip)]
    pub events: Vec<vocab::Event>,
}

#[derive(Debug, Serialize)]
pub struct FileMetrics {
    pub path: String,
    pub lang: &'static str,
    pub total_lines: usize,
    pub comment_lines: usize,
    pub functions: Vec<FnMetrics>,
    /// The call arcs `scan::calls` proved inside this file, as
    /// (caller, callee) indices into `functions`. Skipped: wire
    /// shape, not report vocabulary (schema §7.1 unchanged).
    #[serde(skip)]
    pub calls: Vec<(u32, u32)>,
}
