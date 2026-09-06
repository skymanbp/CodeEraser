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
//!
//! Nothing here touches the disk or the clock. The hook's transports
//! — reading the committed baseline, appending the feed line, asking
//! whether this session already warned — stay in budget.rs.

use crate::config::{Config, Thresholds};
use crate::scan::classes::Classes;
use serde_json::Value;
use std::path::Path;

/// The DECLARED mirror of the core's cut points (CE.Verdict.Cost
/// `zoneWarnPermille` / `zoneAskPermille`), in permille of the (S, H]
/// span. Since 2.21.0 the AUTHORITY is the core's and the pair rides
/// the committed baseline; this pair is what a pre-2.21 baseline, or
/// a tree with no baseline at all, falls back to.
pub const DECLARED_TIERS: (usize, usize) = (250, 750);

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
    let (warn, ask) = tiers.unwrap_or(DECLARED_TIERS);
    let tier = if permille < warn {
        "observe"
    } else if permille <= ask {
        "warn"
    } else {
        "ask"
    };
    Some(Landing { permille, tier })
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
