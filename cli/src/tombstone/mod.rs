//! Tombstone residue (plan v2.26 T track, ADR-008 fifth period): the
//! shape an agent leaves when told to remove X — X is gone from the
//! code, and the same change writes X back as an absence: a heading
//! `Tomato and Egg (no Dongpo Pork)`, an identifier
//! `cook_without_dongpo`, a docstring `this recipe no longer uses
//! braise_dongpo_pork`. Named after the database tombstone, the
//! marker a deleted record leaves in its place.
//!
//! Two counts over one changeset `{(before, after)}`:
//!   label — an added heading / new unit name / new file stem whose
//!           absence frame binds an erased name (frames.rs × names.rs);
//!   prose — an added comment / docstring / paragraph carrying BOTH a
//!           retrospective mark AND an erased name. The conjunction
//!           is the whole precision argument: `no longer` alone is
//!           ordinary prose, a mention alone is a migration guide.
//! A document in the changelog role (role.rs), a file `[tombstone]
//! ledger` declares, or a segment that is a ledger by itself is
//! exempt and counted, never silent. This side MEASURES: every
//! candidate surface becomes a row of three integers (wire.rs) and
//! the core judges which rows are sites, the split and the budget
//! condition (tombstone/1, plan v2.27 — the FPR ledger
//! docs/FPR-TOMBSTONE.md opened that gate); every number lands in the
//! observe feed (feed_json), the judgment beside the measurement.
//!
//! Erased names cross the session as KEYS only (names::key): the
//! PreToolUse leg stores this edit's keys in the feed and the next
//! edit unions them in as `session`, so an X deleted three edits ago
//! still binds the heading written now. No name is ever written out.

pub mod feed;
pub mod frames;
mod marked;
pub mod names;
pub mod policy;
pub mod role;
pub mod surfaces;
pub mod texts;
pub mod vocab;
pub mod wire;

pub use feed::{HASH_CAP, SITE_CAP, feed_json};
pub use policy::Policy;
pub use vocab::TOMBSTONE_REV;
pub use wire::{Judged, Judgment};

use crate::scan::lang::Lang;
use names::Erased;
use role::Witness;
use std::collections::BTreeSet;

/// One changed file: its ce-relative path and both texts (empty =
/// the side does not exist).
pub struct PairText<'a> {
    pub rel: &'a str,
    pub before: &'a str,
    pub after: &'a str,
    pub lang: Lang,
}

/// How a site fired: a parenthesized label frame (spec kind 0a), a
/// bare one (0b), or the prose conjunction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bracketed,
    Bare,
    Prose,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Bracketed => "bracketed",
            Kind::Bare => "bare",
            Kind::Prose => "prose",
        }
    }
}

/// One candidate surface this change wrote, as the wire reads it —
/// the three integers the core judges (`kind`, `marks`, `names`) —
/// and what only this side may know: where it is, the first name it
/// bound (the replay's arbitration column), an excerpt, its segment's
/// ledger tokens. A surface with no mark and no name is not a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub file: String,
    pub line: usize,
    pub kind: Kind,
    /// Retrospective marks in the sentence (0 for a label).
    pub marks: usize,
    /// Erased names the surface binds.
    pub names: usize,
    /// The first bound name's text (empty when none). Never in the feed.
    pub name: String,
    /// The surface's text, clipped to EXCERPT_CHARS; replay only.
    pub excerpt: String,
    /// Distinct ledger tokens of the segment around the row (Markdown
    /// only, 0 elsewhere): the replay's column for the third witness's
    /// threshold, `role::SEGMENT_TOKENS`. Never in the feed.
    pub ledger: usize,
}

impl Row {
    /// `file:line kind` — the feed's spelling of a place.
    pub fn place(&self) -> String {
        format!("{}:{} {}", self.file, self.line, self.kind.name())
    }
}

/// One exemption: a whole file by role (`line` None), or one segment
/// of a file by the third witness (`line` = where it starts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exempt {
    pub file: String,
    pub line: Option<usize>,
    pub why: Witness,
    /// The segment's ledger tokens (0 for a file); replay only.
    pub tokens: usize,
}

