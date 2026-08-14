//! Index-layer tests: cross-file T2 clone detection, the incremental
//! ≡ full-rebuild equivalence (plan §7.3 property, deterministic core
//! here; proptest randomization lands with the M2 acceptance), the
//! content-hash fast path, deleted-file reaping, and the schema-v4
//! Markdown/graph-cache behavior. Arrange steps go through open_idx
//! and put — the dedup ratchet counts test scaffolding too.

use codeeraser::dedup::{Params, index::Index, pairs, tokens};
use codeeraser::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::PathBuf;

mod common;
use common::{rust_fn, tmp};

/// Shared arrange throat: a temp dir and an opened index inside it.
fn open_idx(tag: &str, db: &str) -> (PathBuf, Index) {
    let dir = tmp(tag);
    let idx = Index::open(&dir.join(db), Params::default()).expect("open");
    (dir, idx)
}

/// Refresh one rust source into the index (default params).
fn put(idx: &mut Index, name: &str, src: &str) {
    idx.refresh_file(name, src.as_bytes(), Lang::Rust, Params::default())
        .expect(name);
}

/// Two connections opening a fresh index concurrently must both
/// succeed: the schema rebuild is double-checked under an IMMEDIATE
/// write lock, so racing openers serialize instead of interleaving
/// DROP/CREATE statements (the daemon_e2e failure CI caught twice —
/// "table files/sites already exists").
#[test]
fn concurrent_open_rebuilds_once() {
    let dir = tmp("race-open");
    let db = dir.join("index.db");
    for round in 0..8 {
        for suffix in ["", "-wal", "-shm"] {
            std::fs::remove_file(dir.join(format!("index.db{suffix}"))).ok();
        }
        let barrier = std::sync::Barrier::new(2);
        std::thread::scope(|s| {
            let handles: Vec<_> = (0..2)
                .map(|_| {
                    s.spawn(|| {
                        barrier.wait();
                        Index::open(&db, Params::default()).map(drop)
                    })
                })
                .collect();
            for h in handles {
                h.join()
                    .expect("join")
                    .unwrap_or_else(|e| panic!("round {round}: {e:#}"));
            }
        });
    }
}

#[test]
fn cross_file_t2_clone_detected() {
    let p = Params::default();
    let (_dir, mut idx) = open_idx("t2-clone", "index.db");
    put(&mut idx, "a.rs", &rust_fn(1));
    put(&mut idx, "b.rs", &rust_fn(2));
    let mut streams = pairs::Streams::new();
    for (name, seed) in [("a.rs", 1u32), ("b.rs", 2)] {
        streams.insert(
            name.into(),
            tokens::stream(rust_fn(seed).as_bytes(), Lang::Rust).expect(name),
        );
    }
    let instances = idx.all_instances().expect("instances");
    let filter = pairs::Filter {
        min_tokens: p.guarantee(),
        min_distinct: pairs::DEFAULT_MIN_DISTINCT,
    };
    let found = pairs::clone_blocks(&instances, &streams, filter);
    assert_eq!(found.hot_chained, 0);
    assert_eq!(found.stale_skipped, 0);
    assert!(!found.blocks.is_empty(), "T2 clone must be detected");
    let b = &found.blocks[0];
    assert_eq!((b.a_file.as_str(), b.b_file.as_str()), ("a.rs", "b.rs"));
    assert!(b.tokens >= p.guarantee(), "verified length above t");
    assert!(b.a_start >= 1 && b.a_end <= 12, "range within the function");
}

/// Property core: mutate one file, refresh incrementally — the index
/// must equal a from-scratch rebuild of the same tree.
#[test]
fn incremental_equals_full_rebuild() {
    let (_da, mut incr) = open_idx("incr-full-a", "incr.db");
    for (name, seed) in [("a.rs", 1u32), ("b.rs", 2), ("c.rs", 3)] {
        put(&mut incr, name, &rust_fn(seed));
    }
    // mutate b.rs (append a second function) + drop c.rs
    let b2 = format!("{}{}", rust_fn(2), rust_fn(9));
    put(&mut incr, "b.rs", &b2);
    let live: BTreeSet<String> = ["a.rs".into(), "b.rs".into()].into();
    incr.remove_missing(&live).expect("reap");

    let (_db, mut full) = open_idx("incr-full-b", "full.db");
    put(&mut full, "a.rs", &rust_fn(1));
    put(&mut full, "b.rs", &b2);

    assert_eq!(
        incr.all_instances().expect("incr"),
        full.all_instances().expect("full"),
        "incremental refresh must equal full rebuild"
    );
}

