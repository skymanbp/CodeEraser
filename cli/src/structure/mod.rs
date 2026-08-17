//! The structure family's MEASUREMENT face (M6 S1, design booklet
//! docs/reviews/2026-08-17-m6-structure-manager.md §3): tree-scale
//! aggregates the structure/1 judgment consumes. Everything here is
//! fact production — depths, fanouts, name-pattern distributions,
//! convention bits — and none of it judges: the entropy, the axes
//! and the verdicts live in CE.Structure.* (the ADR-008 boundary,
//! seventh family). Names and paths stay on this side; only codes,
//! counts and dense tree shape will cross the wire (§5.9.2).

pub mod edges;
pub mod tree;
