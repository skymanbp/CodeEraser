//! The candidate surfaces of one changed side (split from the hub in
//! the codex review batch, 2026-09-04): every label whose absence
//! frame binds a known name and every prose sentence carrying a mark
//! or a known name becomes a row — unless the segment around it is a
//! ledger by itself (role::segment, Markdown only), the third witness.
//! The hub decides which pairs get here (a file in the changelog role
//! never does); this module reads one side.

use super::names::{self, Erased};
use super::role::{self, Witness};
use super::{Exempt, Findings, Kind, PairText, Row, frames, surfaces};
use crate::scan::lang::Lang;
use std::collections::BTreeSet;

/// Characters of surface text a row keeps for the replay.
const EXCERPT_CHARS: usize = 160;

/// What a candidate may bind: this changeset's erased names and the
/// keys the session carried in.
pub(super) struct Known<'a> {
    pub erased: &'a Erased,
    pub session: &'a BTreeSet<u64>,
}

impl Known<'_> {
    fn has(&self, k: u64) -> bool {
        self.erased.has(k) || self.session.contains(&k)
    }
}

/// Every candidate surface of one side: a label whose frame binds a
/// known name (names = how many DISTINCT ones — a name written twice
/// is one name), and every prose sentence carrying a mark or a known
/// name (the core applies the conjunction). `message` = an after-only
/// surface that is no file (a commit message): its subject line is
/// its one label, and no segment witness reads it.
pub(super) fn of(
    p: &PairText,
    added: &BTreeSet<usize>,
    known: &Known,
    out: &mut Findings,
    message: bool,
) {
    let labels = if message {
        surfaces::subject(p.after)
    } else {
        surfaces::labels(p, added)
    };
    for l in labels {
        let bound: Vec<frames::Candidate> = frames::label_candidates(&frames::words(&l.text))
            .into_iter()
            .filter(|c| known.has(names::key(&c.span.text)))
            .collect();
        if let Some(first) = bound.first() {
            let kind = if first.bracketed {
                Kind::Bracketed
            } else {
                Kind::Bare
            };
            let keys: BTreeSet<u64> = bound.iter().map(|c| names::key(&c.span.text)).collect();
            let surface = Surface {
                line: l.line,
                kind,
                marks: 0,
                names: keys.len(),
                name: &first.span.text,
                text: &l.text,
            };
            admit(p, out, surface, message);
        }
    }
    for s in surfaces::prose(p, added) {
        let marks = frames::marks(&s.text);
        let bound = bound_names(&s.text, known);
        if marks + bound.len() == 0 {
            continue;
        }
        let surface = Surface {
            line: s.start,
            kind: Kind::Prose,
            marks,
            names: bound.len(),
            name: bound.first().map_or("", String::as_str),
            text: &s.text,
        };
        admit(p, out, surface, message);
    }
}

/// The distinct known names one sentence spells — ASCII spellings and
/// wide names alike, one per key.
fn bound_names(text: &str, known: &Known) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let spelled = names::spelled_all(text, |k| known.has(k));
    let wide = known.erased.wide_all(text).map(str::to_string);
    spelled
        .into_iter()
        .chain(wide)
        .filter(|n| seen.insert(names::key(n)))
        .collect()
}

/// One candidate surface before the third witness has looked at it.
struct Surface<'a> {
    line: usize,
    kind: Kind,
    marks: usize,
    names: usize,
    name: &'a str,
    text: &'a str,
}

/// A candidate surface becomes a row — unless the segment around it
/// is a ledger by itself (role::segment, Markdown only, never a
/// message), which exempts it and is counted once per segment.
fn admit(p: &PairText, out: &mut Findings, s: Surface, message: bool) {
    let (start, tokens) = if p.lang == Lang::Markdown && !message {
        role::segment(p.after, s.line)
    } else {
        (0, 0)
    };
    if tokens >= role::SEGMENT_TOKENS {
        let e = Exempt {
            file: p.rel.to_string(),
            line: Some(start),
            why: Witness::Segment,
            tokens,
        };
        if !out.exempt.contains(&e) {
            out.exempt.push(e);
        }
        return;
    }
    out.rows.push(Row {
        file: p.rel.to_string(),
        line: s.line,
        kind: s.kind,
        marks: s.marks,
        names: s.names,
        name: s.name.to_string(),
        excerpt: s.text.chars().take(EXCERPT_CHARS).collect(),
        ledger: tokens,
    });
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone/candidates.rs"]
mod tests;
