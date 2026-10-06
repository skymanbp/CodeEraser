//! The measurement rows the scan wire carries. The findings, the
//! report, its console lines and its SARIF face are laid out from the
//! core's levels and document (CE.Scan.Document, scan/document.rs);
//! plan v2.33 W1 retired the pinned local mirror (`evaluate`) and the
//! second reading of the levels it was compared against.

use super::metrics::{FileMetrics, FnMetrics};

/// One measurement row bound for the wire, beside the file it was
/// measured in (the rulepack's class is read off the path) — paths
/// never cross (§5.9.2 index privacy; ADR-008 P3).
pub struct Row {
    pub code: u64,
    pub value: usize,
    pub file: String,
}

/// Metric codes a function answers for — 1..=6, the length of the
/// list below. The row arithmetic every other reader does (which row
/// is this function's cognitive one) is this number's, so it is
/// declared once, here, beside the walk that lays the rows out.
pub const FN_CODES: usize = 6;

/// Each file's row count: its own row plus its functions' — the
/// block a chunk boundary must respect, because a call arc never
/// leaves the file that minted it. Owned by rows_of's neighbour,
/// since rows_of is the row order's author.
pub fn blocks_of(files: &[FileMetrics]) -> Vec<usize> {
    files
        .iter()
        .map(|f| 1 + FN_CODES * f.functions.len())
        .collect()
}

/// Every (file or function, metric) measurement in report order — the file
/// row first, then each function's six metric rows: the order the
/// core's levels answer positionally.
pub fn rows_of(files: &[FileMetrics]) -> Vec<Row> {
    let mut out = Vec::new();
    for f in files {
        let file_row = |code, value| Row {
            code,
            value,
            file: f.path.clone(),
        };
        out.push(file_row(0, f.total_lines));
        for func in &f.functions {
            for (i, value) in fn_values(func).into_iter().enumerate() {
                out.push(file_row(i as u64 + 1, value));
            }
        }
    }
    out
}

/// A function's six metric values in code order 1..=6 (naming is
/// boolean: 1 = one non-conforming name, as the core settled it).
fn fn_values(f: &FnMetrics) -> [usize; FN_CODES] {
    [
        f.lines,
        f.params,
        f.cyclomatic as usize,
        f.cognitive as usize,
        f.max_nesting as usize,
        usize::from(!f.name_ok),
    ]
}
