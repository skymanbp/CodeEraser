//! The 3d exit criterion, mechanized: text inside fences, inline
//! code and HTML comments yields ZERO segment words, and the
//! extractor demonstrably rides the detector's one mask.

use super::*;
use crate::docdup::shingle;

fn md_segs(text: &str) -> Vec<RawSeg> {
    extract(text, Lang::Markdown).0
}

fn seg_words(seg: &RawSeg) -> Vec<u64> {
    let mut out = Vec::new();
    for l in &seg.lines {
        shingle::line_words(&l.text, l.mask.as_deref(), &mut out);
    }
    out
}

/// (kind, start, end) rows of one source — the assertion currency of
/// every geometry test below.
fn rows(src: &str, lang: Lang) -> Vec<(i64, i64, i64)> {
    extract(src, lang)
        .0
        .iter()
        .map(|s| (s.kind, s.start_line, s.end_line))
        .collect()
}

#[test]
fn fenced_inline_and_html_comment_text_is_invisible() {
    let text = "para alpha beta\n\n```\nfence secret\n```\n\ngamma `inline secret` delta\n\n<!-- comment secret -->\n";
    let segs = md_segs(text);
    let all: Vec<u64> = segs.iter().flat_map(seg_words).collect();
    let mut secret = Vec::new();
    shingle::line_words("secret", None, &mut secret);
    assert!(!all.contains(&secret[0]), "masked text leaked into words");
    // the fence body and the pure-comment line form no segment at all
    assert_eq!(segs.len(), 2, "alpha/beta and gamma/delta paragraphs");
}

#[test]
fn headings_tables_and_bare_markers_break_paragraphs() {
    let text = "one two\n# Head\nthree four\n| a | b |\nfive six\n-\nseven eight\n- item text\n";
    let bounds: Vec<(i64, i64)> = md_segs(text)
        .iter()
        .map(|s| (s.start_line, s.end_line))
        .collect();
    // each excluded line severs adjacency; the list item WITH text is
    // content and joins the preceding run
    assert_eq!(bounds, [(1, 1), (3, 3), (5, 5), (7, 8)]);
}

#[test]
fn comment_runs_merge_by_column_and_docstrings_extract() {
    let py = "\"\"\"module doc\nsecond line\n\"\"\"\n# one\n# two\nx = 1\n# lone\n\ndef f():\n    \"\"\"fn doc\"\"\"\n    return 1  # trailing\n";
    assert_eq!(
        rows(py, Lang::Python),
        [
            (KIND_DOCSTRING, 1, 3),   // module docstring
            (KIND_COMMENT, 4, 5),     // merged # run
            (KIND_COMMENT, 7, 7),     // lone comment
            (KIND_DOCSTRING, 10, 10), // fn docstring
            (KIND_COMMENT, 11, 11),   // trailing comment
        ]
    );
}

#[test]
fn rust_block_and_line_comments_extract() {
    let rs = "/* block\n   comment */\nfn main() {\n    // a\n    // b\n}\n";
    assert_eq!(
        rows(rs, Lang::Rust),
        [(KIND_COMMENT, 1, 2), (KIND_COMMENT, 4, 5)]
    );
}
