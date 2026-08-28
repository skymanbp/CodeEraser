//! md.rs scanner tests — split out so the scanner file stays inside
//! the E01 300-line ceiling after the Opus-review hardening. Pins
//! the 2b RED conditions plus the review's counterexamples.

use super::{detect, masked_content_lines};

/// F3 (M5-3b): the docdup masking surface is the full triple mask —
/// fenced lines are ABSENT, HTML-comment bytes and inline-code bytes
/// are MASKED — in one additive call; the bare walk stays untouched
/// for the detector's own scan path.
#[test]
fn masked_content_lines_carry_the_triple_mask() {
    let text = "plain `code` tail\n```\nfenced\n```\n<!-- hidden --> shown\n";
    let rows = masked_content_lines(text);
    let lines: Vec<usize> = rows.iter().map(|(n, _, _)| *n).collect();
    assert_eq!(lines, vec![1, 5], "fence interior absent, content kept");
    let (_, l1, m1) = &rows[0];
    let code = l1.find("`code`").unwrap();
    assert!(m1[code] && m1[code + 5], "inline code masked");
    assert!(!m1[0], "plain text unmasked");
    let (_, l5, m5) = &rows[1];
    assert!(m5[0], "comment bytes masked");
    assert!(!m5[l5.find("shown").unwrap()], "text after comment live");
}

fn kinds_specs(text: &str) -> Vec<(&'static str, String)> {
    detect(text).into_iter().map(|s| (s.kind, s.spec)).collect()
}

/// Sub-milestone 2b RED condition: link-shaped strings inside a
/// fenced block or an inline code span must not emit a site.
#[test]
fn fences_and_code_spans_emit_nothing() {
    let text =
        "```md\n[fenced](./no.md)\n```\nsee `[coded](./no.md)` too\n~~~\n[tilde](./no.md)\n~~~\n";
    assert_eq!(kinds_specs(text), vec![]);
}

/// Indented code (step 8, O57): four columns at the document start or
/// after a blank line open a block outside a list — its link-shaped
/// content emits nothing and the block runs across blank lines; a
/// list item's continuation paragraph indented the same way is prose;
/// a lazy line without the blank continues its paragraph; a tab
/// counts four; a fence close leaves no paragraph open, so an
/// indented line right after it is code.
#[test]
fn indented_code_blocks_emit_nothing_outside_lists() {
    let text = "    [top](./no.md)\nintro\n\n    [code](./no.md)\n\n    [still](./no.md)\nback [a](./a.md)\n- item\n\n    [prose](./b.md)\npara\n    [lazy](./c.md)\n\n\t[tab](./no.md)\n```\nx\n```\n    [after](./no.md)\n";
    let specs: Vec<String> = kinds_specs(text).into_iter().map(|(_, s)| s).collect();
    assert_eq!(specs, ["./a.md", "./b.md", "./c.md"]);
}

/// Opus review: single-backtick pairing masked nothing inside a
/// ``double`` span — runs must pair by equal length (CommonMark).
#[test]
fn multi_backtick_spans_are_masked() {
    let text = "see ``[coded](./no.md)`` and ``` [also](./no.md) ``` here\n";
    assert_eq!(kinds_specs(text), vec![]);
}

/// A ``` inside an open ~~~ fence is content, not a closer — and so
/// is a ``` inside a ```` fence: only the same marker in a run at
/// least as long closes (CommonMark; the step-8 review's leak).
#[test]
fn mismatched_fence_markers_do_not_close() {
    for text in [
        "~~~\n```\n[still fenced](./no.md)\n```\n~~~\n[out](./yes.md)\n",
        "````\n```\n[still fenced](./no.md)\n```\n````\n[out](./yes.md)\n",
    ] {
        assert_eq!(kinds_specs(text), vec![("link", "./yes.md".into())]);
    }
}

/// Opus review: HTML comments (single- and multi-line) are not
/// content; link-shaped strings inside them must not emit sites.
#[test]
fn html_comments_emit_nothing() {
    let text = "<!-- [c1](./no.md) -->\n<!--\n[c2](./no.md)\n[id]: ./no.md\n-->\n[out](./yes.md) <!-- [c3](./no.md) -->\n";
    assert_eq!(kinds_specs(text), vec![("link", "./yes.md".into())]);
}

/// Opus review: a badge `[![alt](img)](url)` must emit BOTH the link
/// (url) and the image (img) — the first draft emitted one site
/// labelled link carrying the image target.
#[test]
fn badge_emits_link_and_image() {
    let text = "[![build](badge.svg)](https://ci.example/run)\n";
    assert_eq!(
        kinds_specs(text),
        vec![
            ("link", "https://ci.example/run".into()),
            ("image", "badge.svg".into()),
        ]
    );
}

#[test]
fn link_kinds_and_specs() {
    let text = "[a](./a.md) ![img](img.png) [b][ref]\n[ref]: ./b.md\n<https://x.example>\n[anchor](#sec)\n";
    assert_eq!(
        kinds_specs(text),
        vec![
            ("link", "./a.md".into()),
            ("image", "img.png".into()),
            ("ref_link", "ref".into()),
            ("ref_def", "./b.md".into()),
            ("url", "https://x.example".into()),
            ("link", "#sec".into()),
        ]
    );
}

/// ONE matched angle-bracket pair is stripped so the spec can match
/// scope.files (kept whole, `<guide.md#setup>` matched nothing and
/// lost the edge); an UNmatched `<` is no pair and stays verbatim.
#[test]
fn angle_bracket_destination_unwraps() {
    let text = "[x](<guide.md#setup>) [y](<unclosed)\n";
    let want = [("link", "guide.md#setup"), ("link", "<unclosed")];
    assert_eq!(kinds_specs(text), want.map(|(k, s)| (k, s.to_string())));
}

/// Anti-invention, STRICT (2b exit criterion): every spec is a
/// verbatim substring of its source line — including ref_def, whose
/// first draft synthesized a spec and weakened this very test.
#[test]
fn specs_are_line_substrings() {
    let text = "intro [a](./a.md) and <https://x.example>\n[id]: ./target.md \"title\"\n[![b](i.png)](./l.md)\n[x](<guide.md#setup>)\n";
    let lines: Vec<&str> = text.lines().collect();
    let found = detect(text);
    assert!(!found.is_empty());
    for site in found {
        let line = lines[site.line - 1];
        assert!(
            line.contains(&site.spec),
            "spec {:?} not in line {line:?}",
            site.spec
        );
    }
}
