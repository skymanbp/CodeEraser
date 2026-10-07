//! What stays of the Markdown rungs on this side (design §4 row 5).
//! Since plan v2.33 W2-text stage G the Markdown ladder is the core's
//! (CE.Resolve.Md: the link chain, the directory arm, the reference
//! substitution, the label fold, first-wins and the inert unused
//! definition, the bare fragment taken as written, the cross-file anchor
//! check and the URI-scheme test, which the HTML rungs share there since
//! stage H); this side sends what it reads of the documents
//! (graph/resolve/markdown.rs): every walked Markdown file's anchor set.
//!
//! The anchor set is the rendered heading's slug (ATX and setext —
//! md_head.rs), the heading walk is block-aware, raw-HTML anchors enter
//! the set from a tag read across lines and attribute spellings
//! (md_slug.rs). Cross-file staleness is closed at the key: the ONLY
//! target-content fact the anchor check consults is the anchor set, so
//! every md file's slug_hash is a resolve_key input — a heading edit
//! anywhere shifts the key and the phase-2 sweep re-validates every
//! anchor.

#[path = "md_head.rs"]
pub(crate) mod head;
#[path = "md_slug.rs"]
pub(crate) mod slug;
pub use slug::slug_hash;

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/md_tests.rs"]
mod tests;
