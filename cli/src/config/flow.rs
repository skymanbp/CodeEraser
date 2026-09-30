//! The flow class's own table, `[flow]` (plan v2.31 step 5; design
//! booklet docs/reference/analysis-track.md §5.4). One key:
//!
//! - `tier` — the class's OWN hook tier for the PreToolUse leg (a write
//!   that brings a new judged finding) and the gate `ce flow --check`
//!   reads. `[guard] mode` does not reach it — the tombstone class's
//!   precedent — and the class ships at observe: §4.2's route
//!   discipline (no FPR record, no promotion). The Stop leg never
//!   blocks at any tier.
//!
//! A knob of the canonical form like every other: spelled at its
//! default it is silence to `knobs_digest` (config/canonical.rs spells
//! the default out), spelled elsewhere it moves the digest by name; a
//! tier outside the four is refused at load (config.rs, tier_fault).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FlowCfg {
    pub tier: Option<String>,
}

/// The class's route default.
pub const FLOW_DEFAULT: &str = "observe";

impl FlowCfg {
    /// The declared tier, or the route default (valid by load).
    pub fn tier(&self) -> &str {
        self.tier.as_deref().unwrap_or(FLOW_DEFAULT)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/config/flow.rs"]
mod tests;
