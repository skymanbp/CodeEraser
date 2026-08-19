//! Stop audit v1 (M3): net LOC of the working tree vs HEAD plus
//! duplicate blocks touching the changed files — deliberately NOT the
//! four-way classification (that is M4). PreToolUse-gate discipline:
//! fail-open on any internal failure, every run appended to
//! .ce/observe.ndjson. Stop hooks know exactly one enforcement shape
//! (proven by the locally installed cc-enforcer): top-level
//! {"decision":"block","reason":...}; only deny mode uses it.
//! WHAT changed lives in changes.rs — a mismatch between git's path
//! spelling and ce's is a silent gate bypass, not an error.

mod changes;

use crate::config::Config;
use serde::Deserialize;
use std::path::Path;
use std::process::ExitCode;

#[derive(Deserialize)]
struct Envelope {
    #[serde(default)]
    hook_event_name: String,
    #[serde(default)]
    cwd: String,
    /// Stamped on every hook event; reaches the observe feed so the
    /// M4 evaluation set can be partitioned by session (D2-1/D2-2).
    #[serde(default)]
    session_id: String,
    /// Loop guard: true when a previous Stop hook already blocked.
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
    audit(&crate::hookio::project_root(&env.cwd), &env.session_id)
}

/// Shared head of the Stop audit and the pre-commit gate: guard mode,
/// numstat over `diff`, touched duplicates (None = degraded, A9f) and
/// the `event`-stamped observe entry (precommit must not masquerade
/// as a Stop audit). Outer None = git could not answer: fail open.
type Gathered = (String, i64, Vec<String>, Option<Vec<String>>);
fn gather(
    root: &Path,
    diff_tail: &[&str],
    event: &str,
    session: Option<&str>,
    fourclass: Option<serde_json::Value>,
) -> Option<Gathered> {
    // ONE load: tier rendering and the exclusion list both need it,
    // and a broken ce.toml must name itself (config::tier_of).
    let loaded = Config::load(root);
    // The audit class is not §4.2-promoted: unset mode stays observe.
    let mode = crate::config::tier_of(&loaded, "observe");
    let (mut net_loc, mut changed) = changes::diff(root, diff_tail)?;
    if event == "stop_audit" {
        // §4.2 promises the Stop audit covers Bash writes, but `git
        // diff` cannot see a brand-new untracked file — the stop path
        // merges them in; precommit stays staged-only by design.
        let excludes = loaded.map(|c| c.exclude).unwrap_or_default();
        if let Some((n, mut files)) = changes::untracked(root, &excludes) {
            net_loc += n;
            changed.append(&mut files);
        }
    }
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
            fourclass,
        },
    );
    Some((mode, net_loc, changed, dups))
}

fn audit(root: &Path, session: &str) -> ExitCode {
    // Not a git repo: nothing to audit, and no `.ce/` written here.
    let Some(base) = changes::base_rev(root) else {
        return ExitCode::SUCCESS;
    };
    let Some((mode, net_loc, _, dups)) = gather(
        root,
        &[base.as_str()],
        "stop_audit",
        Some(session),
        Some(fourclass_report(root)),
    ) else {
        return degraded_stop(root, session);
    };
    if mode == "deny"
        && let Some(dups) = dups.as_deref()
        && !dups.is_empty()
    {
        let payload = serde_json::json!({
            "decision": "block",
            // §4.4 B4: the Stop summary rides its own 400-token budget
            "reason": crate::hookio::clip(
                &reason(net_loc, dups),
                crate::hookio::STOP_BUDGET_TOKENS,
            ),
        });
        println!("{payload}");
    }
    ExitCode::SUCCESS
}

