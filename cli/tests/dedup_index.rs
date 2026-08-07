//! Index-layer tests: cross-file T2 clone detection, the incremental
//! ≡ full-rebuild equivalence (plan §7.3 property, deterministic core
//! here; proptest randomization lands with the M2 acceptance), the
//! content-hash fast path, and deleted-file reaping.

use codeeraser::dedup::{Params, index::Index, pairs, tokens};
use codeeraser::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

/// ~60 normalized tokens; `seed` renames identifiers and changes
/// literals so pairs of outputs are T2 (not T1) clones.
fn rust_fn(seed: u32) -> String {
    format!(
        "fn work_{seed}(input_{seed}: &[i64], limit_{seed}: i64) -> i64 {{
    let mut total_{seed} = {seed};
    for value_{seed} in input_{seed} {{
        if *value_{seed} > limit_{seed} {{
            total_{seed} += value_{seed} * {seed} + 7;
        }} else {{
            total_{seed} -= value_{seed} / 3;
        }}
    }}
    total_{seed}
}}
"
    )
}

#[test]
fn cross_file_t2_clone_detected() {
    let dir = tmp("t2-clone");
    let p = Params::default();
    let mut idx = Index::open(&dir.join("index.db"), p).expect("open");
    idx.refresh_file("a.rs", rust_fn(1).as_bytes(), Lang::Rust, p)
        .expect("a");
    idx.refresh_file("b.rs", rust_fn(2).as_bytes(), Lang::Rust, p)
        .expect("b");
    let mut streams = pairs::Streams::new();
    streams.insert(
        "a.rs".into(),
        tokens::stream(rust_fn(1).as_bytes(), Lang::Rust).expect("sa"),
    );
    streams.insert(
        "b.rs".into(),
        tokens::stream(rust_fn(2).as_bytes(), Lang::Rust).expect("sb"),
    );
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
    let dir = tmp("incr-full");
    let p = Params::default();
    let mut incr = Index::open(&dir.join("incr.db"), p).expect("open");
    for (name, seed) in [("a.rs", 1u32), ("b.rs", 2), ("c.rs", 3)] {
        incr.refresh_file(name, rust_fn(seed).as_bytes(), Lang::Rust, p)
            .expect("seed");
    }
    // mutate b.rs (append a second function) + drop c.rs
    let b2 = format!("{}{}", rust_fn(2), rust_fn(9));
    incr.refresh_file("b.rs", b2.as_bytes(), Lang::Rust, p)
        .expect("mutate");
    let live: BTreeSet<String> = ["a.rs".into(), "b.rs".into()].into();
    incr.remove_missing(&live).expect("reap");

    let mut full = Index::open(&dir.join("full.db"), p).expect("open");
    full.refresh_file("a.rs", rust_fn(1).as_bytes(), Lang::Rust, p)
        .expect("a");
    full.refresh_file("b.rs", b2.as_bytes(), Lang::Rust, p)
        .expect("b");

    assert_eq!(
        incr.all_instances().expect("incr"),
        full.all_instances().expect("full"),
        "incremental refresh must equal full rebuild"
    );
}

#[test]
fn unchanged_content_is_fast_path() {
    let dir = tmp("fast-path");
    let p = Params::default();
    let mut idx = Index::open(&dir.join("index.db"), p).expect("open");
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
    let dir = tmp("param-wipe");
    let p1 = Params::default();
    let src = rust_fn(5);
    {
        let mut idx = Index::open(&dir.join("index.db"), p1).expect("open p1");
        assert!(
            idx.refresh_file("a.rs", src.as_bytes(), Lang::Rust, p1)
                .expect("seed")
        );
    }
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

#[test]
fn removed_file_is_purged() {
    let dir = tmp("purge");
    let p = Params::default();
    let mut idx = Index::open(&dir.join("index.db"), p).expect("open");
    idx.refresh_file("a.rs", rust_fn(1).as_bytes(), Lang::Rust, p)
        .expect("a");
    idx.refresh_file("b.rs", rust_fn(2).as_bytes(), Lang::Rust, p)
        .expect("b");
    let live: BTreeSet<String> = ["a.rs".into()].into();
    assert_eq!(idx.remove_missing(&live).expect("reap"), 1);
    let left = idx.all_instances().expect("instances");
    assert!(left.iter().all(|i| i.file == "a.rs"), "cascade must purge");
    assert!(!left.is_empty());
}
