//! trend/1 wire leg (M7.5b): hand the measured trajectory to the
//! core and relay its slope verdict — the eighth judgment family.
//! Rust computes no policy here: sign, floor and the fail bit all
//! come back on the wire (ADR-008), and no verdict mirror is built
//! (the structure-family stance). Knob rows ride only when ce.toml
//! declares them (the ceilings/27b9bc2 pattern).

use super::Row;
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
    let mut link = crate::lockstep::open_family(core, "trend/1")?;
    let mut body = json!({
        "rows": rows.iter().map(|r| json!([r.ts, r.score, r.scale])).collect::<Vec<_>>(),
    });
    if !knobs.is_empty() {
        body["knobs"] = json!(knobs);
    }
    let reply = link.request("trend", body).map_err(anyhow::Error::msg)?;
    Ok(Judgment {
        slope_micro_per_day: reply["slopeMicroPerDay"].as_i64(),
        verdict: reply["verdict"].as_i64(),
        fail: reply["fail"] == json!(true),
        knobs: parse_knobs(&reply["knobs"], &knobs)?,
    })
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

pub fn judgment_json(j: &Judgment) -> Value {
    json!({
        "slopeMicroPerDay": j.slope_micro_per_day,
        "verdict": j.verdict,
        "fail": j.fail,
        "knobs": j.knobs,
    })
}

/// Console words for the verdict code — rendering only, the code is
/// the core's; bilingual via the i18n switch (M8-G3b).
pub fn verdict_str(j: &Judgment) -> &'static str {
    use crate::i18n::t;
    match j.verdict {
        Some(0) => t("improving", "上行"),
        Some(1) => t("flat", "持平"),
        Some(2) => t("degrading", "恶化"),
        _ => t("unjudged (below minPoints)", "未判（低于最小点数）"),
    }
}
