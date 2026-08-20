//! The hard-budget rule class (§4.2 step 2, split from guard.rs at
//! the 300-line dogfood wall): would this write leave the file past
//! thresholds.file_lines_fail? Exact arithmetic on the applied edit,
//! daemon-free, scan-scope only.

use super::Envelope;
use crate::config::Config;
use std::path::Path;

/// Scope + exact post-write size — ONE measurement feeding both the
/// hard-budget rule and the v2.6 zone observer (a second copy of
/// this stanza is exactly what the dedup ratchet exists to refuse).
/// Scope is judged BEFORE any disk read: the old order read the
/// target file whole and only then asked whether the scanner would
/// ever count it — a hook must not pay (or take) an unbounded read
/// for a file it is about to declare out of scope.
pub(super) fn sized_write(root: &Path, cfg: &Config, env: &Envelope) -> Option<usize> {
    let path = Path::new(&env.tool_input.file_path);
    crate::scan::lang::Lang::from_path(path)?;
    if !crate::scan::walk::in_scope(root, path, &cfg.exclude) {
        return None;
    }
    resulting_lines(env)
}

/// The write would leave the file past the hard budget.
pub(super) fn budget_breach(cfg: &Config, env: &Envelope, lines: usize) -> Option<String> {
    let cap = cfg.thresholds.file_lines_fail;
    // cap 0 = no hard line exists (the P3 grade-table contract) —
    // without this the hook read 0 as "every write breaches"
    if cap == 0 || lines <= cap {
        return None;
    }
    Some(format!(
        "ce: this write leaves {} at {lines} lines, past the hard budget \
         of {cap} (plan §4.1). Split the file instead of growing it.",
        env.tool_input.file_path
    ))
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

/// plan v2.6 §A, the OBSERVE leg (feed 0.5.0): a write landing
/// INSIDE the graded zone (S, H] gets a `zone` line in every tier —
/// no stdout, no enforcement (the only armed size line stays the
/// hard budget at H). This feed IS the per-rule record the future
/// zone→tier map must argue its FPR case from (§4.2: no record, no
/// promotion eligibility). S = the committed baseline's frozen
/// softLine, falling back to thresholds.file_lines_warn — the same
/// fallback the score's size axis uses; a degenerate zone (H <= S,
/// or no hard line) logs nothing rather than a made-up position.
pub(super) fn zone_log(root: &Path, cfg: &Config, env: &Envelope, mode: &str, lines: usize) {
    let cap = cfg.thresholds.file_lines_fail;
    let soft = committed_soft(root).unwrap_or(cfg.thresholds.file_lines_warn);
    if cap == 0 || cap <= soft || lines <= soft {
        return;
    }
    crate::hookio::observe_append(
        root,
        Some(&env.session_id),
        serde_json::json!({
            "event": "zone",
            "file": env.tool_input.file_path,
            "mode": mode,
            "resulting_lines": lines,
            "soft": soft,
            "hard": cap,
            "zone_permille": (lines - soft) * 1000 / (cap - soft),
        }),
    );
}

/// The frozen soft line, read off the committed ce-baseline.json —
/// the hook's one channel to the §B fence (daemon-free by design,
/// so a plain local read is the honest transport).
fn committed_soft(root: &Path) -> Option<usize> {
    let doc = crate::score::baseline::read(root).ok()??;
    usize::try_from(doc["softLine"].as_u64()?).ok()
}
