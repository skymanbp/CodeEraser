//! The graded zone's PURE half (plan v2.6 §A, armed behind
//! `[guard] zone_tiers` since v2.7 ①): where a write lands in the
//! zone (S, H], which tier that position maps to, the two zone inputs
//! a committed baseline carries, and the per-file threshold table the
//! zone reads its lines off. Split out of budget.rs when the zone got
//! its own FPR ledger (plan v2.29 step 10, C-zone_tiers): the replay
//! instrument has to reach the SAME cut the hook runs, and a second
//! copy of 25/75 living in a test would measure a rule the product
//! does not run — which is the failure the clone ratchet and the
//! "a stated rule needs an executor" discipline both exist to refuse.
//! The hard-budget predicate lives here too (v2.35 guard lane, GD1),
//! for the same reason: its replay ledger (fpr_budget_replay) and the
//! zone's must run the hook's own rule.
//!
//! The fence over the budget inputs (`fenced`) lives here as well: the
//! replay instruments materialise each parent commit's ce.toml and
//! baseline and must judge with the budgets the hook would have used.
//! It reads the committed baseline under the root it is handed;
//! nothing else here touches the disk or the clock (the line counter
//! reads whatever reader it is handed). The hook's transports —
//! appending the feed line, asking whether this session already warned
//! — stay in budget.rs.

use crate::config::{Config, Thresholds};
use crate::scan::classes::Classes;
use serde_json::Value;
use std::path::Path;

/// The core's cut points (CE.Verdict.Cost `zoneWarnPermille` /
/// `zoneAskPermille`, the package's `limits.zone`), in permille of the
/// (S, H] span: what a baseline without a sound `zoneTiers` pair, or a
/// tree with no baseline at all, falls back to.
pub fn declared_tiers() -> (usize, usize) {
    let zone = &crate::tables::get().limits.zone;
    (zone.warn_permille, zone.ask_permille)
}

/// Where one write landed: its position in the zone and the tier the
/// map gives that position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Landing {
    /// Position in (S, H], permille of the span, floored.
    pub permille: usize,
    /// "observe" | "warn" | "ask" — the spelling the feed line and
    /// the decision carry.
    pub tier: &'static str,
}

/// The v2.6 §A map. `None` = there is nothing to say: no hard line
/// (`cap == 0`, the P3 grade-table contract's published "no line"), a
/// degenerate zone (`cap <= soft`), or a comfortable write
/// (`lines <= soft`) — silence rather than a made-up position. Past H
/// the position keeps climbing and stays priced: the hard-budget rule
/// decides there, never this map, and its caller never asks.
///
/// Since 9.0.0 the core derives S under H, so a baseline it commits
/// reaches `cap <= soft` only at equality. The branch stays for the
/// rest: a baseline committed before 9.0.0 may carry S above H, and a
/// class's own hard line may sit below the global S. Silence is right
/// there too — a write past `cap` is the hard-budget rule's (it denies),
/// and one at or under `cap` is under S as well.
pub fn landing(
    lines: usize,
    soft: usize,
    cap: usize,
    tiers: Option<(usize, usize)>,
) -> Option<Landing> {
    if cap == 0 || cap <= soft || lines <= soft {
        return None;
    }
    let permille = (lines - soft) * 1000 / (cap - soft);
    let (warn, ask) = tiers.unwrap_or_else(declared_tiers);
    let tier = if permille < warn {
        "observe"
    } else if permille <= ask {
        "warn"
    } else {
        "ask"
    };
    Some(Landing { permille, tier })
}

/// What the hard-budget rule makes of one write (v2.35 guard lane,
/// GD1): a write is refused for what it ADDS, measured against the
/// file before it — the K-round ruling for the duplicate class, now the
/// budget class's. `breach ⇔ cap ≠ 0 ∧ after > cap ∧ after > before`.
/// This predicate is the measuring side's on purpose: the budget class
/// is the hook's daemon-free class (it holds when no daemon can start),
/// and splitting one two-clause comparison across two languages would
/// make the hook's hardest refusal depend on a warm core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Budget {
    /// The write takes the file across the line (before ≤ cap).
    Crossing,
    /// The file was already past the line and the write grows it.
    Growth,
    /// Past the line, but the write keeps or shrinks it: no refusal.
    Carried,
    /// Past the line and the size before could not be read: refused
    /// as the rule stood before GD1 (no allowance on an unanswered
    /// question).
    Unread,
}

