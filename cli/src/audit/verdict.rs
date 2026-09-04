//! audit/1 wire leg (M9 batch 7 slice 7): hand the per-block touch
//! bits to the core and relay its conviction verdict — the tenth
//! judgment family. Rust computes no policy here: the disjunction
//! (either side touched convicts) and the zero-tolerance threshold
//! both come back on the wire (ADR-008). Unlike the other judged
//! faces this leg serves a FAIL-OPEN surface: every error path
//! answers None, which the Stop/precommit callers already render as
//! a VISIBLE degraded skip (A9f) — a missing core can never block a
//! session, and can never silently pass one either.

use crate::corelink::Link;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::path::Path;

/// The core's judgment over the touched-block bits, plus the
/// rendered display lines Rust re-labels from the convicted row
/// indices (paths never cross the wire, §5.9.2).
pub struct Verdict {
    /// The zero-tolerance verdict itself — the ONLY block/pass bit.
    pub fail: bool,
    /// Full conviction count (the observe feed and the reason line
    /// both say the true number, not the display cap).
    pub dups: usize,
    /// First 10 convicted blocks, rendered; display truncation only.
    pub shown: Vec<String>,
}

/// The audit's core link — the shared head every judged surface
/// speaks (lockstep::open_family), demanding audit/1. None = no
/// spawnable core, or a pre-2.24 core without the capability: the
/// callers' visible degraded skip; local re-judging stays forbidden.
pub(super) fn open() -> Option<Link> {
    let bin = crate::daemon::judge::core_bin()?;
    crate::lockstep::open_family(&bin, "audit/1").ok()
}

/// Measure (set membership per block side), judge (one audit.request
/// over the audit's link), re-label. None = the pipeline, the core
/// or the wire failed — DEGRADED, stamped in the observe entry,
/// never conflated with "no duplicates" (A9f).
pub fn judge(root: &Path, changed: &[String], link: Option<&mut Link>) -> Option<Verdict> {
    let (found, _) = crate::dedup::analyze(root, None, None, None).ok()?;
    // a set, not a Vec scan: blocks × changed was O(B·C) string
    // compares on every Stop (~50 M on a large index)
    let touched: HashSet<&str> = changed.iter().map(String::as_str).collect();
    let rows: Vec<[i64; 2]> = found
        .blocks
        .iter()
        .map(|b| {
            [
                i64::from(touched.contains(b.a_file.as_str())),
                i64::from(touched.contains(b.b_file.as_str())),
            ]
        })
        .collect();
    let reply = link?.request("audit", json!({ "rows": rows })).ok()?;
    consume(&reply, &found.blocks)
}

/// The reply, consumed: convicted indices re-labelled into display
/// lines. A degraded reply (over-cap) or an out-of-range index (wire
/// skew) answers None — the callers' degraded path, by design.
fn consume(reply: &Value, blocks: &[crate::dedup::pairs::Block]) -> Option<Verdict> {
    if reply["degraded"] == json!(true) {
        return None;
    }
    let dups: Vec<usize> = serde_json::from_value(reply["dups"].clone()).ok()?;
    let shown = dups
        .iter()
        .take(10)
        .map(|&i| {
            let b = blocks.get(i)?;
            Some(format!(
                "{}:{}-{} <-> {}:{}-{} ({} tokens)",
                b.a_file, b.a_start, b.a_end, b.b_file, b.b_start, b.b_end, b.tokens
            ))
        })
        .collect::<Option<Vec<String>>>()?;
    Some(Verdict {
        fail: reply["fail"] == json!(true),
        dups: dups.len(),
        shown,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/audit/verdict.rs"]
mod tests;
