//! `ce probe --hook` end-to-end: real binary, stdin envelope per the
//! captured contract, decisions per ce.toml [guard] mode. This is the
//! M3 acceptance shape: a T1/T2 re-write of indexed content gets
//! intercepted (deny mode), warned (warn), or silently logged
//! (observe default) — and the observe feed always grows.

use std::path::Path;

mod common;
use common::{rust_fn, shutdown_daemon, tmp};

/// Project with a.rs indexed; guard mode written to ce.toml.
fn seed_project(dir: &Path, mode: &str) {
    common::seed_sources(dir, mode);
    common::build_index(dir);
}

use common::pretooluse_envelope as envelope;

fn run_hook(dir: &Path, envelope: &str) -> String {
    common::run_hook(dir, &["probe", "--hook"], envelope)
}

#[test]
fn deny_mode_intercepts_t1_rewrite() {
    let dir = tmp("guard-deny");
    seed_project(&dir, "deny");
    let out = run_hook(&dir, &envelope(&dir, "Write", &rust_fn(1)));
    shutdown_daemon(&dir);
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("decision json");
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");
    let reason = v["hookSpecificOutput"]["permissionDecisionReason"]
        .as_str()
        .expect("reason");
    assert!(reason.contains("a.rs"), "reason names the source: {reason}");
}

#[test]
fn warn_mode_allows_with_reason() {
    let dir = tmp("guard-warn");
    seed_project(&dir, "warn");
    let out = run_hook(&dir, &envelope(&dir, "Edit", &rust_fn(2)));
    shutdown_daemon(&dir);
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("decision json");
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "allow");
}

fn silent_probe_observe(dir: &Path, content: &str) -> serde_json::Value {
    let env = envelope(dir, "Write", content);
    let line = common::silent_hook_observe(dir, &["probe", "--hook"], &env, "probe");
    shutdown_daemon(dir);
    line
}

#[test]
fn observe_mode_is_silent_but_logs() {
    let dir = tmp("guard-observe");
    seed_project(&dir, "observe");
    let line = silent_probe_observe(&dir, &rust_fn(3));
    assert_eq!(line["degraded"], false);
    assert!(line["matches"].as_u64().expect("n") >= 1);
}

/// FAIL-OPEN intake (attack-review coverage gap): garbage stdin must
/// exit 0 with no output for every hook subcommand — the shared
/// hookio::read_envelope path.
#[test]
fn malformed_envelope_fails_open_everywhere() {
    let dir = tmp("guard-garbage");
    for sub in ["probe", "audit", "health"] {
        let out = common::run_hook(&dir, &[sub, "--hook"], "{not json");
        assert!(out.trim().is_empty(), "{sub} must stay silent: {out}");
    }
}

/// A9f guard side (attack-review coverage gap): when the probe
/// cannot deliver a verdict (index corrupt under the daemon), the
/// edit passes even in deny mode (fail-open) and the observe entry
/// stamps degraded=true — mirroring the audit-side degraded test.
#[test]
fn probe_failure_is_stamped_degraded() {
    let dir = tmp("guard-degraded");
    seed_project(&dir, "deny");
    shutdown_daemon(&dir); // next probe respawns against the bad db
    common::corrupt_index(&dir);
    let line = silent_probe_observe(&dir, &rust_fn(1));
    assert_eq!(line["degraded"], true, "stamped, not silent");
}

#[test]
fn clean_content_and_foreign_tools_stay_silent() {
    let dir = tmp("guard-clean");
    seed_project(&dir, "deny");
    // unrelated content: no decision even in deny mode
    let clean = "fn fresh(n: u8) -> u8 { n / 2 }\n";
    let out = run_hook(&dir, &envelope(&dir, "Write", clean));
    assert!(out.trim().is_empty(), "clean content passes: {out}");
    // non-Write/Edit event is ignored outright
    let bash = serde_json::json!({
        "hook_event_name": "PreToolUse", "tool_name": "Bash",
        "cwd": dir.display().to_string(), "tool_input": {"command": "ls"}
    })
    .to_string();
    let out2 = run_hook(&dir, &bash);
    shutdown_daemon(&dir);
    assert!(out2.trim().is_empty(), "foreign tool ignored: {out2}");
}
