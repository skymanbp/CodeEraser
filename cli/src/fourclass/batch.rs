//! L2 batch classification: run L1 per pair unchanged, ship only the
//! leftovers (significant lines L1 called novel/deleted) to ce-core
//! as `[line, fnv1a(trim)]`, and apply the returned monotone delta.
//! L1 is the IR producer, not a modified engine — single-pair batches
//! with no link are bitwise L1, which is what keeps the saturated
//! 200-sample scores structural rather than re-proven.
//!
//! Every fallback (no link, missing capability, link error, degraded
//! core) returns the pure-L1 result with a visible reason (A9f):
//! plan R8's "退回 L1 而非退回无" is a code path, not a promise.

mod delta;

use super::stacking::dup_units;
use super::{Classification, classify, significant};
use crate::corelink::Link;
use crate::dedup::tokens::fnv1a;
use crate::scan::lang::Lang;
use delta::merge;
use serde_json::{Value, json};

pub struct PairInput<'a> {
    pub before: &'a str,
    pub after: &'a str,
    pub lang: Lang,
}

/// One accepted cross-pair correspondence, unit-attributed on both
/// ends (judgment — which lines correspond — stayed in Haskell;
/// symbol lookup stays here, where ADR-002 puts symbols).
#[derive(Debug)]
pub struct Relocation {
    pub from_pair: usize,
    pub from_unit: Option<String>,
    pub to_pair: usize,
    pub to_unit: Option<String>,
    pub lines: usize,
}

pub struct BatchClassification {
    pub pairs: Vec<Classification>,
    pub relocations: Vec<Relocation>,
    /// (pair index, rule name) per firing M4 judgment rule.
    pub suspicions: Vec<(usize, String)>,
    /// None = the cross-file pass ran (or nothing needed asking).
    pub degraded: Option<String>,
    /// Did the LINK fail, or did the core ANSWER without a verdict?
    pub link_failed: bool, // stated, not inferred: the restart budget keys on it
}

pub fn classify_batch(inputs: &[PairInput], link: Option<&mut Link>) -> BatchClassification {
    let pairs: Vec<Classification> = inputs
        .iter()
        .map(|p| classify(p.before, p.after, p.lang))
        .collect();
    let done = |pairs, degraded, link_failed| BatchClassification {
        pairs,
        relocations: Vec::new(),
        suspicions: Vec::new(),
        degraded,
        link_failed,
    };
    let Some(link) = link else {
        return done(pairs, Some("no_link".into()), false); // link_mut counted it
    };
    if !link.has("fourclass/2") {
        return done(pairs, Some("no_capability".into()), false); // alive, wrong family
    }
    let sent = leftovers(inputs, &pairs);
    if sent
        .iter()
        .all(|(rem, add)| rem.is_empty() && add.is_empty())
    {
        return done(pairs, None, false);
    }
    match link.request("fourclass", request_body(inputs, &sent)) {
        Err(e) => done(pairs, Some(e), true), // the link itself
        // A degraded reply (bucket cap) may carry the partial blocks
        // the core still derived — applying them would be partial L2
        // behind a flag, breaking the header's pure-L1 promise. The
        // reason is checked BEFORE merge on purpose (attack review
        // 2026-08-11 F3: the old order merged first).
        Ok(reply) => match reply["reason"].as_str() {
            Some(r) => done(pairs, Some(r.to_string()), false), // it ANSWERED
            // merge works on a COPY: a bad delta falls back to the
            // UNTOUCHED pure-L1 pairs — the in-place form returned a
            // half-merged result here (review 2026-08-20 #5)
            None => match merge(&reply, inputs, &sent, &pairs) {
                Err(e) => done(pairs, Some(e), false), // live link, bad delta
                Ok((merged, relocations)) => BatchClassification {
                    pairs: merged,
                    relocations,
                    suspicions: suspicions_of(&reply),
                    degraded: None,
                    link_failed: false,
                },
            },
        },
    }
}

