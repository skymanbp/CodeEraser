//! The hard-budget rule class (§4.2 step 2, split from guard.rs at
//! the 300-line dogfood wall): does this write GROW the file past
//! thresholds.file_lines_fail? Exact arithmetic on the applied edit
//! (zone::budget_kind, v2.35 guard lane GD1), daemon-free, scan-scope
//! only. A write that keeps or shrinks a file already past the line is
//! carried — recorded, never refused — so "split the file" can be done
//! one step at a time.

use super::applied::{self, Applied};
use super::envelope::Envelope;
use super::speech::Said;
use super::zone::Fence;
use super::zone::{Budget, GROWTH_TIER, GrowthTier};
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
    super::zone::table_for(root, cfg, &rel)
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
/// hard-budget rule and the v2.6 zone observer. Scope is judged BEFORE
/// any disk read: a hook must not pay (or take) an unbounded read for a
/// file it is about to declare out of scope.
fn sized_write(root: &Path, cfg: &Config, env: &Envelope, applied: &Applied) -> Option<usize> {
    let path = Path::new(&env.tool_input.file_path);
    // sized_path (plan v2.30 step 5b-8): a prose-only file has no size
    // budget, exactly like an unknown extension
    crate::scan::lang::Lang::sized_path(path)?;
    if !crate::scan::walk::in_scope(root, path, &cfg.exclude) {
        return None;
    }
    Some(applied.text.lines().count())
}

/// The size before this write, read only when a rule needs it (a write
/// that lands under the soft line pays nothing): `Ok(None)` = the file
/// does not exist yet.
type Prior = Result<Option<usize>, std::io::ErrorKind>;

/// The prior as the pure rule reads it: absent = 0 lines, unread = Err.
fn before_of(p: &Prior) -> Result<usize, ()> {
    p.map(Option::unwrap_or_default).map_err(|_| ())
}

/// The refusal a breach speaks: the file's line, the size before and
/// after, which kind, and the fence the budget was judged under.
fn over_budget(env: &Envelope, kind: Budget, prior: &Prior, lines: usize, cap: usize, fence: Fence) -> Said {
    Said::OverBudget {
        file: env.tool_input.file_path.clone(),
        lines,
        cap,
        fence,
        before: prior.ok().flatten(),
        kind,
    }
}

