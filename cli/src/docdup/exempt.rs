//! Four-way category exemption (plan :74-76, design vol.2 §5.2) as a
//! LEDGERED filter — every shed line or segment lands in a counter,
//! never in silence (the low_diversity_suppressed discipline). Two of
//! the four routes are structurally zero here and say so where the
//! frozen docs state method: path exclusion never reaches the
//! extractor (walk::in_scope refuses the file first) and the baseline
//! exemption stock does not exist until `ce baseline` (3i).

use super::segments::{RawSeg, SegLine};
use super::spec::{
    ALLOW_MARKER, DOC_LINE_CAP, KIND_MD_PARA, LICENSE_HEAD_LINES, LICENSE_MARKERS,
    SKELETON_PREFIXES,
};

/// Exemption classes as frozen position codes; 0 = live.
pub const EXEMPT_NAMES: [&str; 3] = ["live", "license_header", "inline_allow"];
pub const EXEMPT_LIVE: i64 = 0;
pub const EXEMPT_LICENSE: i64 = 1;
pub const EXEMPT_ALLOW: i64 = 2;

/// Every count the extraction pipeline sheds anywhere. The md walk's
/// own shed counters ride embedded (segments::MdShed — one owner, no
/// field copying); fenced_code_line and overlong_line are the
/// comment-side rows of the 2026-08-14 attainment-line-B amendment
/// (ccm #842): embedded code inside documentation is not prose.
#[derive(Default)]
pub struct Ledger {
    pub license_header: u64,
    pub skeleton_line: u64,
    pub md: super::segments::MdShed,
    pub inline_allow: u64,
    pub allow_missing_why: u64,
    pub below_floor: u64,
    pub fenced_code_line: u64,
    pub overlong_line: u64,
}

/// The exemption class of one admitted segment. License first (the
/// narrower claim: only a file's FIRST comment block inside the head
/// window), then the explicit inline allow. The window number's
/// authority is CE.Docdup.Cost.licHeadLines (echo-pinned via
/// spec.rs); the bare-marker rule's authority is the same module's
/// written ruling (batch-7 slice 9).
pub fn classify(seg: &RawSeg, first_comment: bool, ledger: &mut Ledger) -> i64 {
    if first_comment && seg.start_line <= LICENSE_HEAD_LINES && has_any(seg, &LICENSE_MARKERS) {
        ledger.license_header += 1;
        return EXEMPT_LICENSE;
    }
    if has_any(seg, &[ALLOW_MARKER]) {
        if allow_has_why(seg) {
            ledger.inline_allow += 1;
            return EXEMPT_ALLOW;
        }
        // a bare marker exempts NOTHING (plan :76: no why = a
        // violation); the segment stays live and the ledger says so
        ledger.allow_missing_why += 1;
    }
    EXEMPT_LIVE
}

fn has_any(seg: &RawSeg, markers: &[&str]) -> bool {
    seg.lines
        .iter()
        .any(|l| markers.iter().any(|m| l.text.contains(m)))
}

/// The one claim grammar (crate::allow), line by line: the why must
/// share the marker's line.
fn allow_has_why(seg: &RawSeg) -> bool {
    seg.lines
        .iter()
        .any(|l| crate::allow::allow_claim(&l.text, ALLOW_MARKER))
}

/// Line-level strip for comment/docstring segments: skeleton rows
/// (plan :75 — the Google/Sphinx/JSDoc section vocabulary), fenced
/// code regions (```/~~~ toggling, fence lines included — the F3
/// "the judge sees prose" contract extended to documentation text
/// wherever it lives) and overlong data/regex lines (DOC_LINE_CAP).
/// md paragraphs are untouched by ALL three: a `---` there is a
/// thematic break, md fences were masked by the detector already,
/// and a single long md line is legitimate unwrapped prose. Returns
/// the surviving lines.
pub fn strip_skeleton<'a>(seg: &'a RawSeg, ledger: &mut Ledger) -> Vec<&'a SegLine> {
    let mut keep = Vec::new();
    let mut fenced = false;
    for line in &seg.lines {
        if seg.kind == KIND_MD_PARA {
            keep.push(line);
            continue;
        }
        let opens = fence_line(&line.text);
        if fenced || opens {
            // an unclosed fence honestly strips to segment end —
            // everything after ``` is code until proven otherwise
            ledger.fenced_code_line += 1;
            fenced = fenced != opens; // XOR: toggle on fence lines
        } else if skeleton_line(&line.text) {
            ledger.skeleton_line += 1;
        } else if line.text.trim().chars().count() > DOC_LINE_CAP {
            ledger.overlong_line += 1;
        } else {
            keep.push(line);
        }
    }
    keep
}

/// A fence marker line, matched after stripping the comment
/// decoration prefix (rustdoc `/// ```rust`, block-comment indents).
fn fence_line(text: &str) -> bool {
    let bare = text.trim().trim_start_matches(['#', '/', '*', '!', ' ']);
    bare.starts_with("```") || bare.starts_with("~~~")
}

/// A skeleton line, matched after stripping the comment decoration
/// prefix (`#`, `//`, `*`, `/*`, `!`, quotes for docstring openers).
fn skeleton_line(text: &str) -> bool {
    let bare = text
        .trim()
        .trim_start_matches(['#', '/', '*', '!', '"', '\'', ' '])
        .trim_end();
    if !bare.is_empty() && bare.chars().all(|c| c == '-') && bare.len() >= 3 {
        return true;
    }
    SKELETON_PREFIXES.iter().any(|p| bare.starts_with(p))
}

#[cfg(test)]
#[path = "../../tests/unit/docdup/exempt.rs"]
mod tests;