/// git resolved the base but not the diff: a real degradation the
/// feed has to SAY. A Stop that writes nothing reads to the M4
/// ledger exactly like a clean one — and that ledger is the stated
/// gate for promoting this audit to deny.
fn degraded_stop(root: &Path, session: &str) -> ExitCode {
    let mode = crate::config::tier_of(&Config::load(root), "observe");
    observe_log(
        root,
        AuditEvent {
            event: "stop_audit",
            mode: &mode,
            session: Some(session),
            net_loc: 0,
            changed: 0,
            dups: None, // stamps degraded: true
            fourclass: None,
        },
    );
    ExitCode::SUCCESS
}

/// M4 four-class summary of the session's working-tree diff via the
/// daemon-owned ce-core link. INFORMATIONAL only (R-L2-4: no deny
/// path may lean on it); `request_if_running` because a Stop must not
/// pay a daemon spawn — cold = a visible degraded field, not latency.
fn fourclass_report(root: &Path) -> serde_json::Value {
    use crate::daemon::{client, proto::Request, proto::Response};
    let Some(pairs) = crate::fourclass::session::head_pairs(root) else {
        return serde_json::json!({"degraded": "no_git"});
    };
    if pairs.is_empty() {
        return serde_json::json!({"degraded": null, "relocations": []});
    }
    match client::request_if_running(root, &Request::FourClass { pairs }) {
        Ok(Response::FourClassReport { report }) => report,
        _ => serde_json::json!({"degraded": "daemon_unavailable"}),
    }
}

/// pre-commit mode: STAGED changes only; blocks the commit (exit 1)
/// when guard mode is deny and staged files touch duplicate blocks.
/// Unlike the hooks this prints for humans — it runs in a terminal.
pub fn run_precommit(root: &Path) -> ExitCode {
    // session = None honestly: `ce precommit` runs in a terminal, no
    // session owns it — the M4 sampler excludes non-session events.
    // `--cached` needs no base rev: git compares the index against an
    // unborn HEAD without complaint, unlike the Stop leg.
    let Some((mode, net_loc, changed, dups)) = gather(
        root,
        &["--cached"],
        "precommit",
        None,
        None, // four-class is a Stop concern; precommit stays v1
    ) else {
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

/// Duplicate blocks with at least one side in the changed set — the
/// v1 approximation of "newly added duplication" (exact split = M4).
/// None = the dedup pipeline itself failed: DEGRADED, stamped in the
/// observe entry, never conflated with "no duplicates" (A9f).
fn touched_duplicates(root: &Path, changed: &[String]) -> Option<Vec<String>> {
    let (found, _) = crate::dedup::analyze(root, None, None, None).ok()?;
    // a set, not a Vec scan: blocks × changed was O(B·C) string
    // compares on every Stop (~50 M on a large index)
    let touched: std::collections::HashSet<&str> = changed.iter().map(String::as_str).collect();
    Some(
        found
            .blocks
            .iter()
            .filter(|b| touched.contains(b.a_file.as_str()) || touched.contains(b.b_file.as_str()))
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
/// positional parameters: the old signature was already past the
/// scan's fn-params limit of 5.
struct AuditEvent<'a> {
    event: &'a str,
    mode: &'a str,
    /// None for `ce precommit` — see the call site.
    session: Option<&'a str>,
    net_loc: i64,
    changed: usize,
    /// None = the dedup pipeline failed (A9f degraded), never
    /// flattened into "zero duplicates".
    dups: Option<usize>,
    /// M4 informational four-class report (Stop only; absent on the
    /// precommit path).
    fourclass: Option<serde_json::Value>,
}

fn observe_log(root: &Path, ev: AuditEvent) {
    let mut line = serde_json::json!({
        "event": ev.event,
        "mode": ev.mode,
        "net_loc": ev.net_loc,
        "changed_files": ev.changed,
        "degraded": ev.dups.is_none(),
        "dup_blocks": ev.dups.unwrap_or(0),
    });
    if let Some(fc) = ev.fourclass {
        line["fourclass"] = fc;
    }
    crate::hookio::observe_append(root, ev.session, line);
}
