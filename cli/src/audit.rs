//! Stop audit v1 (M3): net LOC of the working tree vs HEAD plus
//! duplicate blocks touching the changed files. Deliberately NOT the
//! four-way edit classification — that is the M4 judgment layer
//! (plan A4). Same discipline as the PreToolUse gate: fail-open on
//! any internal failure, every run appended to .ce/observe.ndjson.
//! Stop hooks know exactly one enforcement shape (proven by the
//! locally installed cc-enslaver): top-level
//! {"decision":"block","reason":...}; only deny mode uses it.

use crate::config::Config;
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
    /// Loop-prevention flag: true when this Stop fired because a
    /// previous Stop hook already blocked once.
    #[serde(default)]
    stop_hook_active: bool,
}

/// Entry point for `ce audit --hook`. Never fails outward.
pub fn run_hook() -> ExitCode {
    let mut raw = String::new();
    if std::io::stdin().read_to_string(&mut raw).is_err() {
        return ExitCode::SUCCESS;
    }
    let Ok(env) = serde_json::from_str::<Envelope>(&raw) else {
        return ExitCode::SUCCESS;
    };
    if env.hook_event_name != "Stop" || env.stop_hook_active || env.cwd.is_empty() {
        return ExitCode::SUCCESS;
    }
    audit(&PathBuf::from(&env.cwd))
}

/// Shared head of the Stop audit and the pre-commit gate: guard
/// mode, numstat over `diff`, touched duplicates (None = degraded,
/// A9f), and the observe entry. Outer None = not a git repo — the
/// caller fails open.
type Gathered = (String, i64, Vec<String>, Option<Vec<String>>);
fn gather(root: &Path, diff: &[&str]) -> Option<Gathered> {
    let mode = Config::load(root)
        .map(|c| c.guard.mode)
        .unwrap_or_else(|_| "observe".into());
    let (net_loc, changed) = diff_args(root, diff)?;
    let dups = if changed.is_empty() {
        Some(Vec::new())
    } else {
        touched_duplicates(root, &changed)
    };
    observe_log(root, net_loc, &changed, &dups, &mode);
    Some((mode, net_loc, changed, dups))
}

fn audit(root: &Path) -> ExitCode {
    let Some((mode, net_loc, _, dups)) = gather(root, &["diff", "--numstat", "HEAD"]) else {
        return ExitCode::SUCCESS; // not a git repo / git failed: fail open
    };
    if mode == "deny"
        && let Some(dups) = dups.as_deref()
        && !dups.is_empty()
    {
        let payload = serde_json::json!({
            "decision": "block",
            "reason": reason(net_loc, dups),
        });
        println!("{payload}");
    }
    ExitCode::SUCCESS
}

/// pre-commit mode: STAGED changes only; blocks the commit (exit 1)
/// when guard mode is deny and staged files touch duplicate blocks.
/// Unlike the hooks this prints for humans — it runs in a terminal.
pub fn run_precommit(root: &Path) -> ExitCode {
    let Some((mode, net_loc, changed, dups)) = gather(root, &["diff", "--cached", "--numstat"])
    else {
        eprintln!("ce precommit: not a git repo (skipped)");
        return ExitCode::SUCCESS;
    };
    let Some(dups) = dups.as_deref() else {
        // A9f: fail open but never silently — the human sees it
        println!("ce precommit: dedup index unavailable (DEGRADED — duplicate check skipped)");
        return ExitCode::SUCCESS;
    };
    if dups.is_empty() {
        println!(
            "ce precommit: {} staged file(s), net {net_loc:+} LOC, no touched duplicates",
            changed.len()
        );
        return ExitCode::SUCCESS;
    }
    println!("{}", reason(net_loc, dups));
    if mode == "deny" {
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn diff_args(root: &Path, args: &[&str]) -> Option<(i64, Vec<String>)> {
    let out = std::process::Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let mut net = 0i64;
    let mut files = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let mut cols = line.split('\t');
        let (a, d, path) = (cols.next()?, cols.next()?, cols.next()?);
        // '-' marks binary files: count the file, skip the arithmetic
        if let (Ok(a), Ok(d)) = (a.parse::<i64>(), d.parse::<i64>()) {
            net += a - d;
        }
        files.push(path.replace('\\', "/"));
    }
    Some((net, files))
}

/// Duplicate blocks with at least one side in the changed set. v1
/// approximation of "newly added duplication" — an exact new-vs-old
/// split needs the session-start baseline (M4). None = the dedup
/// pipeline itself failed (index unavailable): DEGRADED, stamped in
/// the observe entry — never conflated with "no duplicates" (A9f).
fn touched_duplicates(root: &Path, changed: &[String]) -> Option<Vec<String>> {
    let (found, _) = crate::dedup::analyze(root, None, None, None).ok()?;
    Some(
        found
            .blocks
            .iter()
            .filter(|b| changed.contains(&b.a_file) || changed.contains(&b.b_file))
            .take(10)
            .map(|b| {
                format!(
                    "{}:{}-{} <-> {}:{}-{} ({} tokens)",
                    b.a_file, b.a_start, b.a_end, b.b_file, b.b_start, b.b_end, b.tokens
                )
            })
            .collect(),
    )
}

fn reason(net_loc: i64, dups: &[String]) -> String {
    format!(
        "ce audit: this session's edits leave {} duplicate block(s) touching \
         changed files (net {net_loc:+} LOC): {} — deduplicate before stopping.",
        dups.len(),
        dups.join("; ")
    )
}

fn observe_log(
    root: &Path,
    net_loc: i64,
    changed: &[String],
    dups: &Option<Vec<String>>,
    mode: &str,
) {
    let dir = root.join(".ce");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let epoch_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let line = serde_json::json!({
        "ts_ms": epoch_ms,
        "event": "stop_audit",
        "mode": mode,
        "net_loc": net_loc,
        "changed_files": changed.len(),
        "degraded": dups.is_none(),
        "dup_blocks": dups.as_deref().map(<[String]>::len).unwrap_or(0),
    });
    use std::io::Write as _;
    if let Ok(mut fh) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("observe.ndjson"))
    {
        let _ = writeln!(fh, "{line}");
    }
}
