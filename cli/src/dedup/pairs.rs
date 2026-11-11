//! Clone extraction, SIGMOD'03 two-phase: shared fingerprints are
//! candidate ANCHORS only; every anchor pair this module ENUMERATES
//! is verified by exact bidirectional extension over the two
//! normalized token streams; the maximal runs of >= t tokens go to the
//! core, whose coverage rule (cover/1, dedup/cover.rs) says which are
//! reported.
//! Enumeration is not exhaustive above the hot cap: a hot hash group is
//! walked as an adjacent chain, so the non-adjacent pairs inside it
//! are never verified and any block only they would witness is not
//! reported. That is a recall bound, and it is counted, not silent —
//! one `hot_chained` tick per chained group. Replaces the round-1
//! line-merge heuristic whose flat-sort chaining both fragmented
//! boilerplate-dense regions and built pathological blocks
//! (DEDUP-CALIBRATION.md).

use super::anchors::{GroupEvent, each_hash_pair};
use super::cover;
use super::index::Instance;
use super::tokens::Token;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A verified maximal common token run, mapped back to lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub a_file: String,
    pub a_start: usize,
    pub a_end: usize,
    pub b_file: String,
    pub b_start: usize,
    pub b_end: usize,
    /// Exact verified length of the common normalized-token run.
    pub tokens: usize,
    /// Unique token hashes in the run — the literal-degeneracy signal:
    /// data-row matches (`LIT: (LIT,...),` tables) have a tiny
    /// alphabet while real code clones are diverse. Reported for the
    /// M3 judgment layer; no silent filtering here (calibration
    /// arbitration, DEDUP-CALIBRATION.md).
    pub distinct: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Blocks {
    pub blocks: Vec<Block>,
    /// K-way families aggregated from `blocks` (attack-review R8).
    pub groups: Vec<super::groups::Group>,
    /// Hash groups paired as adjacent chains instead of full pairwise.
    pub hot_chained: usize,
    /// Anchors whose stored token offset exceeded the live stream
    /// (file changed between index refresh and stream load) — skipped
    /// and counted, never a panic (attack-review D1).
    pub stale_skipped: usize,
    /// Blocks suppressed by the diversity floor — counted, never
    /// silent (override with --min-distinct).
    pub low_diversity_suppressed: usize,
    /// Every PRE-filter block's distinct count, sorted (batch-7
    /// slice 1): `ce dedup --check` ships these so the core
    /// re-derives the admitted count with ITS floor
    /// (CE.Dedup.Cost.minDistinct) and judges the budget from that.
    pub distincts: Vec<u64>,
}

/// Report filters. `min_distinct` defaults to 7: across the fixture
/// corpus + cobra + requests the arbitrated data-row false positives
/// (status_codes rows, locale key sections, pygments style dicts)
/// measured distinct <= 6 while arbitrated true clones measured >= 7
/// (M2 calibration, DEDUP-CALIBRATION.md; one 16-outlier FP survives
/// — the floor buys precision, not purity).
#[derive(Debug, Clone, Copy)]
pub struct Filter {
    pub min_tokens: usize,
    pub min_distinct: usize,
}

/// The default diversity floor, CE.Dedup.Cost.minDistinct read off
/// the core's package (plan v2.33 W3): it filters the report and the
/// probe hot path, and `ce dedup --check` still proves the core judged
/// the same count on every gated run (reply.dedupBlocks vs the local
/// count).
pub fn default_min_distinct() -> usize {
    crate::tables::get().limits.dedup.min_distinct
}

pub type Streams = BTreeMap<String, Vec<Token>>;

/// The verified runs plus the transparency counters. `t` is the
/// report threshold; runs in [near_floor, t) land in the second sink
/// instead of vanishing (near_floor usize::MAX disables it).
struct Ctx<'s> {
    runs: BTreeSet<(&'s str, usize, &'s str, usize, usize)>,
    near: BTreeSet<(&'s str, usize, &'s str, usize, usize)>,
    stale_skipped: usize,
    t: usize,
    near_floor: usize,
}

/// One grouping walk and one extension pass over the anchors: every
/// verified run of at least `t` tokens, the near runs in [near_floor, t)
/// and the counters. The report path and the T3 candidate source read
/// this one walk, so they cannot disagree about what was verified.
fn verify<'s>(
    instances: &[Instance],
    streams: &'s Streams,
    t: usize,
    near_floor: usize,
) -> (Ctx<'s>, usize) {
    let mut hot_chained = 0;
    let mut ctx = Ctx {
        runs: BTreeSet::new(),
        near: BTreeSet::new(),
        stale_skipped: 0,
        t,
        near_floor,
    };
    each_hash_pair(instances, |ev| match ev {
        GroupEvent::Chained => hot_chained += 1,
        GroupEvent::Pair(a, b) => extend_anchor(a, b, streams, &mut ctx),
    });
    (ctx, hot_chained)
}

