//! The tombstone class's own table, `[tombstone]` (plan v2.27 step 3;
//! the class itself measures in cli/src/tombstone). Four keys, each
//! with a reason to be a key of its own:
//!
//! - `tier` — the class's OWN hook tier. `[guard] mode` does not reach
//!   it: a class with a key of its own decides at that key (the graded
//!   zone's precedent, `zone_tiers`), and this class ships at observe
//!   until docs/FPR-TOMBSTONE.md argues a promotion — §4.2's route
//!   discipline, spelled as the default here.
//! - `budget` — sites one changeset may carry before the class's
//!   condition holds (`sites > budget`, judged in the core over
//!   tombstone/1). Absent = the condition is never evaluated and the
//!   class is feed-only — `[dedup] budget`'s precedent.
//! - `ledger` — files declared to hold the changelog role (the exclude
//!   list's dialect): exempt whole, counted `declared` — the backstop
//!   the 2026-09-04 ruling gave the segment witness.
//! - `terms` — this repository's own vocabulary: words that never
//!   spell a name, whole or as a word of a compound.
//!
//! Every key is a knob of the canonical form (config/canonical.rs):
//! spelled at its effective default it is silence, spelled elsewhere
//! it moves `knobs_digest` by name — the tier included, the way
//! `[guard] mode` already does.

use super::tier::TIERS;
use serde::{Deserialize, Serialize};

/// The class's route default (§4.2: no FPR record, no promotion).
pub const TOMBSTONE_DEFAULT: &str = "observe";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct TombstoneCfg {
    pub tier: Option<String>,
    pub budget: Option<u32>,
    pub ledger: Vec<String>,
    pub terms: Vec<String>,
}

impl TombstoneCfg {
    /// The load-throat refusal: a tier outside the four is a typo,
    /// named and refused (a hook then fails open naming the error
    /// through `config::tier_of`, a command exits 2) — nothing runs
    /// with a tier it cannot read, and nothing looks armed by one.
    pub(crate) fn fault(&self) -> Option<String> {
        match self.tier.as_deref() {
            Some(t) if !TIERS.contains(&t) => Some(format!(
                "ce.toml [tombstone] tier {t:?}: expected one of {}",
                TIERS.join(" | ")
            )),
            _ => None,
        }
    }

    /// The class's tier — declared, or the route default. Valid by
    /// load (`fault`), so a bare read.
    pub fn tier(&self) -> &str {
        self.tier.as_deref().unwrap_or(TOMBSTONE_DEFAULT)
    }

    /// The table as the class judges it when nothing is declared: the
    /// default tier spelled out is silence to the digest (the
    /// `TrendCfg::core` precedent for a knob with a live default).
    pub(crate) fn effective() -> Self {
        Self {
            tier: Some(TOMBSTONE_DEFAULT.to_string()),
            ..Self::default()
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/config/tombstone.rs"]
mod tests;
