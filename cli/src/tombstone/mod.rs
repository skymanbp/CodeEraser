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
//! A document in the changelog role (role.rs) is exempt whole and
//! counted, never silent. Stage one is measurement only: nothing
//! here decides, and every number lands in the observe feed
//! (feed_json) for the FPR ledger. Stage two (a wire family whose
//! Haskell side owns the conjunction and the floor) opens only past
//! that ledger's gate — see docs/DEVELOPMENT_PLAN.md v2.26.
//!
//! Erased names cross the session as KEYS only (names::key): the
//! PreToolUse leg stores this edit's keys in the feed and the next
//! edit unions them in as `session`, so an X deleted three edits ago
//! still binds the heading written now. No name is ever written out.

pub mod frames;
mod marked;
pub mod names;
pub mod policy;
pub mod role;
pub mod surfaces;
pub mod texts;
pub mod vocab;

pub use policy::Policy;
pub use vocab::TOMBSTONE_REV;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    pub file: String,
    pub line: usize,
    pub kind: Kind,
    /// The erased name this site bound — the replay's arbitration
    /// column; never in the feed (no name text leaves the machine).
    pub name: String,
    /// The surface's text, clipped to EXCERPT_CHARS; replay only.
    pub excerpt: String,
    /// Distinct ledger tokens of the segment around the site (Markdown
    /// only, 0 elsewhere): the replay's column for the third witness's
    /// threshold, `role::SEGMENT_TOKENS`. Never in the feed.
    pub ledger: usize,
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

/// The measurement of one changeset.
#[derive(Debug, Default)]
pub struct Findings {
    /// R — this changeset's erased names (keys leave; texts stay here).
    pub erased: Vec<names::Name>,
    pub label: usize,
    pub prose: usize,
    /// Every exemption with its witness — a file by role whenever the
    /// changeset erased something, a segment only when it suppressed a
    /// site (an exemption that suppressed nothing is nothing to see).
    pub exempt: Vec<Exempt>,
    pub sites: Vec<Site>,
}

/// How many erased keys one feed line carries at most (a whole-file
/// rewrite erases hundreds of names; the session union is a bounded
/// read, so the record is bounded too).
pub const HASH_CAP: usize = 256;
/// How many sites a feed line names; the counts stay exact.
pub const SITE_CAP: usize = 10;

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
        judge(p, added, &erased, session, &mut out);
    }
    out
}

fn judge(
    p: &PairText,
    added: &BTreeSet<usize>,
    erased: &Erased,
    session: &BTreeSet<u64>,
    out: &mut Findings,
) {
    let known = |k: u64| erased.has(k) || session.contains(&k);
    for l in surfaces::labels(p, added) {
        let hit = frames::label_candidates(&frames::words(&l.text))
            .into_iter()
            .find(|c| known(names::key(&c.span.text)));
        if let Some(c) = hit {
            let kind = if c.bracketed {
                Kind::Bracketed
            } else {
                Kind::Bare
            };
            let fired = Fired {
                line: l.line,
                kind,
                name: &c.span.text,
                text: &l.text,
            };
            admit(p, out, fired);
        }
    }
    for s in surfaces::prose(p, added) {
        let hit = frames::sentences(&s.text).into_iter().find_map(|sentence| {
            if !frames::has_mark(sentence) {
                return None;
            }
            names::spelled_in(sentence, known)
                .or_else(|| erased.wide_in(sentence).map(str::to_string))
                .map(|name| (name, sentence))
        });
        if let Some((name, sentence)) = hit {
            let fired = Fired {
                line: s.start,
                kind: Kind::Prose,
                name: &name,
                text: sentence,
            };
            admit(p, out, fired);
        }
    }
}

/// Characters of surface text a site keeps for the replay.
const EXCERPT_CHARS: usize = 160;

/// One surface that bound an erased name, before the third witness
/// has looked at it.
struct Fired<'a> {
    line: usize,
    kind: Kind,
    name: &'a str,
    text: &'a str,
}

/// A fired surface becomes a site — unless the segment around it is
/// a ledger by itself (role::segment, Markdown only), which exempts
/// it and is counted once per segment.
fn admit(p: &PairText, out: &mut Findings, f: Fired) {
    let (start, tokens) = if p.lang == Lang::Markdown {
        role::segment(p.after, f.line)
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
    match f.kind {
        Kind::Prose => out.prose += 1,
        Kind::Bracketed | Kind::Bare => out.label += 1,
    }
    out.sites.push(Site {
        file: p.rel.to_string(),
        line: f.line,
        kind: f.kind,
        name: f.name.to_string(),
        excerpt: f.text.chars().take(EXCERPT_CHARS).collect(),
        ledger: tokens,
    });
}

/// The additive feed object every producer writes (the observe-feed
/// contract, `hookio::OBSERVE_SCHEMA` 0.9.0): counts, the exemptions
/// with their witness (a segment's entry says where it starts), the
/// first sites as `file:line kind`, and — for the per-edit leg, which
/// carries names across a session — the erased keys (capped) and the
/// session union's size. No name text.
pub fn feed_json(f: &Findings, session: Option<usize>) -> serde_json::Value {
    let exempt: Vec<serde_json::Value> = f
        .exempt
        .iter()
        .map(|e| {
            let mut v = serde_json::json!({"file": e.file, "why": e.why.name()});
            if let Some(line) = e.line {
                v["line"] = serde_json::json!(line);
            }
            v
        })
        .collect();
    let sites: Vec<String> = f
        .sites
        .iter()
        .take(SITE_CAP)
        .map(|s| format!("{}:{} {}", s.file, s.line, s.kind.name()))
        .collect();
    let mut line = serde_json::json!({
        "rev": TOMBSTONE_REV,
        "erased": f.erased.len(),
        "label": f.label,
        "prose": f.prose,
        "exempt": exempt,
        "sites": sites,
    });
    if let Some(carried) = session {
        let hashes: Vec<u64> = f.erased.iter().take(HASH_CAP).map(|n| n.key).collect();
        line["erased_hashes"] = serde_json::json!(hashes);
        line["session_erased"] = serde_json::json!(carried);
    }
    line
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone.rs"]
mod tests;