/// `f.min_tokens` is the verified-run threshold (the winnowing
/// guarantee t by default); `f.min_distinct` the diversity floor. Which
/// verified runs are reported, which suppressed (kept below the floor,
/// counted) and which dropped is the core's coverage rule (cover/1,
/// CE.Dedup.Cover): per file pair, diverse runs first, longest first,
/// each kept only with `t` tokens of its own on some side — a shorter run
/// that shares lines with a longer one is no longer lost when most of its
/// tokens are its own, and a low-diversity run never hides a diverse one.
pub fn clone_blocks(instances: &[Instance], streams: &Streams, f: Filter) -> Result<Blocks> {
    let (ctx, hot_chained) = verify(instances, streams, f.min_tokens, usize::MAX);
    let runs: Vec<cover::Run> = ctx
        .runs
        .iter()
        .map(|&(a, a0, b, b0, len)| cover::Run {
            a,
            a0,
            b,
            b0,
            len,
            distinct: distinct_of(&streams[a][a0..a0 + len]),
        })
        .collect();
    let classes = cover::classify(&runs, (f.min_tokens, f.min_distinct))?;
    let mut blocks = Vec::new();
    let mut distincts = Vec::new();
    let mut suppressed = 0;
    for (run, class) in runs.iter().zip(classes) {
        match class {
            cover::Class::Reported => blocks.push(to_block(run, streams)),
            cover::Class::Suppressed => suppressed += 1,
            cover::Class::Dropped => continue,
        }
        distincts.push(run.distinct as u64);
    }
    distincts.sort_unstable();
    blocks.sort_by(|x, y| {
        (&x.a_file, x.a_start, &x.b_file, x.b_start)
            .cmp(&(&y.a_file, y.a_start, &y.b_file, y.b_start))
    });
    Ok(Blocks {
        distincts,
        low_diversity_suppressed: suppressed,
        groups: super::groups::group(&blocks),
        blocks,
        hot_chained,
        stale_skipped: ctx.stale_skipped,
    })
}

/// The near-miss sink alone: verified runs BELOW the report threshold
/// `t` but at/above `near_floor` — the population extend_anchor used to
/// drop silently. A shared fingerprint guarantees a common kgram run, so
/// with the floor at kgram this is exactly 25 <= len < 50: the T3
/// candidate source S1 (design §4.2). It asks the core nothing — the
/// near runs need no class.
pub fn near_blocks(
    instances: &[Instance],
    streams: &Streams,
    t: usize,
    near_floor: usize,
) -> Vec<Block> {
    let (ctx, _) = verify(instances, streams, t, near_floor);
    ctx.near
        .iter()
        .map(|&(a, a0, b, b0, len)| {
            let run = cover::Run {
                a,
                a0,
                b,
                b0,
                len,
                distinct: distinct_of(&streams[a][a0..a0 + len]),
            };
            to_block(&run, streams)
        })
        .collect()
}

fn extend_anchor<'s>(a: &Instance, b: &Instance, streams: &'s Streams, ctx: &mut Ctx<'s>) {
    let (a, b) = if (&a.file, a.start_tok) <= (&b.file, b.start_tok) {
        (a, b)
    } else {
        (b, a)
    };
    let (Some((af, sa)), Some((bf, sb))) = (
        streams.get_key_value(&a.file),
        streams.get_key_value(&b.file),
    ) else {
        return; // caller did not provide the stream — no claim made
    };
    // a stored offset beyond the live stream = the file changed after
    // the index refresh; skip and count, never index out of bounds
    if a.start_tok >= sa.len() || b.start_tok >= sb.len() {
        ctx.stale_skipped += 1;
        return;
    }
    if let Some((a0, b0, len)) = extend(sa, a.start_tok, sb, b.start_tok, a.file == b.file) {
        if len >= ctx.t {
            ctx.runs.insert((af.as_str(), a0, bf.as_str(), b0, len));
        } else if len >= ctx.near_floor {
            ctx.near.insert((af.as_str(), a0, bf.as_str(), b0, len));
        }
    }
}

/// Maximal exact common run around the anchor. For same-stream pairs
/// the run is capped at the anchor gap so the two ranges stay
/// disjoint (periodic code reports adjacent segments, like jscpd's
/// models.py 837-847 <-> 847-857 pair). pub(crate): the M3 probe
/// (dedup::probe) reuses the same verified-extension semantics.
pub(crate) fn extend(
    sa: &[Token],
    a_tok: usize,
    sb: &[Token],
    b_tok: usize,
    same: bool,
) -> Option<(usize, usize, usize)> {
    let (mut a0, mut b0) = (a_tok, b_tok);
    while a0 > 0 && b0 > 0 && sa[a0 - 1].hash == sb[b0 - 1].hash {
        a0 -= 1;
        b0 -= 1;
    }
    let mut len = 0;
    let cap = if same { b0 - a0 } else { usize::MAX };
    while a0 + len < sa.len()
        && b0 + len < sb.len()
        && len < cap
        && sa[a0 + len].hash == sb[b0 + len].hash
    {
        len += 1;
    }
    (len > 0).then_some((a0, b0, len))
}

/// Unique token hashes in a run.
fn distinct_of(run: &[Token]) -> usize {
    run.iter().map(|t| t.hash).collect::<BTreeSet<_>>().len()
}

fn to_block(run: &cover::Run, streams: &Streams) -> Block {
    let (sa, sb) = (&streams[run.a], &streams[run.b]);
    Block {
        a_file: run.a.to_string(),
        a_start: sa[run.a0].start_line,
        a_end: sa[run.a0 + run.len - 1].end_line,
        b_file: run.b.to_string(),
        b_start: sb[run.b0].start_line,
        b_end: sb[run.b0 + run.len - 1].end_line,
        tokens: run.len,
        distinct: run.distinct,
    }
}
