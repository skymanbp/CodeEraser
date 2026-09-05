//! similar — the same-role advisor's measurement half (plan v2.29,
//! ADR-008 sixth instalment; the definition of record is methodology
//! booklet 15, drafted as `.ccm/similar-spec-2026-09-05.md`). Every
//! code unit of the unitsig universe gets a sparse six-channel bag of
//! channel-tagged fnv1a64 terms (terms.rs / bag.rs); an integer BM25
//! over the bags ranks candidates (bm25.rs); an in-repo PPMI table
//! widens a query by the terms that co-occur with its own (ppmi.rs).
//! The bags persist in `.ce/index.db` as hashes and counts only
//! (store.rs writes them inside the per-file refresh, by difference;
//! reader.rs ranks off the tables), so a query pays for its own
//! postings and never for a corpus rebuild. Advisory only, in the
//! posture of booklet 13's symbol-layer advisor: nothing here produces
//! a condition bit, feeds `ce check`, or reaches `ce erase`. Rust
//! measures; the "same role" conjunction and the rational ordering
//! become the core's once the wire family lands (step 5) — until then
//! bm25.rs carries them as the ROI instrument's declared mirror.

pub mod bag;
pub mod bm25;
pub mod docs;
pub mod ppmi;
pub mod reader;
pub mod stem;
pub mod store;
pub mod terms;

/// Bump when any term road (identifier splitting, stemming, stop
/// words, channel tags, the shape / literal / structure feature
/// spellings, doc attribution), the PPMI cap, or a scoring constant
/// changes. It sits in the index cache key (dedup/schema.rs), so a
/// bump wipes the persisted bags with the rest of the cache, and it
/// names the definition every frozen `similar-*` eval doc was
/// measured under.
pub const SIMILAR_REV: i64 = 1;

pub use bag::{UnitBag, file_bags};
pub use terms::Channel;
