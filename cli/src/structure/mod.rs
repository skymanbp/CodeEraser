//! The structure family's MEASUREMENT face (M6 S1; axis semantics
//! in docs/reference/structure-axes.md, full booklet in git
//! history): tree-scale aggregates the structure/1 judgment consumes. Everything here is
//! fact production — depths, fanouts, name-pattern distributions,
//! convention bits — and none of it judges: the entropy, the axes
//! and the verdicts live in CE.Structure.* (the ADR-008 boundary,
//! seventh family). Names and paths stay on this side; only codes,
//! counts and dense tree shape will cross the wire (§5.9.2).

pub mod edges;
pub mod judge;
pub mod seams;
// the report faces (JSON doc + bilingual console), split from
// judge.rs at the 300-line dogfood gate when the M8-G3b Chinese
// console landed; callers name report directly since the headroom
// sprint — the judge re-export made judge<->report a cycle the
// graph family itself billed
pub mod report;
pub mod rows;
pub mod tree;
pub mod wire;
