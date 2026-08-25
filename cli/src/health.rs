//! SessionStart health line (M3, ADR-003 A9f: degraded state is
//! surfaced, never silent). Also the session's daemon WARM-UP: the
//! lazy ping here means later PreToolUse probes hit a hot daemon.
//! Output = the additionalContext shape proven by the locally
//! installed cc-enforcer SessionStart hook. Always exits 0.

use crate::config::Config;
use crate::daemon::client;
use crate::daemon::proto::{Request, Response};
use crate::dedup::{Params, index};
use crate::i18n;
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
///
/// `ce doctor`'s own line is NOT built here any more: it renders the
/// doctor DOCUMENT (health::doctor::console), so the two faces of one
/// diagnostic share a measurement rather than a format string. The
/// `doctor_line` twin that used to sit beside this one lost its last
/// caller when that landed and was carried dead until plan v2.15 —
/// invisible to the deadcode gate, which cannot see through `pub`.
pub fn status_line(root: &Path) -> String {
    // Reported tier = what PreToolUse will actually do: the promoted
    // classes' route default (config::PROMOTED_DEFAULT) unless
    // ce.toml overrides. A config ERROR must not print
    // byte-identically to a deliberate observe (review C2): the
    // degradation names its cause here, the one line every session
    // shows — through config::tier_of, the ONE renderer the Stop
    // audit now shares.
    let mode = crate::config::tier_of(&Config::load(root), crate::config::PROMOTED_DEFAULT);
    i18n::line(
        "[ce {} | guard: {} | index: {} | daemon: {}]",
        "〔ce {} | 守卫：{} | 索引：{} | daemon：{}〕",
        &[
            &env!("CARGO_PKG_VERSION"),
            &mode,
            &index_words(index_fact(root)),
            &daemon_words(daemon_warmup(root)),
        ],
    )
}

/// The index's state as a CODE plus the count it measured (plan
/// v2.15). Frozen positions: 0 absent, 1 fresh, 2 stale, 3
/// unreadable. The WORDS used to be the fact — `index_summary`
/// returned English prose, and the doctor document carried that prose
/// to the GUI and the MCP surface, where no lookup switch reaches:
/// i18n.rs declares report JSON the machine face and never translates
/// it. A code crosses; each face owns its sentence.
pub(crate) fn index_fact(root: &Path) -> (i64, Option<i64>) {
    let db = root.join(".ce/index.db");
    if !db.exists() {
        return (0, None);
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
        Ok((n, true)) => (1, Some(n)),
        Ok((n, false)) => (2, Some(n)),
        Err(_) => (3, None),
    }
}

/// One face's rendering of [`index_fact`] — the CLI's, bilingual.
/// The GUI keeps its own table over the same codes.
pub(crate) fn index_words((state, files): (i64, Option<i64>)) -> String {
    i18n::coded(
        state,
        &[
            (
                "absent (first dedup/probe builds it)",
                "缺失（首次 dedup/probe 会建立）",
            ),
            ("{} files", "{} 个文件"),
            (
                "{} files (stale — next dedup rebuilds it)",
                "{} 个文件（陈旧 — 下次 dedup 重建）",
            ),
            (
                "unreadable (degraded — deep checks off until rebuilt)",
                "不可读（已降级 — 重建前深检关闭）",
            ),
        ],
        &[&files.unwrap_or_default()],
    )
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

/// Non-spawning daemon probe for `ce doctor` — a diagnostic reports
/// the pre-existing state instead of creating it (attack review
/// 2026-08-07). Down here means 1: nothing was running, which is not
/// a fault.
pub(crate) fn daemon_fact(root: &Path) -> (i64, Option<u128>) {
    probe_fact(|r| client::request_if_running(r, &Request::Ping), root, 1)
}

/// The warm-up ping: lazy-starts the daemon so the session's probes
/// are hot. Down here means 2 — this probe TRIED to start it, so a
/// failure is a real degradation, said out loud per A9f. The two down
/// codes are why one shared prober takes the code as an argument
/// rather than deriving it.
fn daemon_warmup(root: &Path) -> (i64, Option<u128>) {
    probe_fact(|r| client::request(r, &Request::Ping), root, 2)
}

fn probe_fact(
    ping: impl Fn(&Path) -> anyhow::Result<Response>,
    root: &Path,
    down: i64,
) -> (i64, Option<u128>) {
    let started = std::time::Instant::now();
    match ping(root) {
        Ok(Response::Pong { .. }) => (0, Some(started.elapsed().as_millis())),
        _ => (down, None),
    }
}

/// One face's rendering of a daemon fact — codes 0 warm, 1 not
/// running, 2 unreachable (frozen positions, plan v2.15).
pub(crate) fn daemon_words((state, ms): (i64, Option<u128>)) -> String {
    i18n::coded(
        state,
        &[
            ("warm ({} ms)", "已预热（{} 毫秒）"),
            (
                "not running (lazy-starts on first probe)",
                "未运行（首次 probe 时惰性启动）",
            ),
            (
                "unreachable (DEGRADED: cheap checks only, guard fails open)",
                "不可达（已降级：仅剩廉价检查，守卫失败开放）",
            ),
        ],
        &[&ms.unwrap_or_default()],
    )
}

/// The doctor's facts as a DOCUMENT (K round step 6). Same
/// measurement as the line above, one file down, so the console
/// prose and the GUI screen can never disagree about a diagnostic.
pub mod doctor;

#[cfg(test)]
#[path = "health_tests.rs"]
mod tests;
