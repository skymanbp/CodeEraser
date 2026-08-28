//! Stop audit (M3 shape, judged over audit/1 since 2.24.0): net LOC
//! of the working tree vs HEAD plus duplicate blocks touching the
//! changed files — deliberately NOT the four-way classification
//! (that is M4); the conviction and the zero-tolerance threshold are
//! the core's (verdict.rs). PreToolUse-gate discipline:
//! fail-open on any internal failure, every run appended to
//! .ce/observe.ndjson. Stop hooks know exactly one enforcement shape
//! (proven by the locally installed cc-enforcer): top-level
//! {"decision":"block","reason":...}; only deny mode uses it. WHAT
//! changed lives in changes.rs (git's paths in ce's own vocabulary).

mod changes;
mod observe;
mod verdict;

use crate::config::Config;
use observe::{AuditEvent, observe_log};
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
    let Some((env, root)) =
        crate::hookio::gated_envelope("Stop", |e: &Envelope| (&e.hook_event_name, &e.cwd))
    else {
        return ExitCode::SUCCESS;
    };
    // The loop guard skips the AUDIT, not the RECORD: a session whose
    // only Stop landed here used to leave no line at all, which reads
    // to the ledger exactly like a session the hook never ran in.
    if env.stop_hook_active {
        return unmeasured_stop(&root, &env.session_id, Some("loop_guard"));
    }
    audit(&root, &env.session_id)
}

/// Shared head of the Stop audit and the pre-commit gate: guard mode,
/// numstat over `diff`, the core's touched-duplicate verdict (None =
/// degraded, A9f) and the `event`-stamped observe entry (precommit
/// must not masquerade as a Stop audit). Outer None = git could not
/// answer: fail open.
type Gathered = (String, i64, Vec<String>, Option<verdict::Verdict>);
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
    // nothing changed = judged clean without a spawn (INFORMATION
    // never pays; the enforcement leg pays only when there is
    // something to enforce)
    let dups = if changed.is_empty() {
        Some(verdict::Verdict {
            fail: false,
            dups: 0,
            shown: Vec::new(),
        })
    } else {
        verdict::judge(root, &changed)
    };
    observe_log(
        root,
        AuditEvent {
            event,
            mode: &mode,
            session,
            net_loc,
            changed: changed.len(),
            dups: dups.as_ref().map(|v| v.dups),
            fourclass,
            skipped: None, // this leg MEASURED — see AuditEvent::skipped
            unmeasured: crate::gitmodules::unseated(root),
        },
    );
    Some((mode, net_loc, changed, dups))
}

fn audit(root: &Path, session: &str) -> ExitCode {
    // Not a git repo: nothing to audit, but the skip is RECORDED.
    let Some(base) = changes::base_rev(root) else {
        return unmeasured_stop(root, session, Some("no_git"));
    };
    let Some((mode, net_loc, _, dups)) = gather(
        root,
        &[base.as_str()],
        "stop_audit",
        Some(session),
        Some(fourclass_report(root)),
    ) else {
        return unmeasured_stop(root, session, None);
    };
    if mode == "deny"
        && let Some(v) = dups.as_ref()
        && v.fail
    {
        let payload = serde_json::json!({
            "decision": "block",
            // §4.4 B4: the Stop summary rides its own 400-token budget
            "reason": crate::hookio::clip(
                &reason(net_loc, v),
                crate::hookio::STOP_BUDGET_TOKENS,
            ),
        });
        println!("{payload}");
    }
    ExitCode::SUCCESS
}

/// Every Stop that produces NO measurement, said in the feed instead
/// of passing silently: a Stop that writes nothing reads to the M4
/// ledger exactly like a clean one, and that ledger — counted per
/// SESSION (D2-2) — is the stated gate for promoting this audit to
/// deny. `skipped` names which kind: Some(why) = an early return, the
/// audit never ran; None = git resolved the base but not the diff, a
/// real degradation (A9f). Every root here came through the throat
/// (hookio::gated_envelope), which anchors — an anchorless cwd never
/// reaches this function since batch-8 moved the gate there.
fn unmeasured_stop(root: &Path, session: &str, skipped: Option<&str>) -> ExitCode {
    let mode = crate::config::tier_of(&Config::load(root), "observe");
    observe_log(
        root,
        AuditEvent {
            event: "stop_audit",
            mode: &mode,
            session: Some(session),
            net_loc: 0,
            changed: 0,
            // measuring nothing != failing to measure: only the latter
            dups: skipped.map(|_| 0),
            fourclass: None,
            skipped,
            unmeasured: Vec::new(),
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
/// The count is the JUDGED staged set (`changes::diff`'s universe):
/// a staged `.css` can never match a dedup block either.
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
        eprintln!(
            "{}",
            crate::i18n::line(
                "ce precommit: not a git repo (skipped)",
                "ce precommit：不是 git 仓库（跳过）",
                &[],
            )
        );
        return ExitCode::SUCCESS;
    };
    let Some(v) = dups.as_ref() else {
        // A9f: fail open but never silently — the human still gets
        // the staged summary the healthy path prints
        println!("{}", staged_summary(changed.len(), net_loc, true));
        return ExitCode::SUCCESS;
    };
    if v.dups == 0 {
        println!("{}", staged_summary(changed.len(), net_loc, false));
        return ExitCode::SUCCESS;
    }
    println!("{}", reason(net_loc, v));
    if mode == "deny" && v.fail {
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

/// The staged-summary line both healthy exits print. One body, two
/// tails: written apart they were a token twin by this repo's own
/// gate. The `+` sign flag is pre-rendered because the bilingual
/// switch fills plain `{}` holes only — the en bytes are unchanged.
fn staged_summary(changed: usize, net_loc: i64, degraded: bool) -> String {
    let net = format!("{net_loc:+}");
    if degraded {
        return crate::i18n::line(
            "ce precommit: {} staged file(s), net {} LOC — duplicate \
             verdict unavailable (DEGRADED: duplicate check skipped)",
            "ce precommit：{} 个暂存文件，净 {} 行 — 重复判决不可用\
             （已降级：重复检查已跳过）",
            &[&changed, &net],
        );
    }
    crate::i18n::line(
        "ce precommit: {} staged file(s), net {} LOC, no touched duplicates",
        "ce precommit：{} 个暂存文件，净 {} 行，未触及重复块",
        &[&changed, &net],
    )
}

/// The human-facing conviction line. Cost stance, unified
/// 2026-08-19: ENFORCEMENT pays for its verdict (budgeted —
/// PERF-BUDGET.md Stop row; since 2.24.0 that includes one audit/1
/// spawn), INFORMATION never pays a spawn (fourclass_report).
fn reason(net_loc: i64, v: &verdict::Verdict) -> String {
    let net = format!("{net_loc:+}");
    crate::i18n::line(
        "ce audit: this session's edits leave {} duplicate block(s) touching \
         changed files (net {} LOC): {} — deduplicate before stopping.",
        "ce audit：本会话的编辑留下 {} 个触及改动文件的重复块\
         （净 {} 行）：{} — 停止前请先去重。",
        &[&v.dups, &net, &v.shown.join("; ")],
    )
}
