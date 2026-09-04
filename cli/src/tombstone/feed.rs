//! The `tombstone` object of the observe feed (`hookio::OBSERVE_SCHEMA`
//! 0.9.0): the measurement and the judgment side by side, no name text
//! ever. Every producer — the PreToolUse leg, the Stop audit, precommit
//! — writes this one shape; the FPR ledger and the evaluation set read it.

use super::{Findings, Judgment, Row, TOMBSTONE_REV};

/// How many erased keys one feed line carries at most (a whole-file
/// rewrite erases hundreds of names; the session union is a bounded
/// read, so the record is bounded too).
pub const HASH_CAP: usize = 256;
/// How many sites a feed line names; the counts stay exact.
pub const SITE_CAP: usize = 10;

/// The feed object every producer writes: the measurement — the erased
/// count, the candidate rows, the exemptions with their witness (a
/// segment's entry says where it starts) — and the core's judgment
/// under `judged` (the first sites as `file:line kind`, the split,
/// the condition) or the named reason there is none; the per-edit
/// leg, which carries names across a session, adds the erased keys
/// (capped) and the session union's size.
pub fn feed_json(f: &Findings, session: Option<usize>, judged: &Judgment) -> serde_json::Value {
    let exempt: Vec<serde_json::Value> = f
        .exempt
        .iter()
        .map(|e| {
            let mut v = serde_json::json!({"file": e.file, "why": e.why.name()});
            if let Some(line) = e.line {
                v["line"] = serde_json::json!(line);
            }
            v
        })
        .collect();
    let mut line = serde_json::json!({
        "rev": TOMBSTONE_REV,
        "erased": f.erased.len(),
        "rows": f.rows.len(),
        "exempt": exempt,
        "judged": judged_json(f, judged),
    });
    if let Some(carried) = session {
        let hashes: Vec<u64> = f.erased.iter().take(HASH_CAP).map(|n| n.key).collect();
        line["erased_hashes"] = serde_json::json!(hashes);
        line["session_erased"] = serde_json::json!(carried);
    }
    line
}

/// The judgment as the feed carries it: the core's answer re-labelled
/// into places, or the reason there is none.
fn judged_json(f: &Findings, judged: &Judgment) -> serde_json::Value {
    match judged {
        Ok(j) => {
            let sites: Vec<String> = f.judged_rows(j).take(SITE_CAP).map(Row::place).collect();
            serde_json::json!({"sites": sites, "label": j.label, "prose": j.prose, "over": j.over})
        }
        Err(why) => serde_json::json!({"degraded": why}),
    }
}
