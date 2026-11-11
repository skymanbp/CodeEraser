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

use crate::document::Held;
use crate::erase::model::{Candidate, REASON_NAMES, Verdict};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::ops::Range;

/// The erase.requests over one open core link, at most `cap` rows each
/// (`limits.caps.erase_rows` from the plan): the rows are split by path
/// the way scan/1 splits by file — a path's rows never straddle a
/// request, because the closure (`kept`) chooses among the rows of one
/// path and nothing across paths — and each request carries its own
/// dense path ids. The verdicts concatenate in request order; one
/// degraded reply is the plan's refusal, naming the request. The link
/// goes back for the plan's document (none when nothing was judged).
pub fn judge(core: &str, cands: &[Candidate], cap: usize) -> Result<(Vec<Verdict>, Held)> {
    if cands.is_empty() {
        return Ok((Vec::new(), Err("no candidate was judged".into())));
    }
    let parts = chunks(cands, cap)?;
    let mut link = crate::corelink::judged::open(core, "erase/1")?;
    let mut verdicts = Vec::with_capacity(cands.len());
    for (k, part) in parts.iter().enumerate() {
        let slice = &cands[part.clone()];
        let reply = link
            .request("erase", request_body(slice))
            .map_err(anyhow::Error::msg)?;
        if reply["degraded"] != Value::Bool(false) {
            let why = crate::corelink::judged::degraded(&reply)
                .err()
                .map_or("no degraded flag".into(), String::from);
            anyhow::bail!(
                "erase/1 degraded the plan at request {} of {} (rows {}..{}) ({why})",
                k + 1,
                parts.len(),
                part.start,
                part.end
            );
        }
        verdicts.extend(decode(&reply, slice.len())?);
    }
    Ok((verdicts, Ok(link)))
}

/// One request's body: the fact rows and their target table.
fn request_body(cands: &[Candidate]) -> Value {
    let rows: Vec<[i64; 5]> = cands
        .iter()
        .map(|c| {
            let [w, x, y, z] = c.facts;
            [c.class as i64, w, x, y, z]
        })
        .collect();
    json!({ "rows": rows, "targets": targets_of(cands) })
}

/// The request split: greedy runs of whole paths, each run at most
/// `cap` rows. The planner sorts the candidates by path, so a path's
/// rows are adjacent; one path with more rows than a request holds is
/// refused by name before anything is sent (it cannot be split without
/// splitting its closure), with the narrowing a user has.
pub(crate) fn chunks(cands: &[Candidate], cap: usize) -> Result<Vec<Range<usize>>> {
    crate::corelink::judged::whole_groups(cands, cap, |a, b| a.path == b.path).map_err(|(at, n)| {
        anyhow::anyhow!(
            "erase: {} alone has {n} candidates over the core's erase_rows cap {cap}; a \
             path's rows are judged in one request — narrow the plan with an `exclude` in ce.toml",
            cands[at].path
        )
    })
}

/// The target table: one [pathId, start, end] per candidate — path
/// ids dense in order of first appearance, from 0 in each request
/// (the planner sorts the candidates by path before judging, so the
/// ids ascend with the rows and the table is in the key order the
/// contract asks for),
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
/// in step; a missing `kept` is skew (corelink::skew), refused by name.
fn decode(reply: &Value, n: usize) -> Result<Vec<Verdict>> {
    let verdicts: Vec<[i64; 2]> = crate::corelink::judged::table(reply, "rows")?;
    let kept: Vec<i64> = crate::corelink::judged::table(reply, "kept")
        .with_context(|| crate::corelink::skew("erase/1 `kept`"))?;
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
