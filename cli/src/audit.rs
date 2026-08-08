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
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Deserialize)]
struct Envelope {
    #[serde(default)]
    hook_event_name: String,
    #[serde(default)]
    cwd: String,
    /// Claude Code stamps this on every hook event; it reaches the
    /// observe feed (schema 0.2.0) so the M4 evaluation set can be
    /// partitioned by session (D2-2 count, D2-1 purity).
    #[serde(default)]
    session_id: String,
    /// Loop-prevention flag: true when this Stop fired because a
    /// previous Stop hook already blocked once.
    #[serde(default)]
    stop_hook_active: bool,
}

/// Entry point for `ce audit --hook`. Never fails outward.
pub fn run_hook() -> ExitCode {
    let Some(env) = crate::hookio::read_envelope::<Envelope>() else {
        return ExitCode::SUCCESS;
    };
    if env.hook_event_name != "Stop" || env.stop_hook_active || env.cwd.is_empty() {
        return ExitCode::SUCCESS;
    }
    audit(&PathBuf::from(&env.cwd), &env.session_id)
}

/// Shared head of the Stop audit and the pre-commit gate: guard
/// mode, numstat over `diff`, touched duplicates (None = degraded,
/// A9f), and the observe entry stamped with `event` (the feed's
/// discriminator — precommit runs must not masquerade as Stop
/// audits, attack-review finding). Outer None = not a git repo — the
/// caller fails open.
type Gathered = (String, i64, Vec<String>, Option<Vec<String>>);
fn gather(root: &Path, diff: &[&str], event: &str, session: Option<&str>) -> Option<Gathered> {
    let mode = Config::load(root)
        .map(|c| c.guard.mode)
        .unwrap_or_else(|_| "observe".into());
    let (net_loc, changed) = diff_args(root, diff)?;
    let dups = if changed.is_empty() {
        Some(Vec::new())
    } else {
        touched_duplicates(root, &changed)
    };
    observe_log(
        root,
        AuditEvent {
            event,
            mode: &mode,
            session,
            net_loc,
            changed: changed.len(),
            dups: dups.as_deref().map(<[String]>::len),
        },
    );
    Some((mode, net_loc, changed, dups))
}

fn audit(root: &Path, session: &str) -> ExitCode {
    let Some((mode, net_loc, _, dups)) = gather(
        root,
        &["diff", "--numstat", "HEAD"],
        "stop_audit",
        Some(session),
    ) else {
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
    // session = None, and that is the honest value: `ce precommit`
    // runs in a terminal, not as a hook, so no session owns it. The
    // feed records null rather than inventing one, which is what lets
    // the M4 sampler exclude non-session events instead of guessing.
    let Some((mode, net_loc, changed, dups)) =
        gather(root, &["diff", "--cached", "--numstat"], "precommit", None)
    else {
        eprintln!("ce precommit: not a git repo (skipped)");
        return ExitCode::SUCCESS;
    };
    let Some(dups) = dups.as_deref() else {
        // A9f: fail open but never silently — the human still gets
        // the staged summary the healthy path prints
        println!(
            "ce precommit: {} staged file(s), net {net_loc:+} LOC — dedup index \
             unavailable (DEGRADED: duplicate check skipped)",
            changed.len()
        );
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

/// One Stop-audit / precommit observe entry. A struct rather than
/// positional parameters: the old signature was already at six, one
/// past the scan's fn-params limit of 5, and session identity would
/// have made it seven.
struct AuditEvent<'a> {
    event: &'a str,
    mode: &'a str,
    /// None for `ce precommit` — see the call site.
    session: Option<&'a str>,
    net_loc: i64,
    changed: usize,
    /// None = the dedup pipeline itself failed (A9f degraded), which
    /// must never be flattened into "zero duplicates".
    dups: Option<usize>,
}

fn observe_log(root: &Path, ev: AuditEvent) {
    crate::hookio::observe_append(
        root,
        ev.session,
        serde_json::json!({
            "event": ev.event,
            "mode": ev.mode,
            "net_loc": ev.net_loc,
            "changed_files": ev.changed,
            "degraded": ev.dups.is_none(),
            "dup_blocks": ev.dups.unwrap_or(0),
        }),
    );
}
