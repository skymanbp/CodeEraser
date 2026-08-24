//! The check run's outcome model — a LEAF both mod.rs and report.rs
//! import, so the renderer never reaches back into its parent (the
//! erase model.rs precedent: a child's `super::` import is a file
//! cycle, and the cycle axis charges every member file).

use crate::score::wire;

/// 0.2.0 (review C12): ratchet.fail grew degraded/dedup semantics
/// at proto 2.5/2.6 and the ratchet object gains `failed` (held
/// condition names) + top-level `scoreScale` — plan §7.1 demands
/// the bump.
/// 0.3.0 (2.33.0, H4): candidate rows widen to six columns (the
/// leg-agreement confidence) and `joinSeverity` ships the verdict
/// table's severity face.
pub const SCHEMA_ID: &str = "ce.check-report/0.3.0";

pub struct Outcome {
    pub reply: wire::Reply,
    pub files: usize,
    pub sim_pairs: usize,
    pub members: usize,
    /// Distinct blocks that collapsed into an already-present member
    /// id (same unit pair, second block) — reported, never silent.
    pub collapsed: usize,
    /// Intra-file block pairs the sim table cannot carry (u < v is
    /// the wire contract); their members still enter the set.
    pub skipped_self: usize,
}
