//! The check run's outcome model — a LEAF both mod.rs and the
//! document request import, so the document never reaches back into
//! its parent (the erase model.rs precedent: a child's `super::`
//! import is a file cycle, and the cycle axis charges every member
//! file). The document's schema id is the core's (CE.Score.Document,
//! plan v2.32 step 4).

use crate::score::wire;

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
    /// this: CI arms 911, the GUI could not arm anything, and the
    /// same tree read pass in one and FAIL in the other with nothing
    /// on screen to say why.
    pub floor: Option<u32>,
    /// The verdict's core link, whole after the judgment: the check
    /// document is laid out over it (plan v2.32 step 4), so a run
    /// starts no second core process for its document.
    pub held: crate::document::Held,
}
