//! erase/1 codec (proto 2.16.0): dense fact rows out, per-row
//! [eraseable, reason] verdicts back in request order. Row index is
//! identity — paths never cross the wire (§5.9.2) — and the reply
//! table is length-locked to the request: a short table would let an
//! unjudged candidate default, and an unjudged candidate must never
//! erase. The family is knobless (safety is not tunable), so there
//! is no echo to pin — the capability plus the length lock are the
//! whole handshake.

use crate::erase::model::Candidate;
use anyhow::{Context, Result, ensure};
use serde_json::json;

/// One erase.request over the open core link; degraded refused (the
/// candidate set is bounded by real findings — an over-cap answer
/// means the plan is beyond anything this side should trust).
pub fn judge(core: &str, cands: &[Candidate]) -> Result<Vec<(bool, i64)>> {
    if cands.is_empty() {
        return Ok(Vec::new());
    }
    let mut link = crate::lockstep::open_family(core, "erase/1")?;
    let rows: Vec<[i64; 5]> = cands
        .iter()
        .map(|c| {
            let [w, x, y, z] = c.facts;
            [c.class as i64, w, x, y, z]
        })
        .collect();
    let reply = link
        .request("erase", json!({ "rows": rows }))
        .map_err(anyhow::Error::msg)?;
    crate::lockstep::refuse_degraded(&reply, "erase/wire.rs vs CE.Erase.Cost")?;
    let verdicts: Vec<[i64; 2]> = crate::lockstep::reply_rows(&reply, "rows")?;
    ensure!(
        verdicts.len() == cands.len(),
        "core judged {} of {} erase rows",
        verdicts.len(),
        cands.len()
    );
    verdicts
        .into_iter()
        .map(|[bit, reason]| {
            let _ = crate::erase::model::REASON_NAMES
                .get(reason as usize)
                .context("erase reason out of range — wire-version skew")?;
            Ok((bit == 1, reason))
        })
        .collect()
}
