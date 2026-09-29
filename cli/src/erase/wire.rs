//! erase/1 codec (proto 2.16.0; the target table 7.2.0): dense fact
//! rows out — and, since plan v2.30 step 7b, the rows' TARGETS beside
//! them — per-row [eraseable, reason] verdicts plus the closure's
//! `kept` bit back, in request order. Row index is identity — paths
//! never cross the wire (§5.9.2): a target is [pathId, start, end]
//! with the path id dense in request order and 0/0 for the whole
//! file — and both reply tables are length-locked to the request: a
//! short table would let an unjudged candidate default, and an
//! unjudged candidate must never erase. The family is knobless
//! (safety is not tunable), so there is no knob echo to pin; `kept`
//! is the 7.2.0 fingerprint instead — a core that judged the rows
//! without the closure answers without the key and is refused by
//! name, because the plan it would leave holds two rows per target.

use crate::erase::model::{Candidate, REASON_NAMES, Verdict};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// One erase.request over the open core link; degraded refused (the
/// candidate set is bounded by real findings — an over-cap answer
/// means the plan is beyond anything this side should trust).
pub fn judge(core: &str, cands: &[Candidate]) -> Result<Vec<Verdict>> {
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
        .request(
            "erase",
            json!({ "rows": rows, "targets": targets_of(cands) }),
        )
        .map_err(anyhow::Error::msg)?;
    crate::lockstep::refuse_degraded(&reply, "erase/wire.rs vs CE.Erase.Cost")?;
    decode(&reply, cands.len())
}

/// The target table: one [pathId, start, end] per candidate — path
/// ids dense in order of first appearance (the planner sorts the
/// candidates by path before judging, so the ids ascend with the
/// rows and the table is in the key order the contract asks for),
/// 0/0 for a whole file, else the span's 1-based inclusive lines.
fn targets_of(cands: &[Candidate]) -> Vec<[i64; 3]> {
    let mut ids: BTreeMap<&str, i64> = BTreeMap::new();
    cands
        .iter()
        .map(|c| {
            let next = ids.len() as i64;
            let id = *ids.entry(c.path.as_str()).or_insert(next);
            let (start, end) = c.span.unwrap_or((0, 0));
            [id, start, end]
        })
        .collect()
}

/// The two reply tables, each length-locked to the request and read
/// in step; a missing `kept` is named as the pre-7.2.0 core it means.
fn decode(reply: &Value, n: usize) -> Result<Vec<Verdict>> {
    let verdicts: Vec<[i64; 2]> = crate::lockstep::reply_rows(reply, "rows")?;
    let kept: Vec<i64> = crate::lockstep::reply_rows(reply, "kept").context(
        "the erase reply carries no `kept` table — a core older than 7.2.0 judged the rows without the target closure",
    )?;
    ensure!(
        verdicts.len() == n && kept.len() == n,
        "core judged {} of {} erase rows ({} kept bits)",
        verdicts.len(),
        n,
        kept.len()
    );
    verdicts
        .into_iter()
        .zip(kept)
        .map(|([bit, reason], k)| {
            let _ = REASON_NAMES
                .get(reason as usize)
                .context("erase reason out of range — wire-version skew")?;
            ensure!(k == 0 || k == 1, "erase kept bit {k} is not a boolean");
            Ok(Verdict {
                eraseable: bit == 1,
                reason,
                kept: k == 1,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/erase/wire.rs"]
mod tests;
