//! How a DECLARED `[guard] mode` becomes an ENFORCED tier — policy,
//! not schema. Split out of the ce.toml struct file when resolution
//! stopped being a one-line accessor: it now VALIDATES the value and
//! RENDERS its own degradation, and three surfaces (guard.rs,
//! audit.rs, health.rs) have to agree on both halves. They did not:
//! each rendered a broken ce.toml its own way and the Stop audit
//! dropped the error entirely, while an unrecognized mode disarmed
//! every enforcement path and still printed as live (review
//! 2026-08-19).

use super::Config;
use serde::{Deserialize, Serialize};

/// Passive-guard settings (plan §4.2, decision D-4 gradual rollout:
/// observe → warn → ask → deny, promotion gated on measured FPR).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Guard {
    /// Explicit global tier: "observe" | "warn" | "ask" | "deny".
    /// Unset = the §4.2 route defaults, resolved per rule class by
    /// `tier` (step 2 landed 2026-08-11 — CHANGELOG).
    pub mode: Option<String>,
    /// v2.7 ① (OPT-IN, default off): arm the graded-zone tier map —
    /// a write landing <25% into (S, H] stays observe, 25–75%
    /// injects a warn, >75% asks. The DEFAULT stays observe-only:
    /// §4.2's FPR discipline gates any default flip on the zone
    /// feed's own record, and this switch changes one repo only.
    pub zone_tiers: bool,
}

/// §4.2 step-3 route default for the two promoted PreToolUse classes
/// (T1/T2 duplicate write, hard-budget breach): deny at 1.0, decided
/// M7-P2 on the recorded FPR ledger (CHANGELOG 2026-08-17). ONE
/// constant — guard.rs and health.rs both read it, so the enforced
/// tier and the reported tier cannot drift apart.
pub const PROMOTED_DEFAULT: &str = "deny";

/// The tiers every enforcement surface understands. Anything else in
/// `[guard] mode` is a TYPO, not a tier.
pub const TIERS: [&str; 4] = ["observe", "warn", "ask", "deny"];

impl Guard {
    /// Effective tier for one rule class: an explicit `[guard] mode`
    /// overrides every class; otherwise the plan-§4.2 route default
    /// for that class applies (PROMOTED_DEFAULT for the two classes
    /// promoted through the FPR gates, "observe" for everything else).
    ///
    /// TOTAL by construction. An unrecognized value used to come back
    /// verbatim, and each consumer then failed its own quiet way:
    /// `guard.rs` falls through its `match` to no output, `audit.rs`
    /// compares `== "deny"` and never blocks, while the SessionStart
    /// line printed `guard: Deny` as though it were armed. One typo
    /// therefore disarmed everything and reported nothing — the
    /// looks-live-does-nothing failure `deny_unknown_fields` guards
    /// against for KEYS and nothing guarded for VALUES.
    pub fn tier(&self, route_default: &str) -> String {
        match &self.mode {
            None => route_default.to_string(),
            Some(m) if TIERS.contains(&m.as_str()) => m.clone(),
            Some(bad) => format!(
                "observe (ce.toml ERROR: unknown guard mode {bad:?}, expected {})",
                TIERS.join(" | ")
            ),
        }
    }
}

/// The effective tier from a load RESULT — the ONE renderer for every
/// surface that reports or enforces it, so an unreadable ce.toml can
/// never print byte-identically to a deliberate observe.
pub fn tier_of(loaded: &Result<Config, String>, route_default: &str) -> String {
    match loaded {
        Ok(c) => c.guard.tier(route_default),
        Err(e) => {
            let gist: String = e.chars().take(80).collect();
            format!("observe (ce.toml ERROR: {gist})")
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/config/tier.rs"]
mod tests;
