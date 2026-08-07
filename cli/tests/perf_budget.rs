//! M2 performance budgets (plan §6 M2): 100k-LOC full index < 30s,
//! single-file incremental < 200ms, probe round-trip p95 < 150ms.
//! Ignored by default — run in RELEASE (debug numbers are not the
//! budget's currency; assertions only fire in release):
//!   cargo test --release --test perf_budget -- --ignored --nocapture
//! Results are recorded in docs/PERF-BUDGET.md.

use codeeraser::daemon::client;
use codeeraser::daemon::proto::{Request, Response};
use codeeraser::dedup::{Params, analyze, index::Index};
use codeeraser::scan::lang::Lang;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn corpus_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

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

/// >100k LOC: 460 files x 20 fns x ~11 lines. seed % 900 forms
/// realistic clone families (~10 instances) without hot-cap regimes.
fn generate_corpus(dir: &Path) -> usize {
    let mut lines = 0;
    for f in 0..460u32 {
        let mut src = String::new();
        for i in 0..20u32 {
            src.push_str(&rust_fn((f * 20 + i) % 900));
        }
        lines += src.lines().count();
        std::fs::write(dir.join(format!("f{f:03}.rs")), src).expect("write");
    }
    lines
}

fn release_assert(ok: bool, msg: &str) {
    if cfg!(debug_assertions) {
        println!("(debug build: budget assertion skipped) {msg}");
    } else {
        assert!(ok, "{msg}");
    }
}

#[test]
#[ignore = "perf budget: run in release, records docs/PERF-BUDGET.md"]
fn full_index_100k_loc_under_30s() {
    let dir = corpus_dir("perf-100k");
    let loc = generate_corpus(&dir);
    println!("corpus: {loc} LOC");
    let t0 = Instant::now();
    let (found, _) = analyze(&dir, None, None).expect("analyze");
    let full = t0.elapsed();
    println!(
        "full index+verify: {:.2?} ({} blocks)",
        full,
        found.blocks.len()
    );
    release_assert(
        full.as_secs_f64() < 30.0,
        &format!("full index {full:.2?} exceeds 30s budget"),
    );
}

#[test]
#[ignore = "perf budget: run in release, records docs/PERF-BUDGET.md"]
fn incremental_single_file_under_200ms() {
    let dir = corpus_dir("perf-incr");
    generate_corpus(&dir);
    analyze(&dir, None, None).expect("seed index");
    // mutate one file, then time ONLY its index refresh (the budgeted
    // unit); the warm full analyze is printed as context
    let target = dir.join("f000.rs");
    let mutated = format!("{}{}", rust_fn(1_000), rust_fn(1_001));
    std::fs::write(&target, &mutated).expect("mutate");
    let p = Params::default();
    let mut idx = Index::open(&dir.join(".ce/index.db"), p).expect("open");
    let t0 = Instant::now();
    let changed = idx
        .refresh_file("f000.rs", mutated.as_bytes(), Lang::Rust, p)
        .expect("refresh");
    let refresh = t0.elapsed();
    assert!(changed, "mutation must be re-indexed");
    drop(idx);
    let t1 = Instant::now();
    analyze(&dir, None, None).expect("warm analyze");
    println!(
        "single-file refresh: {refresh:.2?}; warm full analyze: {:.2?}",
        t1.elapsed()
    );
    release_assert(
        refresh.as_millis() < 200,
        &format!("incremental refresh {refresh:.2?} exceeds 200ms budget"),
    );
}

#[test]
#[ignore = "perf budget: run in release, records docs/PERF-BUDGET.md"]
fn probe_round_trip_p95_under_150ms() {
    let root = corpus_dir("perf-probe");
    generate_corpus(&root);
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_ce"))
        .arg("daemon")
        .arg(&root)
        .env("CE_DAEMON_IDLE_SECS", "120")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn daemon");
    let mut ready = false;
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if client::request(&root, &Request::Ping).is_ok() {
            ready = true;
            break;
        }
    }
    assert!(ready, "daemon never came up");
    let mut rtts: Vec<u128> = Vec::with_capacity(100);
    for _ in 0..100 {
        let t0 = Instant::now();
        let r = client::request(&root, &Request::Ping).expect("ping");
        assert!(matches!(r, Response::Pong { .. }));
        rtts.push(t0.elapsed().as_micros());
    }
    rtts.sort_unstable();
    let p95 = rtts[94] as f64 / 1000.0;
    println!(
        "probe RTT over pipe: median {:.2} ms, p95 {p95:.2} ms, max {:.2} ms",
        rtts[49] as f64 / 1000.0,
        rtts[99] as f64 / 1000.0
    );
    let _ = client::request(&root, &Request::Shutdown);
    let _ = child.wait();
    release_assert(p95 < 150.0, &format!("probe p95 {p95:.2}ms exceeds budget"));
}
