//! The trend report as measured (split from mod.rs at the 300-line
//! dogfood gate, M8-G3b): the window, the points, what is pending, the
//! refused commits and the core's judgment. The document and the
//! console lines are the core's (document.rs, CE.Trend.Document /
//! CE.Trend.Lines, plan v2.32 step 5).

use super::judge;
use super::judge::Row;

#[derive(Debug)]
pub struct Report {
    /// Mainline commits found inside the requested window.
    pub window: usize,
    /// Measured rows, oldest first (chart order).
    pub rows: Vec<Row>,
    /// Window commits still unmeasured after this batch (includes
    /// this run's failures — they retry next run).
    pub pending: usize,
    /// (short sha, reason) for commits that refused to measure this
    /// run — reported, never silently absent.
    pub failed: Vec<(String, String)>,
    /// The core's trend/2 judgment over the window (M7.5b; robust
    /// since 2.31.0).
    pub judgment: judge::Judgment,
}
