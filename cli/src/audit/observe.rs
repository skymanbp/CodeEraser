//! The audit's FEED WRITER — one entry per Stop audit or precommit
//! run. Split out of audit.rs at its own 300-line dogfood gate (K
//! step 8): recording what an audit measured is a different job from
//! deciding it, and the field docs here are a WRITER CONTRACT that
//! every reader of .ce/observe.ndjson depends on — those belong with
//! the writer, not folded into the file that judges.

use std::path::Path;

/// One Stop-audit / precommit observe entry. A struct rather than
/// positional parameters: the old signature was already past the
/// scan's fn-params limit of 5.
pub(super) struct AuditEvent<'a> {
    pub event: &'a str,
    pub mode: &'a str,
    /// None for `ce precommit` — see the call site.
    pub session: Option<&'a str>,
    pub net_loc: i64,
    pub changed: usize,
    /// None = the dedup pipeline failed (A9f degraded), never
    /// flattened into "zero duplicates".
    pub dups: Option<usize>,
    /// M4 informational four-class report (Stop only; absent on the
    /// precommit path).
    pub fourclass: Option<serde_json::Value>,
    /// WRITER CONTRACT — OPTIONAL, additive: present only on a
    /// stop_audit line whose audit measured NOTHING, where the
    /// net_loc / changed_files / dup_blocks zeros are placeholders,
    /// not findings. A reader counting session coverage counts the
    /// line; one summing LOC drops it FIRST. Values: "loop_guard"
    /// (a prior Stop already blocked), "no_git" (no repo to diff).
    pub skipped: Option<&'a str>,
}

pub(super) fn observe_log(root: &Path, ev: AuditEvent) {
    let mut line = serde_json::json!({
        "event": ev.event,
        "mode": ev.mode,
        "net_loc": ev.net_loc,
        "changed_files": ev.changed,
        "degraded": ev.dups.is_none(),
        "dup_blocks": ev.dups.unwrap_or(0),
    });
    if let Some(why) = ev.skipped {
        line["skipped"] = serde_json::json!(why);
    }
    if let Some(fc) = ev.fourclass {
        line["fourclass"] = fc;
    }
    crate::hookio::observe_append(root, ev.session, line);
}
