//! SessionStart health line (M3, ADR-003 A9f: degraded state is
//! surfaced, never silent). Also the session's daemon WARM-UP: the
//! lazy ping here means later PreToolUse probes hit a hot daemon.
//! Output = the additionalContext shape proven by the locally
//! installed cc-enforcer SessionStart hook. Always exits 0.

use crate::config::Config;
use crate::daemon::client;
use crate::daemon::proto::{Request, Response};
use crate::dedup::{Params, index::Index};
use serde::Deserialize;
use std::path::Path;
use std::process::ExitCode;

#[derive(Deserialize)]
struct Envelope {
    #[serde(default)]
    hook_event_name: String,
    #[serde(default)]
    cwd: String,
}

/// Entry point for `ce health --hook`. Never fails outward.
pub fn run_hook() -> ExitCode {
    let Some(env) = crate::hookio::read_envelope::<Envelope>() else {
        return ExitCode::SUCCESS;
    };
    if env.hook_event_name != "SessionStart" || env.cwd.is_empty() {
        return ExitCode::SUCCESS;
    }
    let line = status_line(&crate::hookio::project_root(&env.cwd));
    let payload = serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": line,
        }
    });
    println!("{payload}");
    ExitCode::SUCCESS
}

/// The SessionStart status line — includes the deliberate daemon
/// WARM-UP (lazy start) so the session's probes are hot.
pub fn status_line(root: &Path) -> String {
    line(root, &daemon_summary(root))
}

/// The `ce doctor` status line (plan §5.9-5): same fields, but the
/// daemon probe NEVER spawns — a diagnostic reports the pre-existing
/// state instead of creating it (attack review 2026-08-07).
pub fn doctor_line(root: &Path) -> String {
    line(root, &daemon_status(root))
}

fn line(root: &Path, daemon: &str) -> String {
    // Reported tier = what PreToolUse will actually do: the promoted
    // classes' route default (config::PROMOTED_DEFAULT) unless
    // ce.toml overrides. A config ERROR must not print
    // byte-identically to a deliberate observe (review C2): the
    // degradation names its cause here, the one line every session
    // and every `ce doctor` shows.
    let mode = Config::load(root)
        .map(|c| c.guard.tier(crate::config::PROMOTED_DEFAULT))
        .unwrap_or_else(|e| {
            let gist: String = e.chars().take(80).collect();
            format!("observe (ce.toml ERROR: {gist})")
        });
    let index = index_summary(root);
    format!(
        "[ce {} | guard: {mode} | index: {index} | daemon: {daemon}]",
        env!("CARGO_PKG_VERSION")
    )
}

fn index_summary(root: &Path) -> String {
    let db = root.join(".ce/index.db");
    if !db.exists() {
        return "absent (first dedup/probe builds it)".into();
    }
    match Index::open(&db, Params::default()).and_then(|i| i.file_count()) {
        Ok(n) => format!("{n} files"),
        Err(_) => "unreadable (degraded — deep checks off until rebuilt)".into(),
    }
}

/// (degraded, total) entries in the project's observe feed — the A9f
/// visibility counter surfaced by `ce doctor` (plan §5.9-5). Every
/// producer (guard probe, Stop audit, precommit) stamps `degraded`,
/// so this is a plain count. The total gives the lifetime frame: the
/// feed is append-only, so the degraded count alone never returns to
/// zero after one incident (attack-review finding).
pub fn degraded_runs(root: &Path) -> (usize, usize) {
    let Ok(log) = std::fs::read_to_string(root.join(".ce/observe.ndjson")) else {
        return (0, 0);
    };
    let entries: Vec<serde_json::Value> = log
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    let degraded = entries
        .iter()
        .filter(|j| j["degraded"] == serde_json::Value::Bool(true))
        .count();
    (degraded, entries.len())
}

/// Non-spawning daemon probe for `ce doctor`.
fn daemon_status(root: &Path) -> String {
    ping_line(
        |r| client::request_if_running(r, &Request::Ping),
        root,
        "not running (lazy-starts on first probe)",
    )
}

/// The warm-up ping: lazy-starts the daemon so the session's probes
/// are hot. Unreachable = DEGRADED, said out loud per A9f.
fn daemon_summary(root: &Path) -> String {
    ping_line(
        |r| client::request(r, &Request::Ping),
        root,
        "unreachable (DEGRADED: cheap checks only, guard fails open)",
    )
}

fn ping_line(ping: impl Fn(&Path) -> anyhow::Result<Response>, root: &Path, down: &str) -> String {
    let started = std::time::Instant::now();
    match ping(root) {
        Ok(Response::Pong { .. }) => {
            format!("warm ({} ms)", started.elapsed().as_millis())
        }
        _ => down.into(),
    }
}
