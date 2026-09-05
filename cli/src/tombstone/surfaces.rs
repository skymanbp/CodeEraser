//! The two surfaces a change writes that the rule reads (spec §三 M3
//! and M4): the NAMING surface S⁺ — headings this change added, unit
//! names the after side declares and the before side did not, the
//! stem of a brand-new file — and the PROSE surface P⁺ — the comment,
//! docstring and paragraph segments docdup extracts, kept only when
//! they touch an added line. Both read `changed.added` off the four-
//! class diff, so a segment that already existed is never re-judged.
//!
//! Prose is read RAW (no docdup mask): the name a paragraph mentions
//! usually sits in backticks, and masking the span would hide exactly
//! the mention the conjunction needs. Fenced and indented code never
//! become segments, so an example's `(no X)` stays an example.

use super::PairText;
use super::frames::sentences;
use crate::docdup::segments::{self, RawSeg};
use crate::fourclass::{classify, units};
use crate::graph::ladder::md::slug::{atx_heading, render_text};
use crate::graph::md::content_lines;
use crate::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelKind {
    Heading,
    Unit,
    FileStem,
    /// The subject line of an after-only surface that is no file (a
    /// commit message, `subject`).
    Subject,
}

/// One naming surface this change wrote. `line` is on the after side
/// (0 = the file itself).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub line: usize,
    pub text: String,
    pub kind: LabelKind,
}

/// One sentence this change wrote into a prose segment; `start` /
/// `end` are the added lines it spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prose {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// The after-side lines this change added or rewrote (1-based), off
/// the same line diff the four-class family judges with — and whether
/// that diff was BOUNDED (fourclass::Classification::degraded: past
/// its caps every trimmed line counts as added, so the surfaces would
/// read untouched text as written). A leg records a degraded pair and
/// never enforces on it.
pub struct Added {
    pub lines: BTreeSet<usize>,
    pub degraded: bool,
}

pub fn added(pair: &PairText) -> Added {
    let c = classify(pair.before, pair.after, pair.lang);
    Added {
        lines: c.changed.added.into_iter().collect(),
        degraded: c.degraded,
    }
}

/// S⁺: Markdown reads its added headings (a section IS its heading,
/// so the unit path would say the same thing twice); code reads its
/// newly declared unit names; a new file adds its own stem.
pub fn labels(pair: &PairText, added: &BTreeSet<usize>) -> Vec<Label> {
    let mut out = Vec::new();
    if pair.lang == Lang::Markdown {
        headings(pair.after, added, &mut out);
    } else {
        new_units(pair, &mut out);
    }
    if pair.before.is_empty()
        && let Some(stem) = Path::new(pair.rel).file_stem().and_then(|s| s.to_str())
    {
        out.push(Label {
            line: 0,
            text: stem.to_string(),
            kind: LabelKind::FileStem,
        });
    }
    out
}

/// The label of an after-only surface that is no file — a commit
/// message: its subject line, the first non-blank one (a message has
/// no units, and the heading rule would read a `#` line as a heading).
pub fn subject(text: &str) -> Vec<Label> {
    text.lines()
        .enumerate()
        .find(|(_, l)| !l.trim().is_empty())
        .map(|(i, l)| Label {
            line: i + 1,
            text: l.trim().to_string(),
            kind: LabelKind::Subject,
        })
        .into_iter()
        .collect()
}

fn headings(after: &str, added: &BTreeSet<usize>, out: &mut Vec<Label>) {
    for (no, line, mask) in content_lines(after) {
        // a heading inside an HTML comment is markup, not a label
        if !added.contains(&no) || mask.first().copied().unwrap_or(false) {
            continue;
        }
        if let Some(h) = atx_heading(line.trim_start()) {
            out.push(Label {
                line: no,
                text: render_text(h),
                kind: LabelKind::Heading,
            });
        }
    }
}

fn new_units(pair: &PairText, out: &mut Vec<Label>) {
    let before: BTreeSet<String> = units::segments(pair.before, pair.lang)
        .into_iter()
        .map(|u| u.key)
        .collect();
    for u in units::segments(pair.after, pair.lang) {
        if !before.contains(&u.key) {
            out.push(Label {
                line: u.start_line,
                text: super::marked::name_part(&u.key).to_string(),
                kind: LabelKind::Unit,
            });
        }
    }
}

/// P⁺: the SENTENCES this change wrote into every docdup segment of
/// the after side. Boundaries are cut in the WHOLE segment text (an
/// unchanged terminator between two added lines keeps them two
/// sentences), and a sentence is kept when an added line is among its
/// lines — what this change wrote, not the
/// paragraph it touched: the first self-history replay fired 219
/// times, mostly on a version bump inside a list whose OTHER lines
/// said `此前`, and a mark that stood there before this change is
/// nobody's residue. No admission floor (that floor serves the shingle
/// judge, and a one-line `X is no longer needed` is a whole
/// tombstone); `start` is the sentence's first added line, where a
/// reader should look.
pub fn prose(pair: &PairText, added: &BTreeSet<usize>) -> Vec<Prose> {
    let (segs, _) = segments::extract(pair.after, pair.lang);
    segs.iter().flat_map(|s| mine(s, added)).collect()
}

/// One segment's touched sentences: its lines joined by a space, each
/// line's byte span remembered, the sentences cut in that text and
/// attributed back to the lines they overlap.
fn mine(seg: &RawSeg, added: &BTreeSet<usize>) -> Vec<Prose> {
    let Ok(first) = usize::try_from(seg.start_line) else {
        return Vec::new();
    };
    let (mut text, mut spans) = (String::new(), Vec::new());
    for (i, l) in seg.lines.iter().enumerate() {
        if i > 0 {
            text.push(' ');
        }
        let at = text.len();
        text.push_str(&l.text);
        spans.push((first + i, at, text.len()));
    }
    sentences(&text)
        .into_iter()
        .filter_map(|s| {
            let at = s.as_ptr() as usize - text.as_ptr() as usize;
            let end = at + s.len();
            let mut touched = spans
                .iter()
                .filter(|(n, a, b)| *a < end && at < *b && added.contains(n))
                .map(|(n, _, _)| *n);
            let start = touched.next()?;
            Some(Prose {
                start,
                end: touched.next_back().unwrap_or(start),
                text: s.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone/surfaces.rs"]
mod tests;