/// What the two size classes leave for the decision: their feed lines
/// (a budget line, or a zone line — the hook writes them after the
/// probe's, 0.11.0), the budget reason at the class tier unless the
/// breach was warned this session already, and the firings that speak
/// at their own tier (the armed zone; a growth when GROWTH_TIER names
/// one).
pub(super) struct SizeClasses {
    pub lines: Vec<serde_json::Value>,
    pub budget: Option<Said>,
    pub tiered: Vec<(&'static str, Said)>,
}

/// Both size classes over one write, measured once: the hard budget
/// against the lines THIS file is measured against — its class's, or
/// the global table (plan v2.13 ① P4) — and, under it, the graded
/// zone. No config, a write outside the scan's scope, or an Edit that
/// cannot apply (GD3: the probe line says why) is nothing.
pub(super) fn size_classes(
    root: &Path,
    env: &Envelope,
    cfg: Option<&(Config, Fence)>,
    applied: Option<&Applied>,
    mode: &str,
    budget_seen: bool,
) -> SizeClasses {
    let mut out = SizeClasses {
        lines: Vec::new(),
        budget: None,
        tiered: Vec::new(),
    };
    let (Some((c, fence)), Some(a)) = (cfg, applied) else {
        return out;
    };
    let Some(lines) = sized_write(root, c, env, a) else {
        return out;
    };
    let t = lines_for(root, c, &env.tool_input.file_path);
    let prior = || applied::before_lines(env, a);
    let cap = t.file_lines_fail;
    if cap != 0 && lines > cap {
        let p = prior();
        let kind = super::zone::budget_kind(before_of(&p), lines, cap).expect("past a line");
        out.lines.push(budget_line(env, mode, lines, &p, kind));
        if kind == Budget::Carried || budget_seen {
            return out;
        }
        let said = over_budget(env, kind, &p, lines, cap, *fence);
        match (kind, GROWTH_TIER) {
            (Budget::Growth, GrowthTier::Named(tier)) => out.tiered.push((tier, said)),
            _ => out.budget = Some(said),
        }
        return out;
    }
    // under the hard line: the zone observer, plus the v2.7 ① OPT-IN
    // tier map (default stays feed-only)
    let (line, fired) = zone_assess(root, &ZoneLines::of(&t, c), env, mode, (lines, prior));
    out.lines.extend(line);
    out.tiered.extend(fired);
    out
}

/// Every write past the line gets its own feed line, in every tier —
/// a refused one (`kind`), a carried one (`carried: true`, no reason),
/// or one whose size before could not be read (`before_unread`, a
/// degraded stamp: refused as the rule stood, with no allowance).
fn budget_line(env: &Envelope, mode: &str, lines: usize, prior: &Prior, kind: Budget) -> serde_json::Value {
    let mut line = serde_json::json!({
        "event": "budget",
        "file": env.tool_input.file_path,
        "mode": mode,
        "resulting_lines": lines,
        "before_lines": prior.ok().map(Option::unwrap_or_default),
    });
    match (kind, prior) {
        (Budget::Carried, _) => line["carried"] = true.into(),
        (Budget::Unread, Err(e)) => {
            line["before_unread"] = format!("{e:?}").into();
            line["degraded"] = format!("before_unread: {e:?}").into();
        }
        (Budget::Crossing, _) => line["kind"] = "crossing".into(),
        (Budget::Growth, _) => line["kind"] = "growth".into(),
        (Budget::Unread, Ok(_)) => unreachable!("budget_kind reads Unread only off an Err"),
    }
    line
}

/// plan v2.6 §A observe leg + the v2.7 ① OPT-IN tier map: a write
/// landing INSIDE the graded zone (S, H] gets a `zone` feed line in
/// every tier; with `[guard] zone_tiers` armed the position maps to a
/// tier (<25% observe / 25–75% warn / >75% ask) and the firing returns
/// (tier, reason) — the DEFAULT stays observe-only (§4.2 FPR
/// discipline). Since GD1 a tier fires only on a write that GROWS the
/// file: a write that keeps or shrinks it logs `carried: true` and
/// `zone_tier: observe`. A zone warn rides the same per-session (rule,
/// file) injection budget as the class warns; ask is enforcement and
/// repeats. S = the committed baseline's frozen softLine, falling back
/// to the file's warn line (its class's since P4, else
/// thresholds.file_lines_warn); H is the file's hard line likewise; a
/// degenerate zone (H <= S, or no hard line) logs nothing rather than a
/// made-up position.
fn zone_assess(
    root: &Path,
    z: &ZoneLines,
    env: &Envelope,
    mode: &str,
    (lines, prior): (usize, impl FnOnce() -> Prior),
) -> (Option<serde_json::Value>, Option<(&'static str, Said)>) {
    let (frozen, tiers) = committed(root);
    let soft = frozen.unwrap_or(z.warn);
    let Some(super::zone::Landing { permille, tier }) =
        super::zone::landing(lines, soft, z.cap, tiers)
    else {
        return (None, None);
    };
    let p = prior();
    let grows = super::zone::grows(before_of(&p), lines);
    let tier = if grows { tier } else { "observe" };
    let file = &env.tool_input.file_path;
    // the B4 suppression consults the feed BEFORE this event lands in
    // it (a warn must not read its own fresh line as "already warned")
    let seen = tier == "warn" && crate::hookio::already_warned(root, &env.session_id, "zone", file);
    let mut line = serde_json::json!({
        "event": "zone",
        "file": file,
        "mode": mode,
        "resulting_lines": lines,
        "before_lines": p.ok().map(Option::unwrap_or_default),
        "soft": soft,
        "hard": z.cap,
        "zone_permille": permille,
    });
    if !grows {
        line["carried"] = true.into();
    }
    if z.armed {
        line["zone_tier"] = tier.into();
    }
    let fired = (z.armed && tier != "observe" && !seen).then(|| {
        let said = Said::Zone {
            file: file.clone(),
            lines,
            permille,
            soft,
            cap: z.cap,
        };
        (tier, said)
    });
    (Some(line), fired)
}

/// The zone's two core-authored inputs off the committed
/// ce-baseline.json — the hook's one channel to the §B fence
/// (daemon-free by design, so a plain local read is the honest
/// transport). ONE read per write. An unreadable or absent baseline
/// answers (None, None) — every fallback is the pure half's, spelled
/// once (guard::zone).
fn committed(root: &Path) -> (Option<usize>, Option<(usize, usize)>) {
    match crate::score::baseline::document(root) {
        Ok(Some(doc)) => super::zone::envelope(&doc),
        _ => (None, None),
    }
}
