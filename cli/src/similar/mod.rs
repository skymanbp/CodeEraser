//! similar — the same-role advisor's measurement half (plan v2.29,
//! ADR-008 sixth instalment; the definition of record is methodology
//! booklet 15, drafted as `.ccm/similar-spec-2026-09-05.md`). Every
//! code unit of the unitsig universe gets a sparse six-channel bag of
//! channel-tagged fnv1a64 terms (terms.rs / bag.rs); an integer BM25
//! over the bags ranks candidates (bm25.rs); an in-repo PPMI table
//! widens a query by the terms that co-occur with its own (ppmi.rs).
//! Advisory only, in the posture of booklet 13's symbol-layer
//! advisor: nothing here produces a condition bit, feeds `ce check`,
//! or reaches `ce erase`. Rust measures; the "same role" conjunction
//! and the rational ordering become the core's once the wire family
//! lands (step 5) — until then bm25.rs carries them as the ROI
//! instrument's declared mirror.

pub mod bag;
pub mod bm25;
pub mod docs;
pub mod ppmi;
pub mod stem;
pub mod terms;

/// Bump when any term road (identifier splitting, stemming, stop
/// words, channel tags, the shape / literal / structure feature
/// spellings, doc attribution) or a scoring constant changes. It will
/// sit in the index cache key once the bag tables persist (step 3),
/// and it names the definition every frozen `similar-*` eval doc was
/// measured under.
pub const SIMILAR_REV: i64 = 1;

pub use bag::{UnitBag, file_bags};
pub use terms::Channel;
