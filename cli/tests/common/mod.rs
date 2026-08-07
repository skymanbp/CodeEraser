//! Shared integration-test helpers. Extracted after the FPR
//! self-replay arbitration flagged 33 copies of these very functions
//! across `cli/tests/*.rs` — the tool catching its author's own
//! stacking (docs/FPR-REPLAY.md).
//!
//! Each test binary compiles its own copy of this module and uses a
//! subset of it, so unused items here are expected — that is the why
//! for the allow below.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Fresh per-test dir under the cargo target tmpdir (wiped if present).
pub fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
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

/// Run git in `dir` with a throwaway identity; panic on failure.
pub fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=t@t", "-c", "user.name=t"])
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

/// Write `a.rs` (the T2 seed) plus a ce.toml pinning the guard mode.
pub fn seed_sources(dir: &Path, mode: &str) {
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("ce.toml"), format!("[guard]\nmode = \"{mode}\"\n")).expect("ce.toml");
}

/// Build the project index by running the real `ce dedup .` in `dir`.
pub fn build_index(dir: &Path) {
    let out = Command::new(env!("CARGO_BIN_EXE_ce"))
        .args(["dedup", "."])
        .current_dir(dir)
        .output()
        .expect("seed index");
    assert!(out.status.success(), "seed dedup failed");
}

/// Run a `ce` hook subcommand with the envelope piped to stdin.
/// Hooks are fail-open, so the exit must be 0; returns stdout.
pub fn run_hook(dir: &Path, args: &[&str], stdin: &str) -> String {
    use std::io::Write as _;
    let mut child = Command::new(env!("CARGO_BIN_EXE_ce"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn hook");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write envelope");
    let out = child.wait_with_output().expect("wait");
    assert!(out.status.success(), "hook must always exit 0 (fail-open)");
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Spawn the real `ce daemon` for `root` and wait until it answers a
/// ping. Spawning ourselves matters: the client's lazy start would
/// respawn current_exe(), which inside a test harness is the TEST
/// binary, not `ce`.
pub fn spawn_daemon_ready(root: &Path) -> Child {
    use codeeraser::daemon::{client, proto::Request};
    let mut child = Command::new(env!("CARGO_BIN_EXE_ce"))
        .arg("daemon")
        .arg(root)
        .env("CE_DAEMON_IDLE_SECS", "120")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ce daemon");
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if client::request(root, &Request::Ping).is_ok() {
            return child;
        }
    }
    let _ = child.kill();
    let _ = child.wait(); // reap — no zombie on the panic path
    panic!("daemon never came up");
}

/// Last line of the project's observe feed, parsed as JSON.
pub fn last_observe(dir: &Path) -> serde_json::Value {
    let log = std::fs::read_to_string(dir.join(".ce/observe.ndjson")).expect("observe log");
    serde_json::from_str(log.lines().last().expect("line")).expect("ndjson")
}

/// Ask the daemon for `dir` to shut down (ignore errors — may be gone).
pub fn shutdown_daemon(dir: &Path) {
    use codeeraser::daemon::{client, proto::Request};
    let _ = client::request(dir, &Request::Shutdown);
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
