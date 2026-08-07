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

/// Above this group size, pairing switches from full pairwise
/// (C(n,2)) to an adjacent chain (n-1 pairs) sorted by (file, tok).
/// Attack-review D4: skipping hot groups made detection FALL TO ZERO
/// as duplication rose (65 identical files → 0 blocks); chaining
/// keeps every instance in ≥1 verified pair at linear cost.
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
    /// Hash groups paired as adjacent chains instead of full pairwise.
    pub hot_chained: usize,
    /// Anchors whose stored token offset exceeded the live stream
    /// (file changed between index refresh and stream load) — skipped
    /// and counted, never a panic (attack-review D1).
    pub stale_skipped: usize,
    /// Blocks suppressed by the diversity floor — counted, never
    /// silent (override with --min-distinct).
    pub low_diversity_suppressed: usize,
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

pub const DEFAULT_MIN_DISTINCT: usize = 7;

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

/// Collects verified runs plus the transparency counters.
struct Ctx<'s> {
    runs: BTreeSet<(&'s str, usize, &'s str, usize, usize)>,
    stale_skipped: usize,
}

/// `f.min_tokens` is the verified-run threshold (the winnowing
/// guarantee t by default); `f.min_distinct` the diversity floor.
pub fn clone_blocks(instances: &[Instance], streams: &Streams, f: Filter) -> Blocks {
    let mut by_hash: BTreeMap<u64, Vec<&Instance>> = BTreeMap::new();
    for inst in instances {
        by_hash.entry(inst.hash).or_default().push(inst);
    }
    let mut hot_chained = 0;
    let mut ctx = Ctx {
        runs: BTreeSet::new(),
        stale_skipped: 0,
    };
    for group in by_hash.values_mut().filter(|g| g.len() > 1) {
        if group.len() > HOT_CAP {
            hot_chained += 1;
            group.sort_by(|x, y| (&x.file, x.start_tok).cmp(&(&y.file, y.start_tok)));
            for pair in group.windows(2) {
                extend_anchor(pair[0], pair[1], streams, f.min_tokens, &mut ctx);
            }
        } else {
            for (i, a) in group.iter().enumerate() {
                for b in &group[i + 1..] {
                    extend_anchor(a, b, streams, f.min_tokens, &mut ctx);
                }
            }
        }
    }
    let mut blocks = dominant(
        ctx.runs
            .into_iter()
            .map(|r| to_block(r, streams))
            .collect::<Vec<_>>(),
    );
    let before = blocks.len();
    blocks.retain(|b| b.distinct >= f.min_distinct);
    Blocks {
        low_diversity_suppressed: before - blocks.len(),
        blocks,
        hot_chained,
        stale_skipped: ctx.stale_skipped,
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
    ctx: &mut Ctx<'s>,
) {
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
    if let Some((a0, b0, len)) = extend(sa, a.start_tok, sb, b.start_tok, a.file == b.file)
        && len >= t
    {
        ctx.runs.insert((af.as_str(), a0, bf.as_str(), b0, len));
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
