//! SessionStart health line (M3, ADR-003 A9f: degraded state is
//! surfaced, never silent). Also the session's daemon WARM-UP: the
//! lazy ping here means later PreToolUse probes hit a hot daemon.
//! Output = the additionalContext shape proven by the locally
//! installed cc-enforcer SessionStart hook. Always exits 0.

use crate::config::Config;
use crate::daemon::client;
use crate::daemon::proto::{Request, Response};
use crate::dedup::{Params, index};
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
    let gate =
        crate::hookio::gated_envelope("SessionStart", |e: &Envelope| (&e.hook_event_name, &e.cwd));
    let Some((_env, root)) = gate else {
        return ExitCode::SUCCESS;
    };
    let line = status_line(&root);
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
    // and every `ce doctor` shows — through config::tier_of, the ONE
    // renderer the Stop audit now shares.
    let mode = crate::config::tier_of(&Config::load(root), crate::config::PROMOTED_DEFAULT);
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
    // index::peek, never Index::open: the latter runs
    // schema::ensure_cache_key, which WIPES the database on any
    // revision mismatch — so this status line (and `ce doctor`, and
    // every SessionStart) rebuilt the index it claimed to be merely
    // reporting on, and two `ce` binaries of different revisions on
    // one tree destroyed each other's work at every session start
    // while both printed a healthy count. A diagnostic must not
    // mutate the state it reports: the rebuild is the next dedup's,
    // and staleness is said out loud here instead (A9f).
    match index::peek(&db, Params::default()) {
        Ok((n, true)) => format!("{n} files"),
        Ok((n, false)) => format!("{n} files (stale — next dedup rebuilds it)"),
        Err(_) => "unreadable (degraded — deep checks off until rebuilt)".into(),
    }
}

/// A feed line is degraded when its OWN bit says so, or when any
/// producer nested inside it does. A Stop audit's top-level bit is
/// the DEDUP leg's (`ev.dups.is_none()`, audit.rs) while the L2
/// degradation lands a level down as `fourclass.degraded` — so the
/// one the A9f promise is most about, ce-core unreachable, was the
/// one `ce doctor` could not see. Scanning nested objects instead of
/// naming `fourclass` covers the next producer too: choosing which
/// keys to look at is how the hole opened.
fn line_degraded(line: &serde_json::Value) -> bool {
    let flag = |v: &serde_json::Value| v["degraded"] == serde_json::Value::Bool(true);
    flag(line)
        || line
            .as_object()
            .is_some_and(|o| o.values().any(|v| v.is_object() && flag(v)))
}

/// (degraded, total) entries in the project's observe feed — the A9f
/// visibility counter surfaced by `ce doctor` (plan §5.9-5). The
/// total gives the lifetime frame: the feed is append-only, so the
/// degraded count alone never returns to zero after one incident
/// (attack-review finding).
pub fn degraded_runs(root: &Path) -> (usize, usize) {
    let Ok(log) = std::fs::read_to_string(root.join(".ce/observe.ndjson")) else {
        return (0, 0);
    };
    let entries: Vec<serde_json::Value> = log
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    let degraded = entries.iter().filter(|j| line_degraded(j)).count();
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

#[cfg(test)]
#[path = "health_tests.rs"]
mod tests;
