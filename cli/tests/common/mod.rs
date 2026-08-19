//! Shared integration-test helpers. Extracted after the FPR
//! self-replay arbitration flagged 33 copies of these very functions
//! across `cli/tests/*.rs` — the tool catching its author's own
//! stacking (docs/FPR-REPLAY.md).
//!
//! Each test binary compiles its own copy of this module and uses a
//! subset of it, so unused items here are expected — that is the why
//! for the allow below.
#![allow(dead_code)]
// The hooks re-export is likewise unused in binaries that never run
// hooks — same subset story as dead_code above.
#![allow(unused_imports)]

pub mod gates;
pub mod gitio;
pub mod hooks;
pub mod ladder;
pub use gates::*;
pub use gitio::*;
pub use hooks::*;
pub use ladder::*;

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Fresh per-test dir under the cargo target tmpdir (wiped if
/// present). Carries an empty `.git` anchor: hookio::project_root
/// ascends to the nearest ce.toml/.git, and an anchorless fixture
/// under target/tmp would ascend into the REAL repo (three guard
/// batteries did exactly that when the anchoring landed). Real hook
/// cwds are never anchorless voids; the walker skips hidden dirs,
/// so scans see nothing.
pub fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".git")).expect("mkdir");
    dir
}

/// ~60 normalized tokens; `seed` renames identifiers and changes
/// literals so pairs of outputs are T2 (not T1) clones clearing t=50.
pub fn rust_fn(seed: u32) -> String {
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

/// Parse `src` with the language's tree-sitter grammar — the shared
/// head of every metric/token harness (metrics, divergence, sonar,
/// dedup_core each kept a copy before the self-ratchet flagged it).
pub fn parse(lang: codeeraser::scan::lang::Lang, src: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang.grammar().expect("grammar"))
        .expect("set_language");
    parser.parse(src, None).expect("parse")
}

/// One measured unit — every scan-metric battery's assertion currency
/// (metrics / divergence-stances / sonar-whitepaper each kept its own
/// struct + measure loop until the self-ratchet flagged the trio).
pub struct MeasuredUnit {
    pub name: String,
    pub lines: usize,
    pub params: usize,
    pub cc: u32,
    pub coc: u32,
    pub nesting: u32,
}

/// Every extracted unit of `src` with all five metrics.
pub fn measure_units(lang: codeeraser::scan::lang::Lang, src: &str) -> Vec<MeasuredUnit> {
    use codeeraser::scan::{functions, metrics, spec};
    let sp = spec::spec(lang);
    let tree = parse(lang, src);
    functions::extract(tree.root_node(), src.as_bytes(), sp)
        .into_iter()
        .map(|u| {
            let cog = metrics::cognitive::measure(u.node, src.as_bytes(), sp);
            MeasuredUnit {
                name: u.name,
                lines: u.end_line - u.start_line + 1,
                params: u.params,
                cc: metrics::cyclo::measure(u.node, src.as_bytes(), sp),
                coc: cog.score,
                nesting: cog.max_nesting,
            }
        })
        .collect()
}

/// One table row of a metric battery: source, expected unit count and
/// the (unit index, metric, expected, why) checks. The why strings
/// carry the whitepaper citations / stance records — the table IS the
/// register.
pub struct MetricCase {
    pub lang: codeeraser::scan::lang::Lang,
    pub src: &'static str,
    pub fns: usize,
    pub checks: &'static [(usize, &'static str, u32, &'static str)],
}

/// Run a metric battery table — ONE assertion loop for the three
/// scan-metric test files.
pub fn run_metric_cases(cases: &[MetricCase]) {
    for c in cases {
        let m = measure_units(c.lang, c.src);
        assert_eq!(m.len(), c.fns, "unit count for:\n{}", c.src);
        for &(i, key, want, why) in c.checks {
            let got = match key {
                "cc" => m[i].cc,
                "coc" => m[i].coc,
                "nesting" => m[i].nesting,
                "lines" => m[i].lines as u32,
                "params" => m[i].params as u32,
                other => panic!("unknown metric key {other}"),
            };
            assert_eq!(got, want, "{key}[{i}]: {why}");
        }
    }
}

/// Write `a.rs` (the T2 seed) plus a ce.toml pinning the guard mode.
pub fn seed_sources(dir: &Path, mode: &str) {
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("ce.toml"), format!("[guard]\nmode = \"{mode}\"\n")).expect("ce.toml");
}

/// a.rs + b.rs forming a T2 clone pair that clears t=50.
pub fn seed_clone_pair(dir: &Path) {
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs (T2 clone)");
}

/// Stage everything present and commit it — the one commit stanza
/// (the P4 ratchet caught trend_rebuild's seed re-growing this trio
/// of git calls token for token).
pub fn commit_all(dir: &Path, msg: &str) {
    git(dir, &["add", "."]);
    git(dir, &["commit", "-qm", msg]);
}

/// Git repo with the T2 seed committed and b.rs (the clone) staged
/// but uncommitted — the audit/precommit fixture shape.
pub fn seed_git_clone_repo(dir: &Path, mode: &str) {
    seed_sources(dir, mode);
    git(dir, &["init", "-q"]);
    commit_all(dir, "seed");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs (uncommitted clone)");
    git(dir, &["add", "b.rs"]); // numstat vs HEAD sees staged new files
}