#[test]
fn unchanged_content_is_fast_path() {
    let p = Params::default();
    let (_dir, mut idx) = open_idx("fast-path", "index.db");
    let src = rust_fn(5);
    assert!(
        idx.refresh_file("a.rs", src.as_bytes(), Lang::Rust, p)
            .expect("first")
    );
    assert!(
        !idx.refresh_file("a.rs", src.as_bytes(), Lang::Rust, p)
            .expect("second"),
        "identical bytes must not rewrite"
    );
}

/// Attack-review D2 regression: the index cache key includes the
/// winnowing params — reopening with different params wipes the
/// cache instead of silently serving stale fingerprints.
#[test]
fn param_change_invalidates_index() {
    let src = rust_fn(5);
    let (dir, mut idx) = open_idx("param-wipe", "index.db");
    put(&mut idx, "a.rs", &src);
    drop(idx);
    let p2 = Params {
        kgram: 8,
        window: 8,
    };
    let mut idx = Index::open(&dir.join("index.db"), p2).expect("open p2");
    assert!(
        idx.refresh_file("a.rs", src.as_bytes(), Lang::Rust, p2)
            .expect("after wipe"),
        "changed params must invalidate the cached fingerprints"
    );
}

/// 3d exit criterion: docdup_rev sits in the meta cache key, so a
/// bumped extraction revision wipes stale docsegs rows. The const
/// cannot change in-process, so the bump is simulated by rewriting
/// the stored meta value — same mechanism, same wipe.
#[test]
fn docdup_rev_bump_invalidates_index() {
    // hard-wrapped: a single 300-char comment line would (correctly)
    // fall to the REV-3 overlong mask instead of caching
    let line = format!("   {}\n", "word ".repeat(20).trim());
    let src = format!("/* prose\n{}*/\nfn main() {{}}\n", line.repeat(3));
    let (dir, mut idx) = open_idx("docdup-rev", "index.db");
    idx.refresh_file("a.rs", src.as_bytes(), Lang::Rust, Params::default())
        .expect("first");
    drop(idx);
    let db = dir.join("index.db");
    let conn = rusqlite::Connection::open(&db).expect("raw open");
    let segs: i64 = conn
        .query_row("SELECT COUNT(*) FROM docsegs", [], |r| r.get(0))
        .expect("count");
    assert_eq!(segs, 1, "the 60-word comment block must be cached");
    conn.execute("UPDATE meta SET v = v + 1 WHERE k = 'docdup_rev'", [])
        .expect("bump");
    drop(conn);
    let mut idx = Index::open(&db, Params::default()).expect("reopen");
    assert!(
        idx.refresh_file("a.rs", src.as_bytes(), Lang::Rust, Params::default())
            .expect("after wipe"),
        "a bumped docdup_rev must wipe the cache"
    );
}

/// Schema v4: Markdown enters `files` as a zero-fingerprint graph
/// row — the instance set stays empty (the dedup ratchet is
/// structurally untouched), the content-hash fast path covers it,
/// the file count includes it, and the phase-2 gate hands the cached
/// sites (code AND Markdown) to the resolver callback with no
/// re-parse.
#[test]
fn markdown_is_graph_cache_not_fingerprints() {
    let p = Params::default();
    let (_dir, mut idx) = open_idx("md-graph", "index.db");
    assert!(
        idx.refresh_file("doc.md", b"[x](./a.rs)\n", Lang::Markdown, p)
            .expect("md")
    );
    assert!(
        !idx.refresh_file("doc.md", b"[x](./a.rs)\n", Lang::Markdown, p)
            .expect("md again"),
        "content-hash fast path covers markdown"
    );
    put(&mut idx, "a.rs", "use crate::z;\nfn f() {}\n");
    assert!(
        idx.all_instances().expect("instances").is_empty(),
        "below-threshold code + markdown must add zero fingerprints"
    );
    assert_eq!(idx.file_count().expect("count"), 2, "markdown is indexed");
    let mut specs: Vec<String> = Vec::new();
    assert!(
        idx.ensure_edges_resolved(42, |s| {
            specs.push(s.spec.clone());
            Vec::new()
        })
        .expect("sweep"),
        "fresh key fires the sweep"
    );
    specs.sort();
    assert_eq!(
        specs,
        ["./a.rs", "crate::z"],
        "cached sites reach the resolver callback"
    );
    assert!(
        !idx.ensure_edges_resolved(42, |_| Vec::new()).expect("skip"),
        "unchanged key skips the sweep"
    );
}

#[test]
fn removed_file_is_purged() {
    let (_dir, mut idx) = open_idx("purge", "index.db");
    put(&mut idx, "a.rs", &rust_fn(1));
    put(&mut idx, "b.rs", &rust_fn(2));
    let live: BTreeSet<String> = ["a.rs".into()].into();
    assert_eq!(idx.remove_missing(&live).expect("reap"), 1);
    let left = idx.all_instances().expect("instances");
    assert!(left.iter().all(|i| i.file == "a.rs"), "cascade must purge");
    assert!(!left.is_empty());
}
