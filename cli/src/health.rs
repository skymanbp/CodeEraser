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
use std::io::Read;
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
    let mut raw = String::new();
    if std::io::stdin().read_to_string(&mut raw).is_err() {
        return ExitCode::SUCCESS;
    }
    let Ok(env) = serde_json::from_str::<Envelope>(&raw) else {
        return ExitCode::SUCCESS;
    };
    if env.hook_event_name != "SessionStart" || env.cwd.is_empty() {
        return ExitCode::SUCCESS;
    }
    let line = health_line(&PathBuf::from(&env.cwd));
    let payload = serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": line,
        }
    });
    println!("{payload}");
    ExitCode::SUCCESS
}

fn health_line(root: &Path) -> String {
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
