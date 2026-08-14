//! Four-way category exemption (plan :78-80, design vol.2 §5.2) as a
//! LEDGERED filter — every shed line or segment lands in a counter,
//! never in silence (the low_diversity_suppressed discipline). Two of
//! the four routes are structurally zero here and say so where the
//! frozen docs state method: path exclusion never reaches the
//! extractor (walk::in_scope refuses the file first) and the baseline
//! exemption stock does not exist until `ce baseline` (3i).

use super::segments::{RawSeg, SegLine};
use super::spec::{
    ALLOW_MARKER, KIND_MD_PARA, LICENSE_HEAD_LINES, LICENSE_MARKERS, SKELETON_PREFIXES,
};

/// Exemption classes as frozen position codes; 0 = live.
pub const EXEMPT_NAMES: [&str; 3] = ["live", "license_header", "inline_allow"];
pub const EXEMPT_LIVE: i64 = 0;
pub const EXEMPT_LICENSE: i64 = 1;
pub const EXEMPT_ALLOW: i64 = 2;

/// Every count the extraction pipeline sheds anywhere.
#[derive(Default)]
pub struct Ledger {
    pub license_header: u64,
    pub skeleton_line: u64,
    pub inline_allow: u64,
    pub allow_missing_why: u64,
    pub below_floor: u64,
    pub indented_code_lines: u64,
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

/// Line-level skeleton strip for comment/docstring segments (plan
/// :79 "template rows" — the Google/Sphinx/JSDoc section vocabulary).
/// md paragraphs are untouched: a `---` there is a thematic break,
/// not a docstring underline. Returns the surviving lines.
pub fn strip_skeleton<'a>(seg: &'a RawSeg, ledger: &mut Ledger) -> Vec<&'a SegLine> {
    let (mut keep, mut stripped) = (Vec::new(), 0);
    for line in &seg.lines {
        if seg.kind != KIND_MD_PARA && skeleton_line(&line.text) {
            stripped += 1;
        } else {
            keep.push(line);
        }
    }
    ledger.skeleton_line += stripped;
    keep
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
}
