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

mod candidates;
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
    /// Pairs whose line diff was bounded (surfaces::Added::degraded):
    /// their "added" lines are every trimmed line, so their rows may be
    /// untouched text — counted in the feed, never enforced on.
    pub degraded_pairs: usize,
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
    measure_with(pairs, &[], session, policy)
}

/// `measure` with after-only SURFACES beside the pairs — a commit
/// message: each offers candidate rows (its subject line as a label,
/// its sentences as prose) but declares no name and keeps none alive,
/// so R is the pairs' alone (spec: the staged changes' R × every
/// surface; a message item `- dongpo is no longer needed` must not
/// keep `dongpo` alive), and no witness reads it — a ledger is a
/// file's role, and a message is no file.
pub fn measure_with(
    pairs: &[PairText],
    messages: &[PairText],
    session: &BTreeSet<u64>,
    policy: &Policy,
) -> Findings {
    let added: Vec<surfaces::Added> = pairs.iter().map(surfaces::added).collect();
    let erased = names::erased(pairs, &added, policy);
    let mut out = Findings {
        erased: erased.names.clone(),
        degraded_pairs: added.iter().filter(|a| a.degraded).count(),
        ..Findings::default()
    };
    if out.erased.is_empty() && session.is_empty() {
        return out;
    }
    let known = candidates::Known {
        erased: &erased,
        session,
    };
    for (p, a) in pairs.iter().zip(&added) {
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
        candidates::of(p, &a.lines, &known, &mut out, false);
    }
    for m in messages {
        candidates::of(m, &surfaces::added(m).lines, &known, &mut out, true);
    }
    out
}

/// The keys of every name `text` declares — what a later edit of the
/// session REVIVES when one of them was erased before (the guard leg's
/// union subtracts them: a name written back is alive, not residue).
pub fn declared_keys(text: &str, lang: Lang, policy: &Policy) -> BTreeSet<u64> {
    names::names_of(text, lang, policy)
        .into_iter()
        .map(|n| n.key)
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone.rs"]
mod tests;
