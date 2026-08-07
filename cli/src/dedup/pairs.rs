//! Clone extraction, SIGMOD'03 two-phase: shared fingerprints are
//! candidate ANCHORS only; every anchor pair is verified by exact
//! bidirectional extension over the two normalized token streams, and
//! only maximal runs of >= t tokens are reported. Replaces the round-1
//! line-merge heuristic whose flat-sort chaining both fragmented
//! boilerplate-dense regions and built pathological blocks
//! (DEDUP-CALIBRATION.md).

use super::index::Instance;
use super::tokens::Token;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// A hash shared by more locations than this is boilerplate; its
/// pairs explode quadratically. Skipped hashes are COUNTED and
/// reported, never silently dropped.
const HOT_CAP: usize = 64;

/// A verified maximal common token run, mapped back to lines.
#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct Blocks {
    pub blocks: Vec<Block>,
    pub hot_skipped: usize,
}

pub type Streams = BTreeMap<String, Vec<Token>>;

/// Files that participate in any shared hash — the caller provides
/// token streams for exactly these.
pub fn candidate_files(instances: &[Instance]) -> BTreeSet<String> {
    let mut by_hash: BTreeMap<u64, Vec<&Instance>> = BTreeMap::new();
    for inst in instances {
        by_hash.entry(inst.hash).or_default().push(inst);
    }
    by_hash
        .values()
        .filter(|g| g.len() > 1)
        .flat_map(|g| g.iter().map(|i| i.file.clone()))
        .collect()
}

/// `t` is the report threshold in tokens (= the winnowing guarantee
/// threshold, Params::window + Params::kgram - 1).
pub fn clone_blocks(instances: &[Instance], streams: &Streams, t: usize) -> Blocks {
    let mut by_hash: BTreeMap<u64, Vec<&Instance>> = BTreeMap::new();
    for inst in instances {
        by_hash.entry(inst.hash).or_default().push(inst);
    }
    let mut hot_skipped = 0;
    // (a_file, a_tok, b_file, b_tok, len) — extension is maximal, so
    // every anchor inside one true run lands on the same tuple.
    let mut runs: BTreeSet<(&str, usize, &str, usize, usize)> = BTreeSet::new();
    for group in by_hash.values().filter(|g| g.len() > 1) {
        if group.len() > HOT_CAP {
            hot_skipped += 1;
            continue;
        }
        for (i, a) in group.iter().enumerate() {
            for b in &group[i + 1..] {
                extend_anchor(a, b, streams, t, &mut runs);
            }
        }
    }
    let blocks = dominant(
        runs.into_iter()
            .map(|r| to_block(r, streams))
            .collect::<Vec<_>>(),
    );
    Blocks {
        blocks,
        hot_skipped,
    }
}

/// Periodic content yields one maximal run per offset; shifted
/// variants whose BOTH ranges sit inside a longer run of the same
/// file pair add no information — keep only dominant blocks.
fn dominant(mut blocks: Vec<Block>) -> Vec<Block> {
    blocks.sort_by(|x, y| y.tokens.cmp(&x.tokens));
    let mut kept: Vec<Block> = Vec::new();
    for b in blocks {
        let contained = kept.iter().any(|k| {
            k.a_file == b.a_file
                && k.b_file == b.b_file
                && k.a_start <= b.a_start
                && b.a_end <= k.a_end
                && k.b_start <= b.b_start
                && b.b_end <= k.b_end
        });
        if !contained {
            kept.push(b);
        }
    }
    kept.sort_by(|x, y| {
        (&x.a_file, x.a_start, &x.b_file, x.b_start)
            .cmp(&(&y.a_file, y.a_start, &y.b_file, y.b_start))
    });
    kept
}

fn extend_anchor<'s>(
    a: &Instance,
    b: &Instance,
    streams: &'s Streams,
    t: usize,
    runs: &mut BTreeSet<(&'s str, usize, &'s str, usize, usize)>,
) {
    let (a, b) = if (&a.file, a.start_tok) <= (&b.file, b.start_tok) {
        (a, b)
    } else {
        (b, a)
    };
    if a.file == b.file && a.start_tok == b.start_tok {
        return;
    }
    let (Some(sa), Some(sb)) = (streams.get(&a.file), streams.get(&b.file)) else {
        return; // caller did not provide the stream — no claim made
    };
    if let Some((a0, b0, len)) = extend(sa, a.start_tok, sb, b.start_tok, a.file == b.file)
        && len >= t
        && let (Some((af, _)), Some((bf, _))) = (
            streams.get_key_value(&a.file),
            streams.get_key_value(&b.file),
        )
    {
        runs.insert((af.as_str(), a0, bf.as_str(), b0, len));
    }
}

/// Maximal exact common run around the anchor. For same-stream pairs
/// the run is capped at the anchor gap so the two ranges stay
/// disjoint (periodic code reports adjacent segments, like jscpd's
/// models.py 837-847 <-> 847-857 pair).
fn extend(
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

fn to_block(run: (&str, usize, &str, usize, usize), streams: &Streams) -> Block {
    let (a_file, a0, b_file, b0, len) = run;
    let (sa, sb) = (&streams[a_file], &streams[b_file]);
    let distinct = sa[a0..a0 + len]
        .iter()
        .map(|t| t.hash)
        .collect::<BTreeSet<_>>()
        .len();
    Block {
        a_file: a_file.to_string(),
        a_start: sa[a0].start_line,
        a_end: sa[a0 + len - 1].end_line,
        b_file: b_file.to_string(),
        b_start: sb[b0].start_line,
        b_end: sb[b0 + len - 1].end_line,
        tokens: len,
        distinct,
    }
}
