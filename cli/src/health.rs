//! SessionStart health line (M3, ADR-003 A9f: degraded state is
//! surfaced, never silent). Also the session's daemon WARM-UP: the
//! lazy ping here means later PreToolUse probes hit a hot daemon.
//! Output = the additionalContext shape proven by the locally
//! installed cc-enslaver SessionStart hook. Always exits 0.

use crate::config::Config;
use crate::daemon::client;
use crate::daemon::proto::{Request, Response};
use crate::dedup::{Params, index::Index};
use serde::Deserialize;
use std::path::{Path, PathBuf};
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
    let line = status_line(&PathBuf::from(&env.cwd));
    let payload = serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": line,
        }
    });
    println!("{payload}");
    ExitCode::SUCCESS
}

/// The one-line project status — shared by the SessionStart hook and
/// `ce doctor` (plan §5.9-5: daemon health, index freshness).
pub fn status_line(root: &Path) -> String {
    let mode = Config::load(root)
        .map(|c| c.guard.mode)
        .unwrap_or_else(|_| "observe".into());
    let index = index_summary(root);
    let daemon = daemon_summary(root);
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

/// Degraded runs recorded in the project's observe feed — the A9f
/// visibility counter surfaced by `ce doctor` (plan §5.9-5). Both the
/// PreToolUse guard and the Stop audit stamp `degraded` on every
/// entry, so this is a plain count, not a heuristic.
pub fn degraded_runs(root: &Path) -> usize {
    let Ok(log) = std::fs::read_to_string(root.join(".ce/observe.ndjson")) else {
        return 0;
    };
    log.lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter(|j| j["degraded"] == serde_json::Value::Bool(true))
        .count()
}

/// The warm-up ping: lazy-starts the daemon so the session's probes
/// are hot. Unreachable = DEGRADED, said out loud per A9f.
fn daemon_summary(root: &Path) -> String {
    let started = std::time::Instant::now();
    match client::request(root, &Request::Ping) {
        Ok(Response::Pong { .. }) => {
            format!("warm ({} ms)", started.elapsed().as_millis())
        }
        _ => "unreachable (DEGRADED: cheap checks only, guard fails open)".into(),
    }
}
