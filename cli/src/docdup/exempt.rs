//! Four-way category exemption (plan :78-80, design vol.2 §5.2) as a
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
/// window), then the explicit inline allow.
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
        // a bare marker exempts NOTHING (plan :79-80: no why = a
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

fn allow_has_why(seg: &RawSeg) -> bool {
    seg.lines.iter().any(|l| {
        l.text
            .split_once(ALLOW_MARKER)
            .and_then(|(_, tail)| tail.split_once("--"))
            .is_some_and(|(_, why)| !why.trim().is_empty())
    })
}

/// Line-level strip for comment/docstring segments: skeleton rows
/// (plan :79 — the Google/Sphinx/JSDoc section vocabulary), fenced
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
mod tests {
    use super::*;
    use crate::docdup::spec::KIND_DOCSTRING;

    fn seg(kind: i64, start: i64, lines: &[&str]) -> RawSeg {
        RawSeg {
            kind,
            start_line: start,
            end_line: start + lines.len() as i64 - 1,
            lines: lines
                .iter()
                .map(|t| SegLine {
                    text: (*t).to_string(),
                    mask: None,
                })
                .collect(),
        }
    }

    #[test]
    fn license_needs_first_block_and_head_window() {
        let lic = seg(1, 1, &["// Licensed under the Apache License"]);
        let mut lg = Ledger::default();
        assert_eq!(classify(&lic, true, &mut lg), EXEMPT_LICENSE);
        assert_eq!(classify(&lic, false, &mut lg), EXEMPT_LIVE);
        let late = seg(1, 40, &["// Licensed under the Apache License"]);
        assert_eq!(classify(&late, true, &mut lg), EXEMPT_LIVE);
        assert_eq!(lg.license_header, 1);
    }

    #[test]
    fn allow_without_why_is_a_ledgered_violation_not_an_exemption() {
        let mut lg = Ledger::default();
        let ok = seg(1, 10, &["# ce:allow(docdup) -- generated table"]);
        assert_eq!(classify(&ok, false, &mut lg), EXEMPT_ALLOW);
        let bare = seg(1, 10, &["# ce:allow(docdup)"]);
        assert_eq!(classify(&bare, false, &mut lg), EXEMPT_LIVE);
        let empty_why = seg(1, 10, &["# ce:allow(docdup) -- "]);
        assert_eq!(classify(&empty_why, false, &mut lg), EXEMPT_LIVE);
        assert_eq!((lg.inline_allow, lg.allow_missing_why), (1, 2));
    }

    #[test]
    fn skeleton_lines_strip_from_docstrings_but_not_md() {
        let mut lg = Ledger::default();
        let ds = seg(
            KIND_DOCSTRING,
            1,
            &[
                "\"\"\"Fetch.",
                "Args:",
                "    x: input",
                "Returns:",
                "\"\"\"",
            ],
        );
        let kept = strip_skeleton(&ds, &mut lg);
        assert_eq!(kept.len(), 3);
        assert_eq!(lg.skeleton_line, 2);
        let md = seg(KIND_MD_PARA, 1, &["Args:", "---"]);
        assert_eq!(strip_skeleton(&md, &mut lg).len(), 2);
        assert_eq!(lg.skeleton_line, 2, "md untouched");
    }

    #[test]
    fn jsdoc_and_sphinx_markers_match_under_decoration() {
        for line in [" * @param x the input", "    :param x: input", "# Returns:"] {
            assert!(skeleton_line(line), "{line}");
        }
        assert!(!skeleton_line("returns the cached value"));
        assert!(skeleton_line(" * ----"));
    }

    /// Seeded counterfactual for the 2026-08-14 amendment's comment
    /// half: a rustdoc fenced doctest (the audited ripgrep FP shape)
    /// and an overlong regex line (the audited zod FP shape) strip
    /// with their ledger counts; prose around them survives; an
    /// UNCLOSED fence strips to segment end.
    #[test]
    fn fenced_and_overlong_comment_lines_strip_with_ledger() {
        let mut lg = Ledger::default();
        let long = format!("// const rx = /{}/;", "a|".repeat(150));
        let c = seg(
            1,
            10,
            &[
                "/// This crate provides printers.",
                "/// ```rust",
                "/// let x = search();",
                "/// ```",
                "/// More prose after the example.",
                &long,
            ],
        );
        let kept = strip_skeleton(&c, &mut lg);
        assert_eq!(kept.len(), 2, "prose survives, code and regex do not");
        assert_eq!(lg.fenced_code_line, 3);
        assert_eq!(lg.overlong_line, 1);
        let unclosed = seg(1, 1, &["// prose", "// ```", "// code one", "// code two"]);
        assert_eq!(strip_skeleton(&unclosed, &mut lg).len(), 1);
        assert_eq!(lg.fenced_code_line, 6, "unclosed fence strips to end");
        let md = seg(KIND_MD_PARA, 1, &[&long]);
        assert_eq!(strip_skeleton(&md, &mut lg).len(), 1, "md untouched");
    }
}
