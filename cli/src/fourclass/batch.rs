//! L2 batch classification: ask the core for L1 per pair (`moves/1`,
//! moves.rs — the within-pair judgment and the leftover runs it
//! leaves), ship only the leftovers (significant lines L1 called
//! novel/deleted) to the core as `[line, fnv1a(trim), width]`, and
//! apply the returned monotone delta. L1 is the IR producer, not a
//! modified engine.
//!
//! Since plan v2.33 W3 L1 is the core's too, so there is no Rust copy
//! to fall back on: no link, a missing capability, a link error or a
//! degraded L1 answer is a report with no pairs and a visible reason
//! (A9f); a degraded CROSS-PAIR answer keeps the core's L1 pairs
//! (plan R8's "退回 L1 而非退回无").

mod delta;
mod edges;

use super::model::Classification;
use super::moves;
use crate::corelink::Link;
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
    let Some(link) = link else {
        return done(Vec::new(), Some("no_link".into()), false); // link_mut counted it
    };
    if !link.has(moves::CAP) || !link.has("fourclass/2") {
        return done(Vec::new(), Some("no_capability".into()), false); // alive, wrong family
    }
    let (req, bodies) = moves::request(inputs);
    let mut replies = Vec::with_capacity(bodies.len());
    for body in bodies {
        match link.request(moves::KIND, body) {
            Err(e) => return done(Vec::new(), Some(e), true), // the link itself
            Ok(reply) => replies.push(reply),
        }
    }
    let l1 = match moves::read(req, &replies) {
        Ok(l1) => l1,
        Err(e) => return done(Vec::new(), Some(e), false),
    };
    let (pairs, sent) = (l1.pairs, l1.runs);
    if sent
        .iter()
        .all(|(rem, add)| rem.is_empty() && add.is_empty())
    {
        return done(pairs, None, false);
    }
    match link.request("fourclass", request_body(&pairs, &sent, &l1.dup_spans)) {
        Err(e) => done(pairs, Some(e), true), // the link itself
        Ok(reply) => {
            consume(&reply, inputs, &sent, &pairs).unwrap_or_else(|e| done(pairs, Some(e), false))
        }
    }
}

fn done(
    pairs: Vec<Classification>,
    degraded: Option<String>,
    link_failed: bool,
) -> BatchClassification {
    BatchClassification {
        pairs,
        relocations: Vec::new(),
        suspicions: Vec::new(),
        degraded,
        link_failed,
    }
}

fn consume(
    reply: &Value,
    inputs: &[PairInput],
    sent: &[(Side, Side)],
    pairs: &[Classification],
) -> Result<BatchClassification, String> {
    // Check the reason BEFORE merge: partial blocks behind a bucket
    // cap must never escape as a complete L2 result (review F3).
    if let Some(reason) = reply["reason"].as_str() {
        return Err(reason.to_string());
    }
    // Merge copies L1, so a malformed delta or declaration answer
    // returns the untouched pairs (review 2026-08-20 #5).
    let (merged, mut relocations) = merge(reply, inputs, sent, pairs)?;
    relocations.extend(edges::unit_edges(reply, pairs, &relocations)?);
    Ok(BatchClassification {
        pairs: merged,
        relocations,
        suspicions: suspicions_of(reply),
        degraded: None,
        link_failed: false,
    })
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
/// core's anchor floor judges on (wire 2.0.0). Run structure is the
/// core's L1 answer (CE.FourClass.Moves): two leftovers are adjacent
/// iff every line between them is also changed and none of those
/// in-between changed lines is significant, over a bounded bridge.
pub type Run = Vec<(usize, u64, usize)>;
pub type Side = Vec<Run>;

/// The wire request over the leftover runs. Was public for the
/// retired eval_ablation's block-level equivalence replay (v0.5.0,
/// EVAL-SET.md); classify_batch is its one reader today, so the face
/// is private — a revival re-opens it together with the instrument.
fn request_body(pairs: &[Classification], sent: &[(Side, Side)], dup: &[Vec<[u64; 3]>]) -> Value {
    let pairs: Vec<Value> = sent
        .iter()
        .enumerate()
        // Even a pair with no leftovers can be a second declaration
        // destination. The core needs every pair to judge uniqueness.
        .map(|(i, (rem, add))| {
            let (drem, dadd) = super::decls::request_keys(&pairs[i].decls);
            json!({"i": i, "rem": rem, "add": add, "dupSpans": dup[i],
                   "declRem": drem, "declAdd": dadd})
        })
        .collect();
    json!({"pairs": pairs})
}

// Delta application (merge / apply_side / relocations_of) lives in
// delta.rs, split at the 300-line dogfood wall — where a returned
// line is CONSUMED from the sent-leftover set, so a double-listed
// line is a named error instead of a usize underflow.
