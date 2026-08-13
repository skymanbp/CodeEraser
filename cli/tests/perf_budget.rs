//! M2 performance budgets (plan §6 M2): 100k-LOC full index < 30s,
//! single-file incremental < 200ms, probe round-trip p95 < 150ms.
//! M3 budget: `ce probe --hook` end-to-end (ce side) p95 < 1s.
//! Ignored by default — run in RELEASE (debug numbers are not the
//! budget's currency; assertions only fire in release):
//!   cargo test --release --test perf_budget -- --ignored --nocapture
//! Results are recorded in docs/PERF-BUDGET.md.

use codeeraser::daemon::client;
use codeeraser::daemon::proto::{Request, Response};
use codeeraser::dedup::{Params, analyze, index::Index};
use codeeraser::scan::lang::Lang;
use std::path::Path;
use std::time::Instant;

mod common;
use common::{rust_fn, tmp as corpus_dir};

/// 100k+ LOC: 460 files x (site_lines use-decls + 20 fns x ~11
/// lines). seed % 900 forms realistic clone families (~10 instances)
/// without hot-cap regimes. site_lines = 0 reproduces the M2 budget
/// corpus byte-for-byte; site_lines > 0 adds cached graph sites for
/// the M5-2e resolve-sweep budget without forking a second generator.
fn corpus(dir: &Path, site_lines: u32) -> usize {
    let mut lines = 0;
    for f in 0..460u32 {
        let mut src = String::new();
        for i in 0..site_lines {
            src.push_str(&format!("use crate::dep_{}::item_{i};\n", (f + i) % 97));
        }
        for i in 0..20u32 {
            src.push_str(&rust_fn((f * 20 + i) % 900));
        }
        lines += src.lines().count();
        std::fs::write(dir.join(format!("f{f:03}.rs")), src).expect("write");
    }
    lines
}

fn generate_corpus(dir: &Path) -> usize {
    corpus(dir, 0)
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
    let (found, _) = analyze(&dir, None, None, None).expect("analyze");
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
    analyze(&dir, None, None, None).expect("seed index");
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
    analyze(&dir, None, None, None).expect("warm analyze");
    println!(
        "single-file refresh: {refresh:.2?}; warm full analyze: {:.2?}",
        t1.elapsed()
    );
    release_assert(
        refresh.as_millis() < 200,
        &format!("incremental refresh {refresh:.2?} exceeds 200ms budget"),
    );
}

/// M5-2e exit budget (design RG4): a resolve_key change replays the
/// resolver over every cached site; > 2s on 100k LOC narrows phase 2
/// to rung-consulting only changed-directory sites (the trigger is
/// pre-committed in the design brief).
#[test]
#[ignore = "perf budget: run in release, records docs/PERF-BUDGET.md"]
fn resolve_sweep_100k_loc_under_2s() {
    let dir = corpus_dir("perf-resolve");
    // 30 use-decls per file ≈ 13,800 cached sites — the self
    // corpus's site density scaled to 100k LOC
    let loc = corpus(&dir, 30);
    analyze(&dir, None, None, None).expect("seed index");
    let p = Params::default();
    let mut idx = Index::open(&dir.join(".ce/index.db"), p).expect("open");
    let mut sites = 0usize;
    let t0 = Instant::now();
    let fired = idx
        .ensure_edges_resolved(1, |_| {
            sites += 1;
            Vec::new()
        })
        .expect("sweep");
    let sweep = t0.elapsed();
    assert!(fired, "fresh key must fire the sweep");
    assert_eq!(sites, 460 * 30, "every cached site reaches the resolver");
    println!("phase-2 sweep: {sweep:.2?} over {sites} cached sites ({loc} LOC)");
    release_assert(
        sweep.as_secs_f64() < 2.0,
        &format!("resolve sweep {sweep:.2?} exceeds 2s budget"),
    );
}

#[test]
#[ignore = "perf budget: run in release, records docs/PERF-BUDGET.md"]
fn probe_round_trip_p95_under_150ms() {
    let root = corpus_dir("perf-probe");
    generate_corpus(&root);
    let child = common::spawn_daemon_ready(&root);
    let mut rtts: Vec<u128> = Vec::with_capacity(100);
    for _ in 0..100 {
        let t0 = Instant::now();
        let r = client::request_if_running(&root, &Request::Ping).expect("ping");
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
    common::shutdown_daemon(&root);
    common::wait_exit(child, "daemon");
    release_assert(p95 < 150.0, &format!("probe p95 {p95:.2}ms exceeds budget"));
}

fn hook_envelope(dir: &Path, content: &str) -> String {
    let cwd = dir.display().to_string().replace('\\', "/");
    serde_json::json!({
        "session_id": "bench", "transcript_path": "t", "cwd": cwd,
        "hook_event_name": "PreToolUse", "tool_name": "Write",
        "tool_input": {"file_path": format!("{cwd}/b.rs"), "content": content},
        "tool_use_id": "t"
    })
    .to_string()
}

/// 30 timed hook calls; asserts every call decides (or stays silent).
fn hook_samples(dir: &Path, envelope: &str, expect_decision: bool) -> Vec<u128> {
    let mut ms = Vec::with_capacity(30);
    for _ in 0..30 {
        let t0 = Instant::now();
        let out = common::run_hook(dir, &["probe", "--hook"], envelope);
        ms.push(t0.elapsed().as_millis());
        assert_eq!(!out.trim().is_empty(), expect_decision, "hook out: {out}");
    }
    ms.sort_unstable();
    ms
}

#[test]
#[ignore = "perf budget: run in release, records docs/PERF-BUDGET.md"]
fn hook_e2e_p95_under_1s() {
    let dir = corpus_dir("perf-hook");
    common::seed_sources(&dir, "deny");
    common::build_index(&dir);
    // first call lazily spawns the daemon — the cold-path number
    let clone = rust_fn(1);
    let cold = Instant::now();
    let first = common::run_hook(&dir, &["probe", "--hook"], &hook_envelope(&dir, &clone));
    let cold_ms = cold.elapsed().as_millis();
    assert!(first.contains("\"deny\""), "warm-up must decide: {first}");
    let deny = hook_samples(&dir, &hook_envelope(&dir, &clone), true);
    let clean = hook_samples(
        &dir,
        &hook_envelope(&dir, "fn fresh(n: u8) -> u8 { n / 2 }\n"),
        false,
    );
    common::shutdown_daemon(&dir);
    let p95 = deny[28];
    println!(
        "hook e2e (ce side): cold first {cold_ms} ms; deny median {} / p95 {p95} / max {} ms; \
         clean median {} / p95 {} ms",
        deny[14], deny[29], clean[14], clean[28]
    );
    release_assert(
        p95 < 1000,
        &format!("hook e2e p95 {p95}ms exceeds 1s budget"),
    );
}
