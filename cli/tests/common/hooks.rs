//! Hook-side test plumbing: envelope builders per the captured
//! contract, the hook runner, and observe-feed readers. Split from
//! common/mod.rs when the §4.2 step-2 helpers pushed it past the
//! 300-line budget (E01: split before exemption).
#![allow(dead_code)]

use std::path::Path;
use std::process::{Command, Stdio};

use super::shutdown_daemon;

/// PreToolUse envelope per the captured contract: Write carries
/// `content`, Edit carries `new_string`.
pub fn pretooluse_envelope(dir: &Path, tool: &str, content: &str) -> String {
    pretooluse_envelope_at(dir, "b.rs", tool, content)
}

/// Same envelope with the target path chosen by the test (budget-rule
/// coverage needs targets other than b.rs).
pub fn pretooluse_envelope_at(dir: &Path, rel: &str, tool: &str, content: &str) -> String {
    let file = dir.join(rel).display().to_string().replace('\\', "/");
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

/// Run `ce probe --hook`, shut the daemon down, assert the decision
/// tier, and hand back the reason for content asserts — the one home
/// for the decision-JSON shape every guard test would otherwise copy.
pub fn expect_decision(dir: &Path, envelope: &str, want: &str) -> String {
    let out = run_hook(dir, &["probe", "--hook"], envelope);
    shutdown_daemon(dir);
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("decision json");
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], want);
    v["hookSpecificOutput"]["permissionDecisionReason"]
        .as_str()
        .expect("reason")
        .to_string()
}

/// Stop envelope; `stop_hook_active` = the loop-prevention flag.
pub fn stop_envelope(dir: &Path, stop_hook_active: bool) -> String {
    serde_json::json!({
        "session_id": "t", "transcript_path": "t",
        "cwd": dir.display().to_string().replace('\\', "/"),
        "hook_event_name": "Stop", "stop_hook_active": stop_hook_active
    })
    .to_string()
}

/// Run a `ce` hook subcommand with the envelope piped to stdin.
/// Hooks are fail-open, so the exit must be 0; returns stdout.
/// CE_DAEMON_IDLE_SECS rides the hook so the daemon it LAZILY spawns
/// inherits the 2-minute test idle window (batch-8 salvage: an
/// assertion failure between spawn and shutdown_daemon used to leak
/// a 30-minute daemon holding the target exe against the linker).
pub fn run_hook(dir: &Path, args: &[&str], stdin: &str) -> String {
    use std::io::Write as _;
    let mut child = Command::new(env!("CARGO_BIN_EXE_ce"))
        .args(args)
        .env("CE_DAEMON_IDLE_SECS", "120")
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

/// Last line of the project's observe feed, parsed as JSON.
pub fn last_observe(dir: &Path) -> serde_json::Value {
    let log = std::fs::read_to_string(dir.join(".ce/observe.ndjson")).expect("observe log");
    serde_json::from_str(log.lines().last().expect("line")).expect("ndjson")
}

/// Run a hook expecting SILENCE and return the observe entry it
/// wrote, asserting the event discriminator and the ts_ms stamp —
/// the shared tail of every observe/degraded case.
pub fn silent_hook_observe(
    dir: &Path,
    args: &[&str],
    stdin: &str,
    event: &str,
) -> serde_json::Value {
    let out = run_hook(dir, args, stdin);
    assert!(out.trim().is_empty(), "must stay silent: {out}");
    let line = last_observe(dir);
    assert_eq!(line["event"], event, "feed discriminator");
    assert!(line["ts_ms"].as_u64().expect("ts_ms") > 0, "stamped");
    line
}
