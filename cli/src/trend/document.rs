//! The trend report as the core lays it out (plan v2.32 step 5;
//! CE.Trend.Document, CE.Trend.Lines): this side sends every measured
//! point (stamp, score, scale, axes), the window and pending counts,
//! and the trend/2 reply's six keys as integers; the commits and the
//! refused commits' reasons are references. The document, the console
//! lines and the veto (the judgment's fail bit, or a refused commit)
//! come back.

use super::report::Report;
use crate::document::{self, Answer, Held, Request};
use anyhow::Result;

/// The trend document, its lines and its veto, over a fresh link.
pub fn answer(core: &str, r: &Report) -> Result<Answer> {
    laid_out(core, r, Err(String::new()))
}

/// The same over `held` when it is a whole link.
pub(super) fn laid_out(core: &str, r: &Report, held: Held) -> Result<Answer> {
    let points: Vec<Vec<i64>> = (r.rows.iter().enumerate())
        .map(|(i, p)| {
            let head = [i as i64, p.ts, p.score, p.scale];
            head.into_iter()
                .chain(p.axes.iter().flatten().copied())
                .collect()
        })
        .collect();
    let j = &r.judgment;
    let req = Request::new("trend")
        .range("points", points.len())
        .range("failed", r.failed.len())
        .fact("window", r.window)
        .fact("pending", r.pending)
        .fact("fail", u8::from(j.fail))
        .rows("points", points)
        .rows("slope", j.slope_micro_per_day.map(|v| [v]).as_slice())
        .rows("verdict", j.verdict.map(|v| [v]).as_slice())
        .rows("cliff", j.cliff.as_slice())
        .rows("declineRun", j.decline_run.as_slice())
        .rows("knobs", &j.knobs);
    // a `short` commit is the core's: the commit's first twelve
    let column =
        |f: fn(&(String, String)) -> &String| -> Vec<&String> { r.failed.iter().map(f).collect() };
    let req = req
        .texts("commit", r.rows.iter().map(|p| &p.commit))
        .text("sha", column(|(sha, _)| sha))
        .text("reason", column(|(_, why)| why));
    document::assemble_over(core, held, req)
}
