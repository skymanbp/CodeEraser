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
}

/// One naming surface this change wrote. `line` is on the after side
/// (0 = the file itself).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub line: usize,
    pub text: String,
    pub kind: LabelKind,
}

/// One prose segment this change touched, raw lines joined by a space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prose {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// The after-side lines this change added or rewrote (1-based), off
/// the same line diff the four-class family judges with.
pub fn added_lines(pair: &PairText) -> BTreeSet<usize> {
    classify(pair.before, pair.after, pair.lang)
        .changed
        .added
        .into_iter()
        .collect()
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

/// P⁺: the ADDED lines of every docdup segment of the after side —
/// what this change wrote, not the paragraph it touched: the first
/// self-history replay fired 219 times, mostly on a version bump
/// inside a list whose OTHER lines said `此前`, and a mark that stood
/// there before this change is nobody's residue. No admission floor
/// (that floor serves the shingle judge, and a one-line `X is no
/// longer needed` is a whole tombstone); `start` is the first added
/// line, where a reader should look.
pub fn prose(pair: &PairText, added: &BTreeSet<usize>) -> Vec<Prose> {
    let (segs, _) = segments::extract(pair.after, pair.lang);
    segs.iter().filter_map(|s| mine(s, added)).collect()
}

/// One segment's added lines, or None when it has none.
fn mine(seg: &RawSeg, added: &BTreeSet<usize>) -> Option<Prose> {
    let lines: Vec<(usize, &str)> = seg
        .lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            let n = usize::try_from(seg.start_line).ok()? + i;
            added.contains(&n).then_some((n, l.text.as_str()))
        })
        .collect();
    let (start, _) = *lines.first()?;
    let (end, _) = *lines.last()?;
    Some(Prose {
        start,
        end,
        text: lines.iter().map(|(_, t)| *t).collect::<Vec<_>>().join(" "),
    })
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone/surfaces.rs"]
mod tests;
