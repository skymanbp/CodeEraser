//! The structure family's MEASUREMENT face (M6 S1; axis semantics
//! in docs/reference/structure-axes.md, full booklet in git
//! history): what the structure/1 judgment is asked about — the walked
//! paths, the graph's arcs, the staleness and redundancy facts, the
//! split-ROI seams — and none of it judges or aggregates: the tree, its
//! depths, fanouts, name shapes and convention bits (plan v2.33 W1
//! item 3), the entropy, the axes and the verdicts live in
//! CE.Structure.* (the ADR-008 boundary, seventh family). The paths
//! cross the local pipe only (§3 of the algorithm-track booklet).

mod document;
pub mod judge;
pub mod seams;
// the report faces (JSON doc + bilingual console), split from
// judge.rs at the 300-line dogfood gate when the M8-G3b Chinese
// console landed; callers name report directly since the headroom
// sprint — the judge re-export made judge<->report a cycle the
// graph family itself billed
pub mod report;
pub mod rows;
pub mod wire;

// the tree, the edge counts, the request rows and the arch tables as they
// stood before the core built them (plan v2.33 W1 item 3), frozen (tests
// subrepo unit/structure/oracle/, unit/arch/oracle/): their unit tests keep
// testing them, and the differential legs hold the core to them
#[cfg(test)]
#[path = "../../tests/unit/structure/frozen.rs"]
pub(crate) mod oracle;
