//! Four-way category exemption (plan :74-76, design vol.2 §5.2) as a
//! LEDGERED filter — every shed line or segment lands in a counter,
//! never in silence (the low_diversity_suppressed discipline). Two of
//! the four routes are structurally zero here and say so where the
//! frozen docs state method: path exclusion never reaches the
//! extractor (walk::in_scope refuses the file first) and the baseline
//! exemption stock does not exist until `ce baseline` (3i).

use super::segments::{RawSeg, SegLine};
use super::spec::{KIND_HTML_TEXT, KIND_MD_PARA, KIND_TEXT_PARA, table};

/// Exemption classes as frozen position codes; 0 = live (the eval
/// instrument names them, tests it/eval_support/docseg.rs).
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
    let t = table();
    if first_comment && seg.start_line <= t.license_head_lines && has_any(seg, t.license_markers) {
        ledger.license_header += 1;
        return EXEMPT_LICENSE;
    }
    if has_any(seg, &[t.allow_marker]) {
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

/// Whether any line holds any marker of a table.
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
        .any(|l| crate::allow::allow_claim(&l.text, table().allow_marker))
}

/// Line-level strip for comment/docstring segments: skeleton rows
/// (plan :75 — the Google/Sphinx/JSDoc section vocabulary, and a
/// title underline, `title_underline`, which opens no fence), fenced
/// code regions (```/~~~ toggling, fence lines included — the F3
/// "the judge sees prose" contract extended to documentation text
/// wherever it lives) and overlong data/regex lines (`doc_line_cap`).
/// md paragraphs, HTML text and plain-text paragraphs (plan v2.30
/// step 5b-8) are untouched by ALL three: a `---` there is a thematic
/// break, md fences and HTML code were masked by the detector already,
/// plain text has no code to fence, and a single long line of any is
/// legitimate unwrapped prose (an HTML source line is masked markup
/// around it). Returns the surviving lines.
pub fn strip_skeleton<'a>(seg: &'a RawSeg, ledger: &mut Ledger) -> Vec<&'a SegLine> {
    let mut keep = Vec::new();
    let mut fenced = false;
    // the bare width of the line above; 0 when it was blank or shed by
    // a fence
    let mut above = 0;
    let cap = table().doc_line_cap;
    for line in &seg.lines {
        if matches!(seg.kind, KIND_MD_PARA | KIND_HTML_TEXT | KIND_TEXT_PARA) {
            keep.push(line);
            continue;
        }
        let width = bare_text(&line.text).chars().count();
        if !fenced && title_underline(&line.text, above) {
            ledger.skeleton_line += 1;
            above = width;
            continue;
        }
        let opens = fence_line(&line.text);
        if fenced || opens {
            // an unclosed fence honestly strips to segment end —
            // everything after ``` is code until proven otherwise
            ledger.fenced_code_line += 1;
            fenced = fenced != opens; // XOR: toggle on fence lines
            above = 0;
            continue;
        }
        above = width;
        if skeleton_line(&line.text) {
            ledger.skeleton_line += 1;
        } else if line.text.trim().chars().count() > cap {
            ledger.overlong_line += 1;
        } else {
            keep.push(line);
        }
    }
    keep
}

/// A fence marker line, matched after stripping the comment
/// decoration prefix (rustdoc `/// ```rust`, block-comment indents).
/// Asked only of a line that is not a title underline: outside a fence
/// `title_underline` reads first.
fn fence_line(text: &str) -> bool {
    let bare = text.trim().trim_start_matches(['#', '/', '*', '!', ' ']);
    bare.starts_with("```") || bare.starts_with("~~~")
}

/// A title underline (DOCDUP_REV 9): outside a fence, a line whose
/// bare text is only `~`, three or more, and at least as many
/// characters as the bare text of the line directly above it, that line
/// non-empty (`above` > 0). It is a skeleton row and opens no fence: a
/// docstring in the `requests.api` / `~~~~~~~~~~~~` style kept nothing
/// after its title while the underline read as an unclosed fence. A
/// tilde line shorter than the line above, or under a blank line, is a
/// fence marker as before; so is one inside a fence. Stated limit: a
/// tilde fence marker at least as long as the text line above it reads
/// as an underline.
fn title_underline(text: &str, above: usize) -> bool {
    let bare = bare_text(text);
    let width = bare.chars().count();
    above > 0 && width >= 3 && width >= above && bare.chars().all(|c| c == '~')
}

/// A line's bare text: the comment decoration prefix stripped (`#`,
/// `//`, `*`, `/*`, `!`, quotes for docstring openers, roxygen's `#'`)
/// and the end trimmed — what `skeleton_line` and `title_underline`
/// read.
fn bare_text(text: &str) -> &str {
    text.trim()
        .trim_start_matches(['#', '/', '*', '!', '"', '\'', ' '])
        .trim_end()
}

/// A skeleton line, matched on the bare text (`bare_text`), and after
/// the separator rule a leading `--` run, the Lua and Haskell comment
/// marker: a single `-` is a list bullet, and `- Note: ...` is prose,
/// not a template row.
fn skeleton_line(text: &str) -> bool {
    let bare = bare_text(text);
    if !bare.is_empty() && bare.chars().all(|c| c == '-') && bare.len() >= 3 {
        return true;
    }
    let bare = bare
        .strip_prefix("--")
        .map_or(bare, |rest| rest.trim_start_matches(['-', ' ']));
    table()
        .skeleton_prefixes
        .iter()
        .any(|p| bare.starts_with(p))
}

#[cfg(test)]
#[path = "../../tests/unit/docdup/exempt.rs"]
mod tests;
