//! The check document (plan v2.32 step 4; design booklet
//! docs/reference/authority-track.md §5): the core lays it out
//! (document/1, CE.Score.Document) from the verdict reply's rows sent
//! back, the floor this run was armed with, this side's counts, and
//! the held conditions and the degraded reason by their codes in the
//! package's lists. `ce check`, `ce baseline`, the MCP tool and the
//! GUI print it; the console lines and the veto are the core's too
//! (CE.Score.Lines). No repository string rides it, so nothing is
//! bound.

use crate::document::{self, Answer, Request};
use crate::score::model::Outcome;
use anyhow::{Context, Result};

/// The tables a check request carries.
const TABLES: [&str; 12] = [
    "scale",
    "floor",
    "reason",
    "axes",
    "candidates",
    "joinSeverity",
    "added",
    "removed",
    "over",
    "toleranceDrawn",
    "failed",
    "dropped",
];

/// The check document over one outcome, laid out by the core at
/// `core`, with its lines; `roast` is the console's `--roast` (one
/// more line, the document unchanged).
pub fn document(core: &str, o: &mut Outcome, roast: bool) -> Result<Answer> {
    let held = std::mem::replace(&mut o.held, Err(String::new()));
    let req = request(o)?.fact("roast", u8::from(roast));
    document::assemble_over(core, held, req)
}

/// The verdict reply's rows, this run's floor and counts, and the two
/// name lists' codes (the package's `document.check`).
fn request(o: &Outcome) -> Result<Request> {
    let r = &o.reply;
    let cat = &crate::tables::get().document.check;
    let code = |list: &[&str], name: &str, what: &str| {
        list.iter().position(|n| *n == name).with_context(|| {
            format!("verdict reply names a {what} the package does not list: {name}")
        })
    };
    let failed = r
        .failed
        .iter()
        .map(|n| code(cat.failed, n, "fail condition").map(|c| [c]))
        .collect::<Result<Vec<_>>>()?;
    let column = |xs: &[u64]| xs.iter().map(|x| [*x]).collect::<Vec<_>>();
    let mut req = Request::new("check")
        .range("files", o.files)
        .range("why", 0)
        .fact("score", r.score)
        .fact("fail", i64::from(r.fail))
        .fact("droppedRode", i64::from(r.dropped.is_some()))
        .fact("simPairs", o.sim_pairs)
        .fact("members", o.members)
        .fact("collapsed", o.collapsed)
        .fact("skippedSelf", o.skipped_self)
        .rows("axes", &r.axes)
        .rows("candidates", &r.candidates)
        .rows("joinSeverity", &r.join_severity)
        .rows("added", column(&r.added))
        .rows("removed", column(&r.removed))
        .rows("over", &r.over)
        .rows("toleranceDrawn", &r.tolerance_drawn)
        .rows("failed", failed)
        .single("scale", r.knobs.get("scoreScale"))
        .single("floor", o.floor);
    if let Some(reason) = &r.degraded {
        req = req.rows("reason", [[code(cat.reasons, reason, "degraded reason")?]]);
    }
    if let Some(d) = &r.dropped {
        req = req.rows("dropped", d);
    }
    Ok(req.empty(&TABLES))
}
