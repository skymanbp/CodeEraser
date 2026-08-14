//! DocSpec and the docdup constants (design vol.2 §5, instruments
//! §9.6): every number written before any corpus number is seen.
//! Marker tables are language-independent; the per-language surface
//! is the docstring host list.

use crate::scan::lang::Lang;

/// Admission floor in words. Provenance: plan :68 (Lee et al.
/// 2107.06499, verbatim lower bound 50 tokens).
pub const MIN_DOC_TOKENS: usize = 50;

/// License-header window: only a file's FIRST comment block starting
/// at or above this line can be a license header. Decided.
pub const LICENSE_HEAD_LINES: i64 = 5;

/// Verbatim hard-hit floor in words (same provenance as
/// MIN_DOC_TOKENS). A common word run this long is a duplicate
/// regardless of what Jaccard says about the rest of the segment.
pub const VERBATIM_FLOOR: usize = 50;

/// Word-shingle width. Decided, not derived (instruments §9.6): the
/// measured k-window and its counterfactual live in the frozen
/// docdup-segments doc's method line — the constant's backing is a
/// generation-time measurement, not a remembered citation.
pub const DOC_SHINGLE: usize = 5;

/// Longest comment/docstring line still treated as prose, in visible
/// chars. Decided, not derived (2026-08-14 attainment-line-B
/// amendment, ccm #842): hard-wrapped comment prose runs under ~120
/// chars by convention, while the audited FP lines — regex literals
/// and inline snapshots — ran 300+/600+; lines past this cap are
/// data/generated/code, masked with an overlong_line ledger count.
/// md_para is exempt: markdown legitimately writes one long prose
/// line per paragraph.
pub const DOC_LINE_CAP: usize = 200;

/// Segment kinds as frozen position codes (the wire.rs edge-code
/// discipline: reordering is a DOCDUP_REV bump).
pub const KIND_NAMES: [&str; 3] = ["md_para", "comment_block", "docstring"];
pub const KIND_MD_PARA: i64 = 0;
pub const KIND_COMMENT: i64 = 1;
pub const KIND_DOCSTRING: i64 = 2;

/// License-header markers (design vol.2 §5.2). Any one on any line of
/// the first comment block inside the head window exempts the block.
pub const LICENSE_MARKERS: [&str; 5] = [
    "SPDX-License-Identifier",
    "Licensed under the Apache License",
    "Copyright (c)",
    "Permission is hereby granted",
    "MIT License",
];

/// Structured-docstring skeleton line prefixes (plan :79 "template
/// rows", stripped line-level from comment/docstring segments — the
/// Google/Sphinx/NumPy/JSDoc section vocabulary, not prose).
pub const SKELETON_PREFIXES: [&str; 16] = [
    "Args:",
    "Arguments:",
    "Returns:",
    "Raises:",
    "Yields:",
    "Parameters",
    "Attributes:",
    "Example:",
    "Examples:",
    "Note:",
    ":param ",
    ":return",
    ":rtype",
    "@param",
    "@returns",
    "@throws",
];

/// The inline exemption marker (plan :79-80). Without a ` -- <why>`
/// tail it exempts NOTHING — a bare marker is a violation, ledgered.
pub const ALLOW_MARKER: &str = "ce:allow(docdup)";

/// Per-language document spec: which AST node kinds host docstrings.
/// Only Python has a docstring convention (module/function/class body
/// whose first statement is a bare string); JSDoc and Rust `///` are
/// lexically comments and arrive via comment_kinds.
pub struct DocSpec {
    pub docstring_hosts: &'static [&'static str],
}

static PYTHON: DocSpec = DocSpec {
    docstring_hosts: &["module", "function_definition", "class_definition"],
};
static NONE: DocSpec = DocSpec {
    docstring_hosts: &[],
};

pub fn doc_spec(lang: Lang) -> &'static DocSpec {
    match lang {
        Lang::Python => &PYTHON,
        _ => &NONE,
    }
}
