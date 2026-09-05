//! Doc attribution for the bag's D channel: docdup's own comment /
//! docstring segments, wordized through the term road, each owned by
//! at most one unit (`owner`). Split from bag.rs at the file budget.

use super::terms;
use crate::docdup::{exempt, segments, spec as docspec};
use crate::fourclass::units::Unit;
use crate::scan::lang::Lang;

/// A leading comment may end this many lines above the unit's first
/// line (attributes / decorators sit between); a docstring or header
/// comment may open this many lines below it.
pub const LEAD_GAP: usize = 3;
pub const HEAD_GAP: usize = 2;

/// One attributable comment / docstring: its kind, span, prose words,
/// and whether it is a Rust inner doc (`//!` — the module's, never a
/// unit's).
pub struct DocSeg {
    docstring: bool,
    inner: bool,
    start: usize,
    end: usize,
    pub words: Vec<String>,
}

/// Every comment-block and docstring segment of the file (docdup's
/// own extraction, skeleton rows stripped the way docdup strips
/// them), wordized through the term road.
pub fn doc_segments(text: &str, lang: Lang) -> Vec<DocSeg> {
    let (raw, _) = segments::extract(text, lang);
    let mut ledger = exempt::Ledger::default();
    raw.iter()
        .filter(|s| s.kind != docspec::KIND_MD_PARA)
        .map(|s| DocSeg {
            docstring: s.kind == docspec::KIND_DOCSTRING,
            inner: s
                .lines
                .first()
                .is_some_and(|l| l.text.trim_start().starts_with("//!")),
            start: s.start_line as usize,
            end: s.end_line as usize,
            words: exempt::strip_skeleton(s, &mut ledger)
                .iter()
                .flat_map(|l| terms::prose_words(&l.text))
                .collect(),
        })
        .collect()
}

/// The unit a doc segment belongs to (a seat into `all`), by kind: a
/// DOCSTRING is the head of the innermost unit it opens (its first
/// line within HEAD_GAP of the unit's) — a class's is the class's,
/// a module's is nobody's; a COMMENT leads the nearest unit that
/// starts within LEAD_GAP below it (attributes / decorators may sit
/// between) inside the same container, and failing that heads the
/// unit it opens (a header comment at the top of a body). `//!` is
/// the module's and never attributed.
pub fn doc_owner(d: &DocSeg, all: &[Unit]) -> Option<usize> {
    if d.inner {
        return None;
    }
    let contains = |u: &Unit| u.start_line <= d.start && d.end <= u.end_line;
    let container = all
        .iter()
        .enumerate()
        .filter(|(_, u)| contains(u))
        .min_by_key(|(_, u)| u.end_line - u.start_line);
    let head = container.filter(|(_, u)| d.start - u.start_line <= HEAD_GAP);
    if d.docstring {
        return head.map(|(i, _)| i);
    }
    let within = |u: &Unit| {
        container.is_none_or(|(_, c)| c.start_line < u.start_line && u.end_line <= c.end_line)
    };
    let led = all
        .iter()
        .enumerate()
        .filter(|(_, u)| d.end < u.start_line && u.start_line - d.end <= LEAD_GAP && within(u))
        .min_by_key(|(_, u)| u.start_line);
    led.or(head).map(|(i, _)| i)
}
