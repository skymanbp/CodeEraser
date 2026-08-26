//! Markdown-ladder fixtures (design §6 2f exit row). The doc rungs
//! read document CONTENT — target headings for R2 slugs, the linking
//! file's reference definitions for R3 — so this habitat carries
//! real bodies where graph_ladder.rs's import-shaped tree carries
//! empty files. Ambiguity fixtures MUST stay degraded or refused: a
//! collision anchor that lands a section edge, or a fenced
//! definition that resolves anything, is the red condition.

use codeeraser::graph::ladder::{Outcome, Reason};
use codeeraser::scan::lang::Lang;

use crate::common::{Case, ext, fixture, no, ok, pkg, run_cases, sec, secf};

const TREE: &[(&str, &str)] = &[
    (
        "README.md",
        "# Readme\n\nSee [the guide](./guide.md) and [source](./src/).\n",
    ),
    // slugs: guide, setup, dup, dup-1, parse, 문자열 — the fenced and
    // commented headings must NOT join the set
    (
        "guide.md",
        "# Guide\n## Setup\n## Dup\n## Dup\n### `.parse`\n## 문자열\n<!-- ## Hidden -->\n\n```md\n# Fenced Heading\n```\n",
    ),
    // twin, twin-1 — then "Twin 1" ALSO slugs to twin-1: the -N
    // suffix rule's own collision, a slug set that is not 1:1
    ("twins.md", "## Twin\n## Twin\n## Twin 1\n"),
    // one used definition (twice, case-folded), one unused, one
    // undefined use, and two definitions the masking must erase
    (
        "docs/index.md",
        "# Index\n\nSee [g][spec], [g2][SPEC] and [missing][ghost].\n\n[spec]: ../guide.md\n[dead]: ./unref.md\n[idle]: ../src/app.py\n\n```text\n[fence]: ../guide.md\n```\n\n<!-- [cmt]: ../guide.md -->\n",
    ),
    ("src/app.py", "x = 1\n"),
    ("assets/logo.svg", "<svg/>\n"),
];

/// Aliases keep every row on one line (the table IS the spec). Block
/// order is R4/R5 first, R1 last: rung order is evaluation order,
/// not table order, and opening with the claim/exclusion shapes
/// keeps this table's head frame from aligning with the import
/// tables' resolve-first frames (the dedup ratchet caught exactly
/// that pairing against go_cases).
fn md_cases() -> Vec<Case> {
    let (m, ln, im) = (Lang::Markdown, "link", "image");
    let (rm, gd, ix) = ("README.md", "guide.md", "docs/index.md");
    vec![
        // R4: a bare fragment is a claim taken as written — README
        // has no such ATX heading and the section edge still lands
        // (raw-HTML anchors are legal targets we do not model)
        (m, ln, rm, "#raw-html-anchor", sec(rm, "raw-html-anchor", 4)),
        // R5: schemes leave the corpus; a relative asset is outside
        // the modeled file set, never a guessed edge
        (m, ln, rm, "https://github.com/x/y", ext(5)),
        (m, im, rm, "https://img.shields.io/b.svg", ext(5)),
        (m, im, rm, "./assets/logo.svg", no(Reason::OutOfScope)),
        (m, "url", rm, "mailto:a@b.example", ext(5)),
        // R2: unique slug, the -1 duplicate suffix, the audited
        // punctuation-strip and Hangul rows — then four degrades:
        // no such slug, the twin collision, fenced, commented
        (m, ln, rm, "./guide.md#setup", sec(gd, "setup", 2)),
        (m, ln, rm, "./guide.md#dup-1", sec(gd, "dup-1", 2)),
        (m, ln, rm, "./guide.md#parse", sec(gd, "parse", 2)),
        (m, ln, rm, "./guide.md#문자열", sec(gd, "문자열", 2)),
        (m, ln, rm, "./guide.md#nope", secf(gd, 2)),
        (m, ln, rm, "./twins.md#twin-1", secf("twins.md", 2)),
        (m, ln, rm, "./guide.md#fenced-heading", secf(gd, 2)),
        (m, ln, rm, "./guide.md#hidden", secf(gd, 2)),
        // R3: substitution (case-folded), undefined, and the two
        // masked definitions that must not exist; a used definition
        // site resolves its target, an unused one travels INERT
        (m, "ref_link", ix, "spec", ok(gd, 3)),
        (m, "ref_link", ix, "SPEC", ok(gd, 3)),
        (m, "ref_link", ix, "ghost", no(Reason::OutOfScope)),
        (m, "ref_link", ix, "fence", no(Reason::OutOfScope)),
        (m, "ref_link", ix, "cmt", no(Reason::OutOfScope)),
        (m, "ref_def", ix, "../guide.md", ok(gd, 3)),
        // an unresolvable unused definition is an ordinary miss now
        // (2.29.0) — the borrowed External category retired
        (m, "ref_def", ix, "./unref.md", no(Reason::OutOfScope)),
        // the VERDICT-relevant unused case (batch-7 slice 16,
        // executed at H1): the in-scope target resolves and travels
        // INERT — the core owns the liveness exclusion, and this row
        // pins the channel end to end
        (m, "ref_def", ix, "../src/app.py", inert("src/app.py", 3)),
        // R1: doc→doc, doc→code (a #L fragment on a code target is a
        // view detail, dropped), a directory reference (the audited
        // fourclass row), a miss, and the site-root refusal
        (m, ln, rm, "./guide.md", ok(gd, 1)),
        (m, ln, ix, "../guide.md", ok(gd, 1)),
        (m, ln, ix, "../src/app.py", ok("src/app.py", 1)),
        (m, ln, ix, "../src/app.py#L7", ok("src/app.py", 1)),
        (m, ln, rm, "./src/", pkg("src", 1)),
        (m, ln, rm, "./missing.md", no(Reason::OutOfScope)),
        (m, ln, rm, "/guide.md", no(Reason::OutOfScope)),
    ]
}

/// The inert outcome (H1 slice 16) — single consumer, so the alias
/// lives here rather than in the shared helper set.
fn inert(path: &str, rung: u8) -> Outcome {
    Outcome::ResolvedInert {
        path: path.into(),
        rung,
    }
}

#[test]
fn doc_rungs_resolve_and_refuse() {
    let fx = fixture("ladder-md", TREE);
    run_cases(&fx, md_cases());
}
