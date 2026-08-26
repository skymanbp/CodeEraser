//! `INDEX_READY` is a cache of a cross-process fact, not the fact.
//!
//! The index db is wiped out of process whenever any of the schema,
//! tokenizer or graph revisions differ (dedup::schema), which happens
//! routinely between a SHA-pinned hook binary and a `cargo install`ed
//! terminal one — they wipe each other on every invocation. A READY
//! flag that never re-checked then answered every probe `matches: []`,
//! which is a clean probe: the silent-failure class coldstart.rs opens
//! by forbidding, arriving through its own state (review 2026-08-19).
//!
//! ONE test in its own binary on purpose: `INDEX_STATE` is a process
//! global, so a second test here would race it.

use std::path::PathBuf;

#[test]
fn a_ready_flag_over_a_missing_index_stops_claiming_ready() {
    use codeeraser::daemon::coldstart::{index_ready, mark_ready};

    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("coldstart-state");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");

    // exactly what a finished Dedup request does (server.rs), and what
    // a completed cold-start build does
    mark_ready();
    assert!(
        !index_ready(&dir),
        "READY must re-consult the full-build stamp: this root has no index at all, \
         and answering ready here is a probe that reports matches=0 on nothing"
    );

    std::fs::remove_dir_all(&dir).ok();
}
