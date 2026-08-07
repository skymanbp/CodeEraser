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

fn envelope(dir: &Path, tool: &str, content: &str) -> String {
    let file = dir.join("b.rs").display().to_string().replace('\\', "/");
    let cwd = dir.display().to_string().replace('\\', "/");
    let input = if tool == "Write" {
        serde_json::json!({"file_path": file, "content": content})
    } else {
        serde_json::json!({"file_path": file, "old_string": "x", "new_string": content, "replace_all": false})
    };
    serde_json::json!({
        "session_id": "t", "transcript_path": "t", "cwd": cwd,
        "hook_event_name": "PreToolUse", "tool_name": tool,
        "tool_input": input, "tool_use_id": "t"
    })
    .to_string()
}

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

#[test]
fn observe_mode_is_silent_but_logs() {
    let dir = tmp("guard-observe");
    seed_project(&dir, "observe");
    let out = run_hook(&dir, &envelope(&dir, "Write", &rust_fn(3)));
    shutdown_daemon(&dir);
    assert!(out.trim().is_empty(), "observe emits nothing: {out}");
    let line = common::last_observe(&dir);
    assert_eq!(line["degraded"], false);
    assert!(line["matches"].as_u64().expect("n") >= 1);
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
