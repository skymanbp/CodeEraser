//! The hard-budget rule class (§4.2 step 2, split from guard.rs at
//! the 300-line dogfood wall): would this write leave the file past
//! thresholds.file_lines_fail? Exact arithmetic on the applied edit,
//! daemon-free, scan-scope only.

use super::envelope::Envelope;
use crate::config::{Config, Thresholds};
use std::path::Path;

/// The lines THIS write is measured against (plan v2.13 ① P4, zero
/// wire): the file's class's effective table, the global one for
/// class 0 — the same reading the scan gate and the score take, so
/// the hook denies at exactly the line the CI wall would fail at.
/// The classes compile per hook run (one override set per class —
/// the exclude set's own cost, which in_scope already pays). A class
/// glob that will not compile falls to the global table: the exclude
/// list's own stance (in_scope answers "not ours" on the same
/// failure), and `ce scan`/`ce check` refuse that config out loud.
pub(super) fn lines_for(root: &Path, cfg: &Config, path: &str) -> Thresholds {
    let rel = crate::scan::walk::rel_str(root, Path::new(path));
    match crate::scan::classes::Classes::compile(root, &cfg.rules) {
        Ok(classes) => classes.thresholds_for(cfg, &rel),
        Err(_) => cfg.thresholds.clone(),
    }
}

/// The graded zone's two lines and its arming, resolved once per
/// write from the file's own table (P4) and ce.toml's opt-in.
pub(super) struct ZoneLines {
    pub cap: usize,
    pub warn: usize,
    pub armed: bool,
}

impl ZoneLines {
    pub(super) fn of(t: &Thresholds, cfg: &Config) -> Self {
        ZoneLines {
            cap: t.file_lines_fail,
            warn: t.file_lines_warn,
            armed: cfg.guard.zone_tiers,
        }
    }
}

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

/// The write would leave the file past ITS hard budget — the file's
/// class line, or the global one (lines_for). `fence` is the note a
/// drifted config earned (fenced): the reason says which budget it
/// was judged against and why.
pub(super) fn budget_breach(
    t: &Thresholds,
    env: &Envelope,
    lines: usize,
    fence: Option<&str>,
) -> Option<String> {
    let cap = t.file_lines_fail;
    // cap 0 = no hard line exists (the P3 grade-table contract) —
    // without this the hook read 0 as "every write breaches"
    if cap == 0 || lines <= cap {
        return None;
    }
    Some(format!(
        "ce: this write leaves {} at {lines} lines, past the hard budget \
         of {cap} (plan §4.1). Split the file instead of growing it.{}",
        env.tool_input.file_path,
        fence.map_or(String::new(), |f| format!(" {f}"))
    ))
}

/// The config the hook judges budgets with (6.4.0, O33): the declared
/// one when the tree is unfenced (no committed baseline — the fence
/// arms with the file, nowhere else) or the fence matches; the
/// SHIPPED defaults for the three budget inputs — thresholds, the
/// exclude list, the classes — when the declared ce.toml drifted from
/// the fenced baseline or the baseline cannot be read. A drifted
/// config is an unverified document, and a guard that kept any part
/// of it would let the edit that produced the drift choose its own
/// budget (`file_lines_fail = 0` = no line; `exclude = ["src/**"]` =
/// out of scope). `[guard]` mode and the tier arming stay as
/// declared: a mode is not a budget, it prints in the status line,
/// and loosening it is a visible act. The note names the fence for
/// the deny reason.
pub(super) fn fenced(root: &Path, cfg: Config) -> (Config, Option<&'static str>) {
    match crate::score::baseline::fence_status(root, &cfg) {
        Ok(f) if !f.drifted() => (cfg, None),
        Ok(_) => (
            shipped_budgets(cfg),
            Some("(ce.toml drifted from the fenced baseline: judged with the shipped budgets)"),
        ),
        Err(_) => (
            shipped_budgets(cfg),
            Some("(the committed baseline is unreadable: judged with the shipped budgets)"),
        ),
    }
}

/// The three budget inputs at their shipped values; everything else
/// as declared.
fn shipped_budgets(mut cfg: Config) -> Config {
    let shipped = Config::default();
    cfg.thresholds = shipped.thresholds;
    cfg.exclude = shipped.exclude;
    cfg.rules = shipped.rules;
    cfg
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
/// frozen softLine, falling back to the file's warn line (its class's
/// since P4, else thresholds.file_lines_warn) — the same fallback the
/// score's size axis uses; H is the file's hard line likewise; a
/// degenerate zone (H <= S, or no hard line) logs nothing rather
/// than a made-up position.
pub(super) fn zone_assess(
    root: &Path,
    z: &ZoneLines,
    env: &Envelope,
    mode: &str,
    lines: usize,
) -> Option<(&'static str, String)> {
    let cap = z.cap;
    let soft = committed_soft(root).unwrap_or(z.warn);
    if cap == 0 || cap <= soft || lines <= soft {
        return None;
    }
    let permille = (lines - soft) * 1000 / (cap - soft);
    let armed = z.armed;
    let tier = zone_tier(permille, committed_tiers(root));
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

/// The v2.6 §A map, wired v2.7 ① behind the ce.toml opt-in. Since
/// 2.21.0 (batch-7 slice 5) the cut points are CORE-authored
/// (CE.Verdict.Cost zoneWarnPermille/zoneAskPermille) and ride the
/// committed baseline; the compiled pair is the declared mirror a
/// pre-2.21 baseline falls back to.
fn zone_tier(permille: usize, tiers: Option<(usize, usize)>) -> &'static str {
    let (warn, ask) = tiers.unwrap_or((250, 750));
    if permille < warn {
        "observe"
    } else if permille <= ask {
        "warn"
    } else {
        "ask"
    }
}

/// The core-authored tier pair off the committed baseline (the
/// committed_soft transport, second key): [warn, ask] permille,
/// sane only when 0 < warn <= ask.
fn committed_tiers(root: &Path) -> Option<(usize, usize)> {
    let doc = crate::score::baseline::read(root).ok()??;
    let warn = usize::try_from(doc["zoneTiers"][0].as_u64()?).ok()?;
    let ask = usize::try_from(doc["zoneTiers"][1].as_u64()?).ok()?;
    (warn >= 1 && warn <= ask).then_some((warn, ask))
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
