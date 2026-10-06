//! What stays of the Markdown rungs on this side (design §4 row 5).
//! Since plan v2.33 W2-text stage G the Markdown ladder is the core's
//! (CE.Resolve.Md: the link chain, the directory arm, the reference
//! substitution, the label fold, first-wins and the inert unused
//! definition); this side sends what it reads of the documents
//! (graph/resolve/markdown.rs). Three readings remain here because the
//! HTML rungs (html.rs) share them until stage H moves that ladder too —
//! the core holds its own spelling of each meanwhile (CE.Resolve.Md,
//! CE.Resolve.Url): R4 a bare fragment is an in-file section claim
//! taken AS WRITTEN, never validated; R2 a cross-file fragment,
//! percent-decoded, checked against the target's anchor set (md_slug.rs:
//! rendered-text slugs of the headings by GitHub rules plus raw-HTML
//! anchor ids verbatim), exactly one match naming the section and
//! anything else degrading to the file (ResolvedSection { slug: None });
//! and the URI-scheme test that sends a target out of the corpus before
//! any join runs.
//!
//! The anchor set is the rendered heading's slug (ATX and setext —
//! md_head.rs), the heading walk is block-aware, raw-HTML anchors enter
//! the set from a tag read across lines and attribute spellings, and
//! percent escapes decode (md_slug.rs). Cross-file staleness is closed
//! at the key: the ONLY target-content fact the anchor check consults is
//! the anchor set, so every md file's slug_hash is a resolve_key input —
//! a heading edit anywhere shifts the key and the phase-2 sweep
//! re-validates every anchor.

use super::{Outcome, Scope};
use slug::{percent_decode, slug_set};

#[path = "md_head.rs"]
pub(crate) mod head;
#[path = "md_slug.rs"]
pub(crate) mod slug;
pub use slug::slug_hash;

/// R4: a bare fragment names a section of the linking document, as
/// written — there is nothing to validate against (module header).
/// The HTML rungs read a bare fragment this way (html.rs).
pub(super) fn fragment(from: &str, frag: Option<&str>) -> Outcome {
    match frag {
        Some(f) if !f.is_empty() => Outcome::ResolvedSection {
            path: from.to_string(),
            slug: Some(f.to_string()),
            rung: 4,
        },
        _ => Outcome::Resolved {
            path: from.to_string(),
            rung: 4,
        },
    }
}

/// R2: the fragment, percent-decoded, against the target's anchor
/// set; anything but exactly one match degrades to the file (slug:
/// None). A page linking into a Markdown document asks here (html.rs).
pub(super) fn anchor(target: String, frag: &str, scope: &Scope) -> Outcome {
    // per-sweep: one read + slug pass per TARGET file, not one per
    // anchored link pointing at it (review MED)
    let slugs = scope.memo.cached("md_slugs", &target, || {
        let text = std::fs::read_to_string(scope.root.join(&target)).unwrap_or_default();
        slug_set(&text)
    });
    let frag = percent_decode(frag);
    let hits = slugs.iter().filter(|s| **s == frag).count();
    let slug = (hits == 1).then_some(frag);
    Outcome::ResolvedSection {
        path: target,
        slug,
        rung: 2,
    }
}

/// An RFC 3986 scheme head (or a protocol-relative // form) leaves
/// the corpus before any join runs — join_rel would silently
/// normalize "//host/x" into a bogus in-tree path. The HTML rungs
/// read it (html.rs).
pub(super) fn is_scheme(spec: &str) -> bool {
    if spec.starts_with("//") {
        return true;
    }
    let head = spec.split(['/', '#']).next().unwrap_or("");
    let Some((scheme, _)) = head.split_once(':') else {
        return false;
    };
    scheme
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/md_tests.rs"]
mod tests;
