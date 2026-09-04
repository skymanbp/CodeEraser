//! The audit's FEED WRITER — one entry per Stop audit or git-hook
//! face run (precommit, commitmsg). Split out of audit.rs at its own 300-line dogfood gate (K
//! step 8): recording what an audit measured is a different job from
//! deciding it, and the field docs here are a WRITER CONTRACT that
//! every reader of .ce/observe.ndjson depends on — those belong with
//! the writer, not folded into the file that judges.

use std::path::Path;

/// One Stop-audit / precommit / commitmsg observe entry. A struct rather than
/// positional parameters: the old signature was already past the
/// scan's fn-params limit of 5.
pub(super) struct AuditEvent<'a> {
    pub event: &'a str,
    pub mode: &'a str,
    /// None for the git-hook faces (`ce precommit` / `ce commitmsg`)
    /// — see the call site.
    pub session: Option<&'a str>,
    pub net_loc: i64,
    pub changed: usize,
    /// None = the dedup pipeline failed (A9f degraded), never
    /// flattened into "zero duplicates".
    pub dups: Option<usize>,
    /// M4 informational four-class report (Stop only; absent on the
    /// precommit path).
    pub fourclass: Option<serde_json::Value>,
    /// WRITER CONTRACT — OPTIONAL, additive (0.8.0): the tombstone
    /// measurement of the same diff (tombstone::feed_json without the
    /// per-edit keys); absent when git could not pair the change.
    pub tombstone: Option<serde_json::Value>,
    /// WRITER CONTRACT — OPTIONAL, additive: present only on a
    /// stop_audit line whose audit measured NOTHING, where the
    /// net_loc / changed_files / dup_blocks zeros are placeholders,
    /// not findings. A reader counting session coverage counts the
    /// line; one summing LOC drops it FIRST. Values: "loop_guard"
    /// (a prior Stop already blocked), "no_git" (no repo to diff).
    pub skipped: Option<&'a str>,
    /// WRITER CONTRACT — OPTIONAL, additive: the declared submodules
    /// (gitmodules.rs) this audit could NOT measure because their
    /// checkout is unseated, ce-root-relative. The line's numbers are
    /// the parent's real measurement — PARTIAL, never `skipped` (that
    /// field voids the whole line for a LOC-summing reader); a reader
    /// judging a session's coverage treats a non-empty list as a
    /// named shortfall, the audit's mirror of trend's by-name refusal.
    pub unmeasured: Vec<String>,
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
    if !ev.unmeasured.is_empty() {
        line["unmeasured"] = serde_json::json!(ev.unmeasured);
    }
    if let Some(fc) = ev.fourclass {
        line["fourclass"] = fc;
    }
    if let Some(t) = ev.tombstone {
        line["tombstone"] = t;
    }
    crate::hookio::observe_append(root, ev.session, line);
}
