//! M5-3h churn-attribution ledger (3h exit row): five blind auditors
//! independently reimplemented the attribution spec (pairing, minimal
//! diff, unit segmentation, nth, owner, rewrite) and derived the
//! per-unit ledger for the 40 most recent qualifying commits at the
//! pinned tip. The frozen ledger is THEIRS (14 commits adjudicated to
//! the product's Myers under the equal-minimal-alignment conservation
//! certificate, both views preserved in the doc); the product must
//! match it row for row — conservation alone cannot catch systematic
//! misattribution (wrong nth order, outermost owner, nth-keyed
//! rewrite), which is exactly why this instrument exists.
//!
//! Gates: `ledger_structure_is_sound` runs everywhere (doc-internal
//! consistency); the replay and conservation gates need the real git
//! histories (self at the pinned tip; corpora under .ce-eval), so
//! they are ignored in CI and run locally:
//!   cargo test --test eval_churn_ledger -- --ignored --nocapture

mod common;
mod eval_support;

use serde_json::Value;
use std::collections::BTreeMap;

fn doc() -> Value {
    eval_support::load(&eval_support::eval_doc("churn-ledger"))
}

/// (path, key, nth, appended, rewrote) from one frozen row array —
/// the row IS the tuple, so serde does the shape checking.
fn frozen_row(v: &Value) -> (String, String, i64, usize, usize) {
    serde_json::from_value(v.clone()).expect("row [path,key,nth,appended,rewrote]")
}

/// Validate one frozen commit (hex sha; rows sorted, identity-unique,
/// non-vacuous) and return (agent, rows, appended, rewrote).
fn checked_commit(c: &Value) -> (String, u64, u64, u64) {
    let sha = c["sha"].as_str().expect("sha");
    assert!(
        sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit()),
        "full hex sha: {sha}"
    );
    let rows: Vec<_> = c["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(frozen_row)
        .collect();
    let (mut ap, mut rw) = (0u64, 0u64);
    let ids: Vec<_> = rows
        .iter()
        .map(|(p, k, n, a, r)| {
            assert!(a + r >= 1, "vacuous row in {sha}");
            ap += *a as u64;
            rw += *r as u64;
            (p.clone(), k.clone(), *n)
        })
        .collect();
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(ids, sorted, "{sha}: rows sorted and identity-unique");
    let agent = c["agent"].as_str().expect("agent").to_string();
    (agent, rows.len() as u64, ap, rw)
}

#[test]
fn ledger_structure_is_sound() {
    let d = doc();
    assert_eq!(d["schema"], "ce.churn-ledger/1");
    let commits = d["commits"].as_array().expect("commits");
    assert_eq!(commits.len(), 40, "the 40 qualifying commits");
    assert_eq!(
        commits[0]["sha"], d["tip"],
        "log order starts at the pinned tip"
    );
    let mut seats: BTreeMap<String, usize> = BTreeMap::new();
    let (mut rows_n, mut appended, mut rewrote) = (0u64, 0u64, 0u64);
    let mut shas: Vec<String> = Vec::new();
    for c in commits {
        shas.push(c["sha"].as_str().expect("sha").to_string());
        let (agent, n, ap, rw) = checked_commit(c);
        *seats.entry(agent).or_default() += 1;
        rows_n += n;
        appended += ap;
        rewrote += rw;
    }
    shas.sort();
    shas.dedup();
    assert_eq!(shas.len(), 40, "40 distinct commits");
    let want: BTreeMap<String, usize> = ["A", "B", "C", "D", "E"]
        .iter()
        .map(|a| (a.to_string(), 8))
        .collect();
    assert_eq!(seats, want, "five auditors, eight commits each");
    assert_eq!(d["totals"]["rows"].as_u64(), Some(rows_n));
    assert_eq!(d["totals"]["appended"].as_u64(), Some(appended));
    assert_eq!(d["totals"]["rewrote"].as_u64(), Some(rewrote));
    assert!(rows_n > 0, "an all-empty ledger audits nothing");
}
