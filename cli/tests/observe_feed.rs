//! Observe-feed contract (ce.observe/0.2.0): the NDJSON feed is the
//! M4 evaluation-set raw material, so its line shape is pinned by a
//! golden. One deterministic run of all three producers — probe,
//! stop audit, precommit — with volatile fields normalized (ts_ms,
//! elapsed_ms, absolute file path); key sets, schema/event tags,
//! counts, mode, and degraded flags must match byte-for-byte.
//! Bless flow: `CE_BLESS=1 cargo test --test observe_feed`.
//!
//! `session_id` is deliberately NOT normalized: the hook envelopes
//! carry the literal "t", so the golden pins that the id survives the
//! whole path — and that precommit, which is not a hook and owns no
//! session, records null instead of borrowing one.

mod common;

/// The whole feed, volatile fields zeroed, as pretty JSON.
fn normalized_feed(dir: &std::path::Path) -> String {
    let log = std::fs::read_to_string(dir.join(".ce/observe.ndjson")).expect("feed");
    let entries: Vec<serde_json::Value> = log
        .lines()
        .map(|l| {
            let mut j: serde_json::Value = serde_json::from_str(l).expect("entry json");
            j["ts_ms"] = serde_json::json!(0);
            if j.get("elapsed_ms").is_some() {
                j["elapsed_ms"] = serde_json::json!(0);
            }
            if j.get("file").is_some() {
                j["file"] = serde_json::json!("<file>");
            }
            j
        })
        .collect();
    serde_json::to_string_pretty(&entries).expect("serialize")
}

#[test]
fn feed_shape_matches_golden() {
    let dir = common::tmp("observe-golden");
    common::seed_git_clone_repo(&dir, "observe");
    common::build_index(&dir);
    // entry 1: probe (T2 rewrite of indexed content -> matches)
    let env = common::pretooluse_envelope(&dir, "Write", &common::rust_fn(3));
    common::run_hook(&dir, &["probe", "--hook"], &env);
    common::shutdown_daemon(&dir);
    // entry 2: stop audit (staged b.rs = one touched duplicate)
    common::run_hook(
        &dir,
        &["audit", "--hook"],
        &common::stop_envelope(&dir, false),
    );
    // entry 3: precommit (observe mode reports but exits 0)
    assert!(common::run_ce(&dir, &["precommit"]).status.success());
    common::assert_matches_golden(
        &normalized_feed(&dir),
        &common::golden_path("observe-feed/feed.golden.json"),
    );
}
