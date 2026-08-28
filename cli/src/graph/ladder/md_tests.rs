//! slug_hash ↔ slug_set coupling battery (the M5-close staleness
//! repayment's one unpinned invariant): the hashed projection and
//! the consulted projection must MOVE TOGETHER — a refactor that
//! hashes the raw text, the unmasked text, or a deduplicated set
//! keeps every e2e green while quietly re-opening (or degrading)
//! the cross-file staleness repayment. Their divergence is the only
//! way that debt returns, so it is pinned here as a table.

use super::slug::{percent_decode, slug_hash, slug_set};

#[test]
fn hash_moves_exactly_when_the_consulted_projection_moves() {
    let pairs: &[(&str, &str, &str)] = &[
        ("body-only edit holds the hash", "# A\nx\n", "# A\ny\n"),
        ("heading edit moves it", "# Alpha\nbody\n", "# Beta\nbody\n"),
        (
            "fenced pseudo-headings are masked on both sides",
            "# A\n```\n# Hidden\n```\n",
            "# A\n```\n# Other\n```\n",
        ),
        ("a duplicate heading enters as -N", "# A\n", "# A\n# A\n"),
        (
            "an html anchor is part of the consulted set",
            "# A\n",
            "# A\n<a name=\"x\"></a>\n",
        ),
        (
            "link syntax in a heading is not: the rendered text is",
            "# [T](./x.md)\n",
            "# T\n",
        ),
    ];
    for (why, a, b) in pairs {
        assert_eq!(
            slug_hash(a) == slug_hash(b),
            slug_set(a) == slug_set(b),
            "projection coupling broke: {why}"
        );
    }
    assert_eq!(slug_set("# A\n# A\n"), ["a", "a-1"], "the -N suffix rule");
}

/// Step 8 (O57): the slug is the RENDERED heading's (code spans,
/// emphasis, links, images, inline HTML and comments, escapes), raw-HTML
/// anchors enter verbatim — on a heading line too, whose own slug drops
/// the tag — an anchor inside a code span does not, indented code
/// offers no heading, a commented anchor is masked, a tab after the
/// hashes still opens a heading, an unpaired `_` renders, and a
/// fragment percent-decodes (a bad escape or a non-UTF-8 result stays
/// as written). One document carries every heading row, so the -N
/// counter is exercised by two rendering to the same slug.
#[test]
fn rendered_slugs_html_anchors_and_percent_decoding() {
    let doc = "# `ce scan` runs\n\
               # **Bold** _em_ snake_case __x__\n\
               # [Text](./x.md) and ![alt](i.png) [ref][id]\n\
               # <code>x</code> \\# y\n\
               # a < b ``ti`ck``\n\
               # _private_helper()\n\
               # foo_\n\
               #\tTabbed\n\
               ## Setup <!-- internal -->\n\
               # 中文 标题\n\
               ## Title <a name=\"top\"></a>\n\
               # `ce scan` runs\n\
               <h3 id='deep'>D</h3>\n\
               <a id=\"self\"/>\n\
               see `<a id=\"coded\"></a>` in code\n\
               <div id=\"not-an-anchor\">\n\
               \n    # not a heading\n\
               <!-- <a name=\"hidden\"> -->\n";
    let want = "ce-scan-runs bold-em-snake_case-x text-and-alt-ref x--y a--b-tick \
                _private_helper foo_ tabbed setup 中文-标题 title top ce-scan-runs-1 deep self";
    assert_eq!(slug_set(doc).join(" "), want);
    assert_eq!(percent_decode("%E4%B8%AD-x%2"), "中-x%2");
    assert_eq!(percent_decode("bad%ZZ%ff"), "bad%ZZ%ff");
}
