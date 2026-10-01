//! DocSpec and the docdup constants (design vol.2 §5, instruments
//! §9.6): every number written before any corpus number is seen. Since
//! plan v2.32 step 2 the numbers and the marker tables are the core's
//! (CE.Lang.Common.Prose `docdup`, read off `tables/1`, crate::tables): the
//! admission floor and verbatim floor (Lee et al. 2107.06499, 50
//! words), the five-word shingle, the 200-char prose line cap, the
//! five-line license window, the license markers, the skeleton line
//! prefixes and the inline allow marker. The numbers are the very
//! values CE.Docdup.Cost judges by, so the docdup reply's knob echo
//! still pins the two sides equal on every judged run. Marker tables
//! are language-independent; the per-language surface is the docstring
//! host list.

use crate::scan::lang::Lang;
use crate::tables::Docdup;
use crate::tables::leak::leaked;

/// The docdup numbers and marker tables (`docdup` in the package).
pub fn table() -> &'static Docdup {
    &crate::tables::get().docdup
}

/// Segment kinds as frozen position codes (the wire.rs edge-code
/// discipline: reordering is a DOCDUP_REV bump; `html_text` — a block
/// element's prose, docdup/html.rs — appended at rev 6, plan v2.30
/// step 5; `text_para` — a plain-text file's paragraph, segments.rs
/// text_paragraphs — at rev 8, step 5b-8). Their names are the package's
/// `kind_names`, in this order.
pub const KIND_MD_PARA: i64 = 0;
pub const KIND_COMMENT: i64 = 1;
pub const KIND_DOCSTRING: i64 = 2;
pub const KIND_HTML_TEXT: i64 = 3;
pub const KIND_TEXT_PARA: i64 = 4;

leaked! {
/// Per-language document spec: which AST node kinds host docstrings.
/// Only Python has a docstring convention (module/function/class body
/// whose first statement is a bare string); JSDoc and Rust `///` are
/// lexically comments and arrive via comment_kinds.
    pub struct DocSpec {
        pub docstring_hosts: &'static [&'static str],
    }
}

static NONE: DocSpec = DocSpec {
    docstring_hosts: &[],
};

pub fn doc_spec(lang: Lang) -> &'static DocSpec {
    table().doc_spec.get(lang).unwrap_or(&NONE)
}