fn suspicions_of(reply: &Value) -> Vec<(usize, String)> {
    reply["suspicions"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| Some((s[0].as_u64()? as usize, s[1].as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

/// One contiguous run of significant leftover lines, as (1-based
/// line, fnv1a(trim), alnum width) — the width is the line fact the
/// core's anchor floor judges on (wire 2.0.0). Run structure is
/// ALIGNMENT data and therefore produced here, by the aligner: two
/// leftovers are adjacent iff every line between them is also changed
/// and none of those in-between changed lines is significant (blanks
/// and bare punctuation bridge a run — git's moved blocks span them
/// too; a significant in-between line, moved or unchanged, breaks it).
pub type Run = Vec<(usize, u64, usize)>;
pub type Side = Vec<Run>;

/// Per pair: the significant changed lines L1 left novel/deleted,
/// grouped into runs, as (1-based line, fnv1a of the trimmed
/// content). Public so eval and contract tests can assert the exact
/// request content.
pub fn leftovers(inputs: &[PairInput], pairs: &[Classification]) -> Vec<(Side, Side)> {
    inputs
        .iter()
        .zip(pairs)
        .map(|(input, c)| {
            let moved = |removed: bool| -> Vec<usize> {
                c.moved
                    .iter()
                    .filter(|m| m.removed == removed)
                    .map(|m| m.line)
                    .collect()
            };
            (
                side_runs(input.before, &c.changed.removed, &moved(true)),
                side_runs(input.after, &c.changed.added, &moved(false)),
            )
        })
        .collect()
}

/// Longest insignificant bridge a run may span. Unbounded bridging
/// let two significant lines 1000 punctuation lines apart compress
/// into adjacency and fuse remote coincidences (attack review F6).
/// Bound = the maximum observed on the frozen slice — bridge-width
/// histogram over every leftover run of all 47 commits:
/// {0:7037, 1:663, 2:411, 3:90, 4:19, 5:17, 6:2, 7:1} — revalidated
/// by the full-corpus rerun (recall stayed 547/547).
const MAX_BRIDGE: usize = 7;

fn side_runs(text: &str, changed: &[usize], moved: &[usize]) -> Side {
    let lines: Vec<&str> = text.lines().collect();
    let mut runs: Side = Vec::new();
    let mut open = false;
    let mut prev = 0usize;
    let mut last_kept = 0usize;
    for &l in changed {
        if l != prev + 1 {
            open = false; // an unchanged gap breaks the run
        }
        prev = l;
        let t = lines[l - 1];
        if !significant(t) {
            continue; // blank/punctuation changed line bridges
        }
        if moved.contains(&l) {
            open = false; // a within-moved line breaks the run
            continue;
        }
        if open && l - last_kept - 1 > MAX_BRIDGE {
            open = false; // an over-long bridge is not adjacency
        }
        let entry = (l, fnv1a(t.trim().as_bytes()), super::alnum_width(t));
        match runs.last_mut() {
            Some(run) if open => run.push(entry),
            _ => runs.push(vec![entry]),
        }
        open = true;
        last_kept = l;
    }
    runs
}

/// Public so the ablation's block-level equivalence gate can replay
/// the EXACT wire request and compare the shadow's site decomposition
/// against the core's reply blocks (Codex review 2026-08-11 F1: the
/// moved-map equivalence alone cannot pin block boundaries).
pub fn request_body(inputs: &[PairInput], sent: &[(Side, Side)]) -> Value {
    let pairs: Vec<Value> = sent
        .iter()
        .enumerate()
        .filter(|(_, (rem, add))| !rem.is_empty() || !add.is_empty())
        .map(
            |(i, (rem, add))| json!({"i": i, "rem": rem, "add": add, "dup": dup_units(&inputs[i])}),
        )
        .collect();
    json!({"pairs": pairs})
}

// Delta application (merge / apply_side / relocations_of) lives in
// delta.rs, split at the 300-line dogfood wall — where a returned
// line is CONSUMED from the sent-leftover set, so a double-listed
// line is a named error instead of a usize underflow.
