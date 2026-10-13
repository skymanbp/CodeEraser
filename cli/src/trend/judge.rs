//! trend/2 wire leg (M7.5b; the robust estimator since 2.31.0):
//! hand the measured trajectory to the core and relay its judgment
//! — the Theil-Sen slope verdict plus the two shape facts, cliff
//! and decline run, each naming a commit by REQUEST INDEX (the row
//! order this side sent; hashes never cross, §5.9.2). Rust computes
//! no policy here: sign, floor and the fail bit all come back on
//! the wire (ADR-008), and no verdict mirror is built (the
//! structure-family stance). Knob rows ride only when ce.toml
//! declares them (the ceilings/27b9bc2 pattern).

use crate::corelink::Link;
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;

/// The core's judgment over the window, relayed verbatim: absent
/// slope/verdict = below minPoints (unjudged, not flat); fail is
/// armed only by a declared decline floor.
#[derive(Debug)]
pub struct Judgment {
    pub slope_micro_per_day: Option<i64>,
    pub verdict: Option<i64>,
    /// [request index of the LATER point, drop in micro units] —
    /// the steepest single-step fall of the judged window (trend/2).
    pub cliff: Option<[i64; 2]>,
    /// [request index of the run's FIRST point, points in the run]
    /// — the longest strictly-falling run (trend/2).
    pub decline_run: Option<[i64; 2]>,
    pub fail: bool,
    pub knobs: Vec<[i64; 2]>,
}

/// One measured trajectory point — built by mod.rs, judged over this
/// wire leg, laid out by the core (document.rs).
#[derive(Debug, PartialEq, Eq)]
pub struct Row {
    pub commit: String,
    pub ts: i64,
    pub score: i64,
    pub scale: i64,
    pub axes: Vec<[i64; 2]>,
}

/// One trend.request over a fresh core link (the deadcode::judge
/// shape); a missing capability or a non-result reply is an error,
/// never a silently unjudged report. The link goes back for the
/// report's document.
pub fn judge(root: &Path, core: &str, rows: &[Row]) -> Result<(Judgment, Link)> {
    let knobs = declared(root)?;
    let mut link = crate::corelink::judged::open(core, "trend/2")?;
    let mut body = json!({
        "rows": rows.iter().map(|r| json!([r.ts, r.score, r.scale])).collect::<Vec<_>>(),
    });
    if !knobs.is_empty() {
        body["knobs"] = json!(knobs);
    }
    let reply = link.request("trend", body).map_err(anyhow::Error::msg)?;
    // the degraded posture first: an over-cap reply carries null slope
    // and verdict, `fail` true and the DEFAULT knobs, which read as an
    // unjudged failing window or a false knob pin (step 11b item 5)
    if let Err(why) = crate::corelink::judged::degraded(&reply) {
        anyhow::bail!("trend/2 degraded the judgment ({why})");
    }
    let j = Judgment {
        slope_micro_per_day: reply["slopeMicroPerDay"].as_i64(),
        verdict: reply["verdict"].as_i64(),
        cliff: opt_pair(&reply["cliff"])?,
        decline_run: opt_pair(&reply["declineRun"])?,
        fail: reply["fail"] == json!(true),
        knobs: parse_knobs(&reply["knobs"], &knobs)?,
    };
    // the shape facts point INTO the request this side just sent —
    // an index past it is core drift, refused before anything
    // renders a commit name from it
    for (name, pair) in [("cliff", j.cliff), ("declineRun", j.decline_run)] {
        if let Some([idx, _]) = pair {
            anyhow::ensure!(
                idx >= 0 && (idx as usize) < rows.len(),
                "trend {name} points at row {idx} of {} — index drift",
                rows.len()
            );
        }
    }
    Ok((j, link))
}

/// The knob rows ce.toml declares (`[trend]`), in code order.
fn declared(root: &Path) -> Result<Vec<[i64; 2]>> {
    let cfg = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
    let mut knobs: Vec<[i64; 2]> = Vec::new();
    if let Some(mp) = cfg.trend.min_points {
        knobs.push([0, mp as i64]);
    }
    if let Some(floor) = cfg.trend.decline_floor_micro {
        knobs.push([1, floor as i64]);
    }
    Ok(knobs)
}

/// A window of `commits` against the core's row ceiling
/// (`limits.caps.trend_rows`, rows and declared knob rows together),
/// before a commit of it is measured: over it is refused by name with
/// the largest window that fits (plan v2.33 step 11b item 5) — measuring
/// commits the core would not judge is the slowest thing this binary
/// could waste.
pub fn window_fits(root: &Path, commits: usize) -> Result<()> {
    let knobs = declared(root)?.len();
    window_within(commits, knobs, crate::tables::get().limits.caps.trend_rows)
}

/// `window_fits` over the three numbers.
pub(crate) fn window_within(commits: usize, knobs: usize, cap: usize) -> Result<()> {
    let room = cap.saturating_sub(knobs);
    anyhow::ensure!(
        commits <= room,
        "trend: --commits {commits} over the core's trend_rows cap {cap} ({knobs} declared knob \
         row(s) ride with the rows); ask for at most {room}"
    );
    Ok(())
}

/// Null -> None, [i, v] -> Some — anything else is a wire error.
fn opt_pair(v: &Value) -> Result<Option<[i64; 2]>> {
    Ok(serde_json::from_value(v.clone())?)
}

/// The effective-knob echo, refused on drift from the request's own
/// declared rows (the P4 round-trip pin, relay form).
///
/// It counted rows only, so an echo of two WRONG values passed and
/// was reported effective — what a round-trip pin exists to catch.
fn parse_knobs(v: &Value, sent: &[[i64; 2]]) -> Result<Vec<[i64; 2]>> {
    let rows: Vec<[i64; 2]> = serde_json::from_value(v.clone())?;
    anyhow::ensure!(
        rows.len() == 2,
        "trend knob echo carries {} rows",
        rows.len()
    );
    for [code, want] in sent {
        let got = rows.iter().find(|[c, _]| c == code).map(|[_, v]| *v);
        anyhow::ensure!(
            got == Some(*want),
            "trend knob {code}: sent {want}, echo says {got:?} — one number, two owners"
        );
    }
    Ok(rows)
}

#[cfg(test)]
#[path = "../../tests/unit/trend/judge.rs"]
mod tests;
