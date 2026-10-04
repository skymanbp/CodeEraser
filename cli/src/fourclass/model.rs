//! The L1 judgment's shapes and line facts (the mod.rs header carries
//! the contract prose). The judgment itself — move semantics, the four
//! counts, unit attribution, the intact-relocation summary — is the
//! core's since plan v2.33 W3 (CE.FourClass.Moves, lowered by
//! moves.rs); what stays is the line diff's throat and the two line
//! facts the lowering measures. Split from the hub in the headroom
//! sprint: batch.rs and delta.rs importing these THROUGH mod.rs made
//! the family a module cycle the graph axis itself billed.

use super::decls::Decl;
use super::diff;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FourClass {
    pub added_novel: usize,
    pub added_moved: usize,
    pub removed_deleted: usize,
    pub removed_moved: usize,
}

/// One moved line with its unit attribution. `unit` is the owning
/// unit's key on the side the line sits on (None = file top level).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovedLine {
    pub line: usize, // 1-based, on its own side
    pub removed: bool,
    pub unit: Option<String>,
}

/// 1-based changed-line indices from the underlying diff, both
/// sides — the batch layer derives L1 leftovers from these without
/// re-running the diff.
#[derive(Clone, Debug)]
pub struct ChangedLines {
    pub removed: Vec<usize>,
    pub added: Vec<usize>,
}

// Clone: the batch delta merges onto a COPY so its error path can
// return the untouched pure-L1 result (review 2026-08-20 #5).
#[derive(Clone, Debug)]
pub struct Classification {
    pub counts: FourClass,
    pub moved: Vec<MovedLine>,
    /// Unit keys present on both sides whose changed lines are all
    /// moves — the "function relocated intact" summary.
    pub relocated_units: Vec<String>,
    /// Declarations that vanished from the before side / appeared on
    /// the after side (`fourclass/1` 7.1.0, O48). Measured HERE
    /// because this is the one place both unit tables are already in
    /// hand; the pairing across pairs is judged in CE.FourClass.Decl.
    pub decls: (Vec<Decl>, Vec<Decl>),
    pub changed: ChangedLines,
    pub degraded: bool,
}

/// The line diff alone: the 1-based changed lines of both sides and
/// whether the edit-distance cap tripped. The probe path's throat
/// (tombstone's added lines, churn's ledger) — no verdict needed, so
/// no core: the moves themselves are judged in CE.FourClass.Moves.
pub fn changed(before: &str, after: &str) -> (ChangedLines, bool) {
    let a: Vec<&str> = before.lines().collect();
    let b: Vec<&str> = after.lines().collect();
    let d = diff::diff(&line_hashes(&a), &line_hashes(&b));
    let one_based = |v: Vec<usize>| v.into_iter().map(|i| i + 1).collect();
    (
        ChangedLines {
            removed: one_based(d.removed),
            added: one_based(d.added),
        },
        d.degraded,
    )
}

pub(super) fn line_hashes(lines: &[&str]) -> Vec<u64> {
    use std::hash::{DefaultHasher, Hash, Hasher};
    lines
        .iter()
        .map(|l| {
            let mut h = DefaultHasher::new();
            l.hash(&mut h);
            h.finish()
        })
        .collect()
}

/// A line can carry move identity only if something in it names
/// anything — blank lines and bare punctuation match anywhere and
/// mean nothing. Public because it *is* the ground-truth significance
/// convention (labels-v1 / commit-labels-v1); eval tooling must apply
/// the same rule, from one source.
pub fn significant(line: &str) -> bool {
    line.chars().any(char::is_alphanumeric)
}

/// A line's anchor width: alphanumeric chars of the TRIMMED content.
/// A LINE FACT the aligner ships to the core (wire 2.0.0) where the
/// judgment (Cost.anchorFloor) consumes it; eval tooling measures
/// with the same rule, from one source.
pub fn alnum_width(line: &str) -> usize {
    line.trim().chars().filter(|c| c.is_alphanumeric()).count()
}
