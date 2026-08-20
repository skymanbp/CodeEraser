//! The hard-budget rule class (§4.2 step 2, split from guard.rs at
//! the 300-line dogfood wall): would this write leave the file past
//! thresholds.file_lines_fail? Exact arithmetic on the applied edit,
//! daemon-free, scan-scope only.

use super::Envelope;
use crate::config::Config;
use std::path::Path;

/// The write would leave the file past the hard budget. Scope is
/// judged BEFORE any disk read: the old order read the target file
/// whole and only then asked whether the scanner would ever count it
/// — a hook must not pay (or take) an unbounded read for a file it
/// is about to declare out of scope.
pub(super) fn budget_breach(root: &Path, cfg: &Config, env: &Envelope) -> Option<(usize, String)> {
    let cap = cfg.thresholds.file_lines_fail;
    // cap 0 = no hard line exists (the P3 grade-table contract) —
    // without this the hook read 0 as "every write breaches"
    if cap == 0 {
        return None;
    }
    let path = Path::new(&env.tool_input.file_path);
    crate::scan::lang::Lang::from_path(path)?;
    if !crate::scan::walk::in_scope(root, path, &cfg.exclude) {
        return None;
    }
    let lines = resulting_lines(env)?;
    if lines <= cap {
        return None;
    }
    let why = format!(
        "ce: this write leaves {} at {lines} lines, past the hard budget \
         of {cap} (plan §4.1). Split the file instead of growing it.",
        env.tool_input.file_path
    );
    Some((lines, why))
}

/// Exact post-write line count: Write is the payload itself; Edit is
/// the payload applied to the on-disk file under Edit's own semantics
/// — CRLF-normalized, and single replacement requires a UNIQUE match
/// exactly as the real tool does (attack review F11: replacen on a
/// non-unique old_string would judge a write that will never land).
/// None = the tool call is failing on its own (missing file, absent
/// or ambiguous old_string) — the budget rule stays silent.
fn resulting_lines(env: &Envelope) -> Option<usize> {
    let t = &env.tool_input;
    if env.tool_name == "Write" {
        return Some(t.content.lines().count());
    }
    let on_disk = std::fs::read_to_string(&t.file_path)
        .ok()?
        .replace("\r\n", "\n");
    let (old, new) = (
        t.old_string.replace("\r\n", "\n"),
        t.new_string.replace("\r\n", "\n"),
    );
    if old.is_empty() {
        return None;
    }
    let applied = match (t.replace_all, on_disk.matches(&old).count()) {
        (_, 0) => return None,
        (false, 1) => on_disk.replacen(&old, &new, 1),
        (false, _) => return None, // the real Edit rejects ambiguity
        (true, _) => on_disk.replace(&old, &new),
    };
    Some(applied.lines().count())
}

/// Budget firings get their own feed line (accounting for the §4.2
/// step-3 decision at 1.0 needs per-rule records), in every tier.
pub(super) fn budget_log(root: &Path, env: &Envelope, mode: &str, lines: usize) {
    crate::hookio::observe_append(
        root,
        Some(&env.session_id),
        serde_json::json!({
            "event": "budget",
            "file": env.tool_input.file_path,
            "mode": mode,
            "resulting_lines": lines,
        }),
    );
}
