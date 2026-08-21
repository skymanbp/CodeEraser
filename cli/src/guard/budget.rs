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

/// plan v2.6 §A observe leg + the v2.7 ① OPT-IN tier map: a write
/// landing INSIDE the graded zone (S, H] gets a `zone` feed line in
/// every tier; with `[guard] zone_tiers` armed the position maps to
/// a tier (<25% observe / 25–75% warn / >75% ask) and the firing
/// returns (tier, reason) for the decision line — the DEFAULT stays
/// observe-only (§4.2 FPR discipline). A zone warn rides the same
/// per-session (rule, file) injection budget as the class warns;
/// ask is enforcement and repeats. S = the committed baseline's
/// frozen softLine, falling back to thresholds.file_lines_warn — the
/// same fallback the score's size axis uses; a degenerate zone
/// (H <= S, or no hard line) logs nothing rather than a made-up
/// position.
pub(super) fn zone_assess(
    root: &Path,
    cfg: &Config,
    env: &Envelope,
    mode: &str,
    lines: usize,
) -> Option<(&'static str, String)> {
    let cap = cfg.thresholds.file_lines_fail;
    let soft = committed_soft(root).unwrap_or(cfg.thresholds.file_lines_warn);
    if cap == 0 || cap <= soft || lines <= soft {
        return None;
    }
    let permille = (lines - soft) * 1000 / (cap - soft);
    let armed = cfg.guard.zone_tiers;
    let tier = zone_tier(permille);
    let file = &env.tool_input.file_path;
    // the B4 suppression consults the feed BEFORE this event lands
    // in it (the probe rule's ordering — a warn must not read its
    // own fresh line as "already warned")
    let seen = tier == "warn" && crate::hookio::already_warned(root, &env.session_id, "zone", file);
    let mut line = serde_json::json!({
        "event": "zone",
        "file": file,
        "mode": mode,
        "resulting_lines": lines,
        "soft": soft,
        "hard": cap,
        "zone_permille": permille,
    });
    if armed {
        line["zone_tier"] = tier.into();
    }
    crate::hookio::observe_append(root, Some(&env.session_id), line);
    if !armed || tier == "observe" || seen {
        return None;
    }
    Some((
        tier,
        format!(
            "ce: this write leaves {file} at {lines} lines, {permille}‰ into the \
             graded zone ({soft}..{cap}); `ce structure --split-candidates` prices \
             its best seam.",
        ),
    ))
}

/// The v2.6 §A map, wired v2.7 ① behind the ce.toml opt-in.
fn zone_tier(permille: usize) -> &'static str {
    match permille {
        0..=249 => "observe",
        250..=750 => "warn",
        _ => "ask",
    }
}

/// The frozen soft line, read off the committed ce-baseline.json —
/// the hook's one channel to the §B fence (daemon-free by design,
/// so a plain local read is the honest transport). Bounded >= 1 the
/// way the core bounds it: a zero the core would refuse must fall to
/// the warn threshold here too, or the zone opens on every file
/// (same guard as structure::judge::committed_soft, its twin).
fn committed_soft(root: &Path) -> Option<usize> {
    let doc = crate::score::baseline::read(root).ok()??;
    usize::try_from(doc["softLine"].as_u64()?)
        .ok()
        .filter(|s| *s >= 1)
}