/// The measurement of one changeset — facts only; the judgment is
/// the core's (wire::Judged).
#[derive(Debug, Default)]
pub struct Findings {
    /// R — this changeset's erased names (keys leave; texts stay here).
    pub erased: Vec<names::Name>,
    /// Every exemption with its witness — a file by role or
    /// declaration whenever the changeset erased something, a segment
    /// only when it suppressed a row (an exemption that suppressed
    /// nothing is nothing to see).
    pub exempt: Vec<Exempt>,
    /// The candidate rows, in the order the wire sends them.
    pub rows: Vec<Row>,
}

impl Findings {
    /// The rows the core judged sites, in its order.
    pub fn judged_rows<'a>(&'a self, j: &'a Judged) -> impl Iterator<Item = &'a Row> + 'a {
        j.sites.iter().filter_map(move |&i| self.rows.get(i))
    }
}

/// Measure a changeset. `session` = erased keys carried over from
/// earlier edits of the same session (empty for a Stop audit, which
/// sees the whole session's diff at once); `policy` = what ce.toml
/// declares (`[tombstone] ledger` / `terms`; default = nothing).
pub fn measure(pairs: &[PairText], session: &BTreeSet<u64>, policy: &Policy) -> Findings {
    let added: Vec<BTreeSet<usize>> = pairs.iter().map(surfaces::added_lines).collect();
    let erased = names::erased(pairs, &added, policy);
    let mut out = Findings {
        erased: erased.names.clone(),
        ..Findings::default()
    };
    if out.erased.is_empty() && session.is_empty() {
        return out;
    }
    for (p, added) in pairs.iter().zip(&added) {
        // a declared ledger is exempt by the repository's own word,
        // whatever its language; the witnesses read the rest
        let declared = policy.declared(p.rel).then_some(Witness::Declared);
        if let Some(w) = declared.or_else(|| role::changelog_role(p.rel, p.after, p.lang)) {
            out.exempt.push(Exempt {
                file: p.rel.to_string(),
                line: None,
                why: w,
                tokens: 0,
            });
            continue;
        }
        candidates(p, added, &erased, session, &mut out);
    }
    out
}

/// Every candidate surface of one pair: a label whose frame binds an
/// erased name (names = how many), and every prose sentence carrying
/// a mark or an erased name (the core applies the conjunction).
fn candidates(
    p: &PairText,
    added: &BTreeSet<usize>,
    erased: &Erased,
    session: &BTreeSet<u64>,
    out: &mut Findings,
) {
    let known = |k: u64| erased.has(k) || session.contains(&k);
    for l in surfaces::labels(p, added) {
        let bound: Vec<frames::Candidate> = frames::label_candidates(&frames::words(&l.text))
            .into_iter()
            .filter(|c| known(names::key(&c.span.text)))
            .collect();
        if let Some(first) = bound.first() {
            let kind = if first.bracketed {
                Kind::Bracketed
            } else {
                Kind::Bare
            };
            let surface = Surface {
                line: l.line,
                kind,
                marks: 0,
                names: bound.len(),
                name: &first.span.text,
                text: &l.text,
            };
            admit(p, out, surface);
        }
    }
    for s in surfaces::prose(p, added) {
        for sentence in frames::sentences(&s.text) {
            let marks = frames::marks(sentence);
            let mut bound = names::spelled_all(sentence, known);
            bound.extend(erased.wide_all(sentence).map(str::to_string));
            if marks + bound.len() == 0 {
                continue;
            }
            let surface = Surface {
                line: s.start,
                kind: Kind::Prose,
                marks,
                names: bound.len(),
                name: bound.first().map_or("", String::as_str),
                text: sentence,
            };
            admit(p, out, surface);
        }
    }
}

/// Characters of surface text a row keeps for the replay.
const EXCERPT_CHARS: usize = 160;

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
/// is a ledger by itself (role::segment, Markdown only), which exempts
/// it and is counted once per segment.
fn admit(p: &PairText, out: &mut Findings, s: Surface) {
    let (start, tokens) = if p.lang == Lang::Markdown {
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
#[path = "../../tests/unit/tombstone.rs"]
mod tests;
