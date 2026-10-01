//! The tombstone rule's vocabulary (plan v2.26 T track; the spec's §三
//! tables). Since plan v2.32 step 2 the tables are the core's
//! (CE.Lang.Common.Prose `tombstone`, read off `tables/1`,
//! crate::tables):
//! the absence words, the reserved words of the judged languages, the
//! English and Chinese absence frames, the retrospective marks, the
//! English function words, the name floors (3 ASCII chars, 2 wide), the
//! three-word join window and the bracket pairs. The readers live in
//! frames.rs and names.rs. Every table is a `TOMBSTONE_REV` input:
//! extend one only with corpus evidence from the FPR ledger, and bump
//! the revision.

use crate::tables::Tombstone;

/// The vocabulary revision the feed records (`rev`): a reader of the
/// FPR ledger must know which tables produced a row.
pub const TOMBSTONE_REV: i64 = 1;

/// The vocabulary (`tombstone` in the package).
pub fn v() -> &'static Tombstone {
    &crate::tables::get().tombstone
}

/// The entries of a table.
pub fn entries(table: &'static [&'static str]) -> impl Iterator<Item = &'static str> {
    table.iter().copied()
}

/// Whether a table holds `word` exactly.
pub fn has(table: &[&str], word: &str) -> bool {
    table.contains(&word)
}

/// Whether `w` is a word of the instrument's own vocabulary — a frame,
/// an absence word, a function word, a word of an English mark, or a
/// Chinese mark whole (a Chinese run is one word, so the mark itself
/// would otherwise be a name — and its own conjunction): none of these
/// spells a name, on either side of a change.
pub fn vocabulary(w: &str) -> bool {
    let t = v();
    [
        t.negations,
        t.en_prefix,
        t.en_suffix,
        t.zh_prefix,
        t.zh_suffix,
        t.marks_zh,
        t.stop_en,
    ]
    .iter()
    .any(|table| has(table, w))
        || entries(t.marks_en).any(|phrase| phrase.split(' ').any(|x| x == w))
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone/vocab.rs"]
mod tests;
