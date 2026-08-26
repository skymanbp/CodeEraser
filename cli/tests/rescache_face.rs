//! The analyze result cache at the user face (K step 10,
//! dedup/rescache.rs): a warm re-run must serve byte-identical
//! output, and a content change must reach the report on the very
//! next run. The decisive hit-path counterfactual (a poisoned slot
//! served verbatim) lives in the rescache unit battery, where the
//! slot is reachable; this face proves the cache can never be SEEN
//! from the outside except as speed.

mod common;
use common::fixtures::{rust_fn, seed_clone_pair, tmp};
use common::gates::run_expect;

/// Cold run, warm hit, and a fresh recompute after `.ce` is wiped:
/// the judgment content (blocks + groups) is identical in all three,
/// non-empty (a vacuous equality would prove nothing), while the
/// summary's refresh facts stay THIS run's — `refreshed` honestly
/// reads 2/0/2, exactly as it did before the cache existed.
#[test]
fn served_and_recomputed_judgments_are_identical() {
    let dir = tmp("rescache-bytes");
    seed_clone_pair(&dir);
    let first = report(&run_expect(&dir, &["dedup", ".", "--format", "json"]));
    assert!(
        first["blocks"].to_string().contains("a.rs"),
        "anti-vacuity: the seeded pair must be reported"
    );
    let warm = report(&run_expect(&dir, &["dedup", ".", "--format", "json"]));
    std::fs::remove_dir_all(dir.join(".ce")).expect("wipe .ce");
    let fresh = report(&run_expect(&dir, &["dedup", ".", "--format", "json"]));
    for (label, other) in [("warm hit", &warm), ("fresh recompute", &fresh)] {
        assert_eq!(first["blocks"], other["blocks"], "{label}");
        assert_eq!(first["groups"], other["groups"], "{label}");
    }
    let refreshed: Vec<_> = [&first, &warm, &fresh]
        .iter()
        .map(|r| r["summary"]["refreshed"].as_u64())
        .collect();
    assert_eq!(
        refreshed,
        [Some(2), Some(0), Some(2)],
        "refresh facts are per-run, never replayed from the slot"
    );
}

fn report(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout).expect("report json")
}

/// The next run after a content move reports the moved content —
/// the invalidation is invisible except in the answer being current.
#[test]
fn a_new_clone_lands_in_the_very_next_report() {
    let dir = tmp("rescache-invalidate");
    seed_clone_pair(&dir);
    let before = run_expect(&dir, &["dedup", ".", "--format", "json"]);
    assert!(!before.contains("c.rs"));
    std::fs::write(dir.join("c.rs"), rust_fn(3)).expect("c.rs");
    let after = run_expect(&dir, &["dedup", ".", "--format", "json"]);
    assert!(
        after.contains("c.rs"),
        "the report must be current on the first run after the change"
    );
}
