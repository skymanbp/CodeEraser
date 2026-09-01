//! Metric data model + shared AST walking helpers.

pub mod cognitive;
pub mod cyclo;
pub mod naming;
pub mod size;
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
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub max_nesting: u32,
    /// Name conforms to the language's convention (readability §4.1)
    /// — the pinned mirror's verdict; the core judges the same from
    /// the facts below, and the whole-report ensure holds them equal.
    pub name_ok: bool,
    /// The five naming facts bound for the wire ([lang, style,
    /// upper, under, test] — naming::facts). Skipped: wire shape,
    /// not report vocabulary (schema §7.1 unchanged).
    #[serde(skip)]
    pub naming: [i64; 5],
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
