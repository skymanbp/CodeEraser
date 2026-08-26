//! A concurrent out-of-process wipe must not be laundered into a
//! completeness claim. The index is wiped whenever the schema,
//! tokenizer or graph revisions differ — routine between a SHA-pinned
//! hook binary and a `cargo install`ed terminal one — and the writer
//! in flight noticed nothing: the files it had already indexed were
//! gone, everything it wrote afterwards landed under the wiper's cache
//! key, and it then stamped `full_build` over the result. Coldstart
//! read that stamp, answered READY, and probes reported matches=0
//! against a truncated index: a silent clean, which is the one answer
//! this product may never give.

use crate::common;
use codeeraser::dedup::{Params, index::Index};
use std::path::Path;

/// Another binary's revision mismatch, performed the way it really
/// arrives: a connection that is nobody's Index ages the stored
/// tokenizer revision, and the next `Index::open` finds the cache key
/// stale and recreates every table. Both probes need it, and spelling
/// the raw-open + read + UPDATE stanza out per probe re-cloned
/// dedup_index.rs's (dedup gate, this batch) — the connection is
/// chained, never named, so the two helpers share no statement shape.
fn age_cache_key(db: &Path) {
    rusqlite::Connection::open(db)
        .expect("open raw")
        .execute("UPDATE meta SET v = v + 1 WHERE k = 'tokenizer_rev'", [])
        .expect("age the cache key");
}

/// One meta scalar, read from outside — what a second process sees.
fn meta(db: &Path, k: &str) -> i64 {
    rusqlite::Connection::open(db)
        .expect("open raw")
        .query_row("SELECT v FROM meta WHERE k = ?1", (k,), |r| r.get(0))
        .expect("meta row exists")
}

/// The stamp asserts "a full analyze completed on THIS database".
/// After someone else rebuilds it, that sentence is false and the
/// stamp must be refused by name — a rebuild is a cost, a wrong
/// answer is a defect.
#[test]
fn a_wipe_under_a_live_writer_refuses_the_completeness_stamp() {
    let dir = common::tmp("epoch-wipe");
    let db = dir.join(".ce/index.db");

    let mut writer = Index::open(&db, Params::default()).expect("open");
    writer.mark_full_build().expect("stamp under our own epoch");

    // the wipe itself: age the key out of process, then let the next
    // opener rebuild — our `writer` handle notices nothing
    age_cache_key(&db);
    drop(Index::open(&db, Params::default()).expect("wiper rebuilds"));

    let err = writer
        .mark_full_build()
        .expect_err("the stamp must not survive someone else's rebuild");
    let text = format!("{err:#}");
    assert!(
        text.contains("rebuilt by another process"),
        "the refusal must name what happened, got: {text}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// The generation changes on every wipe, so two rebuilds are never
/// mistaken for the same database — without this the check above
/// could pass vacuously on a constant.
#[test]
fn every_rebuild_mints_a_new_generation() {
    let dir = common::tmp("epoch-distinct");
    let db = dir.join(".ce/index.db");
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..3 {
        drop(Index::open(&db, Params::default()).expect("open"));
        seen.insert(meta(&db, "epoch"));
        age_cache_key(&db);
    }
    assert_eq!(seen.len(), 3, "each wipe is its own generation: {seen:?}");
    std::fs::remove_dir_all(&dir).ok();
}