/// The rule over one write. `before` = the file's line count before
/// the write (0 for a new file), `Err` when it could not be read.
/// `None` = under the line, or no line (`cap == 0`, the P3 grade-table
/// contract's published "no line").
pub fn budget_kind(before: Result<usize, ()>, after: usize, cap: usize) -> Option<Budget> {
    if cap == 0 || after <= cap {
        return None;
    }
    Some(match before {
        Err(()) => Budget::Unread,
        Ok(_) if !grows(before, after) => Budget::Carried,
        Ok(b) if b <= cap => Budget::Crossing,
        Ok(_) => Budget::Growth,
    })
}

/// Does the write add lines? An unread `before` answers yes: an
/// allowance never rides an unanswered question. The graded zone's
/// tier fires only on a growing write (GD1), the hard budget likewise.
pub fn grows(before: Result<usize, ()>, after: usize) -> bool {
    !before.is_ok_and(|b| after <= b)
}

/// The zone's two core-authored inputs as a committed baseline
/// document carries them. Absent or unsound keys answer None, which
/// is the caller's own fallback — never a made-up value.
pub fn envelope(doc: &Value) -> (Option<usize>, Option<(usize, usize)>) {
    (soft_of(doc), tiers_of(doc))
}

/// The frozen soft line, bounded >= 1 the way the core bounds it: a
/// zero the core would refuse must fall back to the warn threshold
/// here too, or the zone opens on every file (the same guard
/// structure::judge's twin reader keeps).
fn soft_of(doc: &Value) -> Option<usize> {
    usize::try_from(doc["softLine"].as_u64()?)
        .ok()
        .filter(|s| *s >= 1)
}

/// The core-authored `[warn, ask]` permille pair, sane only when
/// 0 < warn <= ask.
fn tiers_of(doc: &Value) -> Option<(usize, usize)> {
    let warn = usize::try_from(doc["zoneTiers"][0].as_u64()?).ok()?;
    let ask = usize::try_from(doc["zoneTiers"][1].as_u64()?).ok()?;
    (warn >= 1 && warn <= ask).then_some((warn, ask))
}

/// Why a write was judged against the shipped budgets rather than the
/// declared ones (O33): its code is the guard `say` row's fence column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fence {
    None,
    Drifted,
    Unreadable,
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
/// and loosening it is a visible act. The fence rides the deny reason.
pub fn fenced(root: &Path, cfg: Config) -> (Config, Fence) {
    match crate::score::baseline::fence_status(root, &cfg) {
        Ok(f) if !f.drifted() => (cfg, Fence::None),
        Ok(_) => (shipped_budgets(cfg), Fence::Drifted),
        Err(_) => (shipped_budgets(cfg), Fence::Unreadable),
    }
}

/// The budget inputs at their shipped values; everything else as
/// declared. The tombstone table's ledger and terms are budget inputs
/// too (a drifted ledger could exempt the very file being written);
/// its tier stays declared, as `[guard] mode` does, and so does its
/// BUDGET: the shipped one is absent, which evaluates no condition at
/// all, so a drift that dropped the declared budget would have
/// switched the class off — the fence's strict side is the declared
/// number (codex review 2026-09-04).
fn shipped_budgets(mut cfg: Config) -> Config {
    let shipped = Config::default();
    cfg.thresholds = shipped.thresholds;
    cfg.exclude = shipped.exclude;
    cfg.rules = shipped.rules;
    cfg.tombstone = crate::config::TombstoneCfg {
        tier: cfg.tombstone.tier.take(),
        budget: cfg.tombstone.budget,
        ..shipped.tombstone
    };
    cfg
}

/// The threshold table one root-relative path is measured against
/// (plan v2.13 ① P4): the file's `[[rules.class]]` effective table,
/// the global one for class 0, and the global one when a declared
/// glob will not compile — the exclude list's own stance (`in_scope`
/// answers "not ours" on the same failure, and `ce scan` / `ce check`
/// refuse that config out loud on their own roads). `root` only
/// anchors the glob compiler; nothing is read from disk, so a history
/// replay can ask this of a tree it never materialized.
pub fn table_for(root: &Path, cfg: &Config, rel: &str) -> Thresholds {
    match Classes::compile(root, &cfg.rules) {
        Ok(classes) => classes.thresholds_for(cfg, rel),
        Err(_) => cfg.thresholds.clone(),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/guard/zone.rs"]
mod tests;
