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
/// 0.5.0 (6.4.0, O40): `ratchet.dropped` — the committed rows an
/// exclusion explains — present exactly when the provenance table
/// rode (every check road; absent on a legacy core's reply, which
/// judge() refuses anyway).
pub const SCHEMA_ID: &str = "ce.check-report/0.5.0";

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
    /// The floor this run was judged under (`--fail-under`), echoed
    /// so a consumer can tell "passed with a floor armed" from
    /// "passed with none". Two faces of one gate disagreed on exactly
    /// this: CI arms 950, the GUI could not arm anything, and the
    /// same tree read pass in one and FAIL in the other with nothing
    /// on screen to say why.
    pub floor: Option<u32>,
}