/// Run the library dedup pipeline on `dir`, assert the pairwise-block
/// and group counts every caller checks anyway, and return the result
/// for the distinctive per-test assertions. Counts live HERE so the
/// per-test stanzas stay below the winnowing guarantee t — the
/// ratchet caught three copies of that pattern (dedup_groups.rs).
pub fn analyze(dir: &Path, blocks: usize, groups: usize) -> codeeraser::dedup::pairs::Blocks {
    let (found, _) = codeeraser::dedup::analyze(dir, None, None, None).expect("analyze");
    assert_eq!(found.blocks.len(), blocks, "pairwise block count");
    assert_eq!(found.groups.len(), groups, "group count");
    found
}

/// Path of a golden fixture under contracts/fixtures.
pub fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .join("contracts/fixtures")
        .join(name)
}

/// CE_BLESS=1 regenerates the golden; otherwise byte-compare
/// (CRLF-normalized). Exactly "1": any-value is_ok() would let
/// CE_BLESS=0 or an empty var silently bless-and-pass
/// (attack-review finding).
pub fn assert_matches_golden(json: &str, path: &Path) {
    if std::env::var("CE_BLESS").as_deref() == Ok("1") {
        std::fs::create_dir_all(path.parent().expect("golden dir")).expect("mkdir");
        std::fs::write(path, format!("{json}\n")).expect("bless golden");
        return;
    }
    let golden = std::fs::read_to_string(path)
        .unwrap_or_else(|e| {
            panic!(
                "missing golden {} ({e}); CE_BLESS=1 to create",
                path.display()
            )
        })
        .replace("\r\n", "\n");
    assert_eq!(
        json.trim_end(),
        golden.trim_end(),
        "report shape drifted — bump the schema id and re-bless deliberately"
    );
}

/// Corrupt the project's dedup index so every deep check degrades —
/// the A9f test fixture (audit, precommit, and guard variants).
pub fn corrupt_index(dir: &Path) {
    std::fs::create_dir_all(dir.join(".ce")).expect(".ce");
    std::fs::write(dir.join(".ce/index.db"), b"not a sqlite database").expect("corrupt db");
}

/// Run the real `ce` binary with `args` in `dir`; the caller asserts
/// on success or failure (gate tests need both directions).
/// run_expect / write_all (the success-direction and multi-write
/// stanzas) live in gates.rs — mod.rs sits at its own 300-line gate.
pub fn run_ce(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ce"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run ce")
}

/// Build the project index by running the real `ce dedup .` in `dir`.
pub fn build_index(dir: &Path) {
    let out = run_ce(dir, &["dedup", "."]);
    assert!(out.status.success(), "seed dedup failed");
}

/// Spawn the real `ce daemon` for `root` and wait until it answers a
/// ping. Tests must NEVER go through the lazy-spawning
/// `client::request`: it respawns current_exe(), which inside a test
/// harness is the TEST binary — libtest then treats the `daemon` arg
/// as a name filter and runs `*_daemon` tests NESTED, wiping shared
/// tmp dirs and double-serving sockets mid-test (the Windows-CI
/// cold-start flake class). `request_if_running` never spawns; a
/// connect failure is a loud test failure, not silent process spray.
/// Daemon stderr is inherited so CI logs show its cold-start lines.
pub fn spawn_daemon_ready(root: &Path) -> Child {
    use codeeraser::daemon::{client, proto::Request};
    let mut child = Command::new(env!("CARGO_BIN_EXE_ce"))
        .arg("daemon")
        .arg(root)
        .env("CE_DAEMON_IDLE_SECS", "120")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn ce daemon");
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if client::request_if_running(root, &Request::Ping).is_ok() {
            return child;
        }
    }
    let _ = child.kill();
    let _ = child.wait(); // reap — no zombie on the panic path
    panic!("daemon never came up");
}

/// Ask the daemon for `dir` to shut down (ignore errors — may be
/// gone). Never the lazy-spawning path: spawning a daemon in order to
/// shut it down would be absurd, and in a test harness it sprays
/// nested test-binary processes (see spawn_daemon_ready).
pub fn shutdown_daemon(dir: &Path) {
    use codeeraser::daemon::{client, proto::Request};
    let _ = client::request_if_running(dir, &Request::Shutdown);
}

/// Clean daemon shutdown (Bye asserted) then reaped exit — the
/// shared tail of every daemon e2e case.
pub fn shutdown_and_wait(root: &Path, child: Child, what: &str) {
    use codeeraser::daemon::{client, proto::Request, proto::Response};
    match client::request_if_running(root, &Request::Shutdown).expect("shutdown") {
        Response::Bye => {}
        other => panic!("expected bye, got {other:?}"),
    }
    wait_exit(child, what);
}

/// Wait ~5s for `child` to exit; kill it and panic on timeout.
pub fn wait_exit(mut child: Child, what: &str) {
    for _ in 0..50 {
        if let Ok(Some(_)) = child.try_wait() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = child.kill();
    let _ = child.wait(); // reap — no zombie on the panic path
    panic!("{what} did not exit");
}
