//! trend/1 wire leg (M7.5b): hand the measured trajectory to the
//! core and relay its slope verdict — the eighth judgment family.
//! Rust computes no policy here: sign, floor and the fail bit all
//! come back on the wire (ADR-008), and no verdict mirror is built
//! (the structure-family stance). Knob rows ride only when ce.toml
//! declares them (the ceilings/27b9bc2 pattern).

use super::Row;
use crate::corelink::Link;
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::path::Path;

/// The core's judgment over the window, relayed verbatim: absent
/// slope/verdict = below minPoints (unjudged, not flat); fail is
/// armed only by a declared decline floor.
#[derive(Debug)]
pub struct Judgment {
    pub slope_micro_per_day: Option<i64>,
    pub verdict: Option<i64>,
    pub fail: bool,
    pub knobs: Vec<[i64; 2]>,
}

/// One trend.request over a fresh core link (the deadcode::judge
/// shape); a missing capability or a non-result reply is an error,
/// never a silently unjudged report.
pub fn judge(root: &Path, core: &str, rows: &[Row]) -> Result<Judgment> {
    let cfg = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
    let mut knobs: Vec<[i64; 2]> = Vec::new();
    if let Some(mp) = cfg.trend.min_points {
        knobs.push([0, mp as i64]);
    }
    if let Some(floor) = cfg.trend.decline_floor_micro {
        knobs.push([1, floor as i64]);
    }
    let (mut link, _hello) = Link::open(core).map_err(anyhow::Error::msg)?;
    if !link.has("trend/1") {
        bail!("ce-core offers no trend/1 capability — upgrade the core");
    }
    let mut body = json!({
        "rows": rows.iter().map(|r| json!([r.ts, r.score, r.scale])).collect::<Vec<_>>(),
    });
    if !knobs.is_empty() {
        body["knobs"] = json!(knobs);
    }
    let reply = link.request("trend", body).map_err(anyhow::Error::msg)?;
    if reply["type"] != json!("trend.result") {
        bail!("core replied {}: {reply}", reply["type"]);
    }
    Ok(Judgment {
        slope_micro_per_day: reply["slopeMicroPerDay"].as_i64(),
        verdict: reply["verdict"].as_i64(),
        fail: reply["fail"] == json!(true),
        knobs: parse_knobs(&reply["knobs"])?,
    })
}

/// The effective-knob echo, refused on drift from the request's own
/// declared rows (the P4 round-trip pin, relay form).
fn parse_knobs(v: &Value) -> Result<Vec<[i64; 2]>> {
    let rows: Vec<[i64; 2]> = serde_json::from_value(v.clone())?;
    anyhow::ensure!(
        rows.len() == 2,
        "trend knob echo carries {} rows",
        rows.len()
    );
    Ok(rows)
}

pub fn judgment_json(j: &Judgment) -> Value {
    json!({
        "slopeMicroPerDay": j.slope_micro_per_day,
        "verdict": j.verdict,
        "fail": j.fail,
        "knobs": j.knobs,
    })
}

/// Console words for the verdict code — rendering only, the code is
/// the core's.
pub fn verdict_str(j: &Judgment) -> &'static str {
    match j.verdict {
        Some(0) => "improving",
        Some(1) => "flat",
        Some(2) => "degrading",
        _ => "unjudged (below minPoints)",
    }
}
