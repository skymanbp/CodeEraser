//! M7-P4 acceptance: trend data re-derives from git history — the
//! cache is a CACHE (charter ruling ②). Three legs on one seeded
//! two-commit repo: (1) a full run measures every mainline commit;
//! (2) wiping the database and re-running reproduces the SAME rows
//! (history is the source of truth); (3) an untouched re-run is a
//! pure cache read — same rows, nothing pending, nothing failed.

use crate::common;
use crate::common::{git, rust_fn, tmp};
use std::path::Path;

fn seed_two_commits(dir: &Path) {
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    git(dir, &["init", "-q"]);
    common::commit_all(dir, "one");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs");
    common::commit_all(dir, "two");
}

fn run_all(dir: &Path, core: &str) -> codeeraser::trend::Report {
    codeeraser::trend::run(dir, None, core, 10, None).expect("trend run")
}

#[test]
fn trend_rows_rebuild_from_history() {
    let dir = tmp("trend-rebuild");
    seed_two_commits(&dir);
    let core = common::core_bin();

    let first = run_all(&dir, &core);
    assert_eq!(first.window, 2, "two mainline commits seeded");
    assert_eq!(first.rows.len(), 2, "both measured: {:?}", first.failed);
    assert_eq!((first.pending, first.failed.len()), (0, 0));
    assert!(
        first.rows[0].ts <= first.rows[1].ts,
        "rows come oldest-first"
    );
    // the second commit lands a T2 clone pair, so its judged facts
    // must differ — a constant-score fixture would prove nothing
    // about per-commit measurement (commit hashes differing is not
    // evidence; the judgment payload is)
    assert!(
        (first.rows[0].score, &first.rows[0].axes) != (first.rows[1].score, &first.rows[1].axes),
        "the two commits must JUDGE differently: {:?}",
        first.rows
    );

    // Leg 3 first (cheap): an untouched re-run is a pure cache read.
    let cached = run_all(&dir, &core);
    assert_eq!(cached.rows, first.rows, "cache read drifted");

    // Leg 2: history is the truth — wipe the whole database and the
    // same rows come back from re-measurement.
    std::fs::remove_dir_all(dir.join(".ce")).expect("wipe .ce");
    let rebuilt = run_all(&dir, &core);
    assert_eq!(
        rebuilt.rows, first.rows,
        "rebuild from history must reproduce the cached rows"
    );
    assert_eq!((rebuilt.pending, rebuilt.failed.len()), (0, 0));
}

/// Batching arithmetic: batch=1 measures exactly one commit per run
/// and reports the remainder as pending — the GUI's progress loop
/// contract, pinned as a table over consecutive runs.
#[test]
fn trend_batch_measures_incrementally() {
    let dir = tmp("trend-batch");
    seed_two_commits(&dir);
    let core = common::core_bin();

    for (rows_want, pending_want) in [(1, 1), (2, 0)] {
        let r = codeeraser::trend::run(&dir, None, &core, 10, Some(1)).expect("batch run");
        assert_eq!((r.rows.len(), r.pending), (rows_want, pending_want));
    }
}
