//! M5-1c-ii shadow ablation engine — a Rust mirror of the core's L2
//! judgment (CE.FourClass.Anchor `sites` + Provenance phases 2/3)
//! over the exact `leftovers()` run structure the wire ships. It
//! exists to measure judgment VARIANTS offline on frozen corpora
//! without touching the Haskell core (a semantic core change needs a
//! plan revision first; this instrument supplies the data that picks
//! the winner — user decision 2026-08-11, ccm #470).
//!
//! Fidelity is asserted, not assumed: the generator requires the
//! BASELINE shadow to equal the live ce-core delta on every commit
//! of every corpus, so any drift between this mirror and the core is
//! a loud generation failure, never a silently wrong matrix.
#![allow(dead_code)]

pub mod ledgers;
pub mod variants;

use crate::eval_l2_parts as parts;
use codeeraser::fourclass::batch::Side;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// (pair index, 1-based line on its own side).
pub type Mark = (usize, usize);

/// Mirror of CE.FourClass.Cost.destFloor — derived there from the
/// four cost constants and pinned by core/test/Spec.hs; a value
/// drift here fails the generation-time equivalence assert.
pub const DEST_FLOOR: usize = 2;

/// Mirror of CE.FourClass.Cost.anchorFloor (M5-1c-iii: a site needs
/// one evidence line this wide; the ablation's quality variant at 20
/// remains as the one-notch-stricter control).
pub const ANCHOR_FLOOR: usize = 19;

/// Mirror of CE.FourClass.Anchor.bucketCap.
pub const BUCKET_CAP: usize = 64;

/// Mirror of CE.FourClass.Wire.Block, plus the positional evidence
/// hashes the variants judge on (the core never sends them back;
/// the shadow keeps them so filters can look content up).
#[derive(Clone, Debug)]
pub struct ShadowBlock {
    pub from_pair: usize,
    pub from_lines: Vec<usize>,
    pub to_pair: usize,
    pub to_lines: Vec<usize>,
    pub hashes: Vec<u64>,
}

/// One side occurrence of a hash: (pair, run id, position in run).
struct Occ {
    pair: usize,
    run: usize,
    pos: usize,
}

fn build_ix(sent: &[(Side, Side)], rem: bool) -> HashMap<u64, Vec<Occ>> {
    let mut ix: HashMap<u64, Vec<Occ>> = HashMap::new();
    for (p, pair) in sent.iter().enumerate() {
        let side = if rem { &pair.0 } else { &pair.1 };
        for (r, run) in side.iter().enumerate() {
            for (i, &(_, h, _)) in run.iter().enumerate() {
                ix.entry(h).or_default().push(Occ {
                    pair: p,
                    run: r,
                    pos: i,
                });
            }
        }
    }
    ix
}

/// Anchor.hs `sites`: all accepted cross-pair blocks plus whether the
/// bucket cap tripped. No exclusivity, no tie-break — a union of
/// independently derived sets, order-independent by construction
/// (sorted canonically only so ledgers are stable).
pub fn sites(sent: &[(Side, Side)]) -> (Vec<ShadowBlock>, bool) {
    let rem_ix = build_ix(sent, true);
    let add_ix = build_ix(sent, false);
    // Per-hash pairing budget (Anchor.hs `overWork`): the removed x
    // added occurrence product is the work the enumeration spends;
    // one-sided piles and small-x-large boilerplate buckets (ripgrep
    // b9de003f8, 082245dad) have tiny products and judge normally.
    let over_work = |r: &[Occ], a: &[Occ]| r.len() * a.len() > BUCKET_CAP * BUCKET_CAP;
    let empty: Vec<Occ> = Vec::new();
    let capped = rem_ix
        .iter()
        .any(|(h, v)| over_work(v, add_ix.get(h).unwrap_or(&empty)));
    let mut blocks: Vec<ShadowBlock> = Vec::new();
    for (h, rem_occs) in &rem_ix {
        let Some(add_occs) = add_ix.get(h) else {
            continue;
        };
        if over_work(rem_occs, add_occs) {
            continue;
        }
        for ro in rem_occs {
            for ao in add_occs {
                if ro.pair != ao.pair {
                    blocks.extend(try_block(sent, ro, ao));
                }
            }
        }
    }
    canon(&mut blocks);
    (blocks, capped)
}

/// Canonical block order (Provenance's sortedBlocks) — sites and the
/// flow filter share one ordering so ledgers stay stable.
pub fn canon(blocks: &mut [ShadowBlock]) {
    blocks.sort_by(|x, y| {
        (x.from_pair, &x.from_lines, x.to_pair, &x.to_lines).cmp(&(
            y.from_pair,
            &y.from_lines,
            y.to_pair,
            &y.to_lines,
        ))
    });
}

/// Scan one side's flattened (line, hash, width) entries and keep the
/// marks the predicate accepts — the shared tail of both phase passes.
fn side_marks(
    sent: &[(Side, Side)],
    rem: bool,
    mut keep: impl FnMut(usize, usize, u64) -> bool,
) -> BTreeSet<Mark> {
    let mut out = BTreeSet::new();
    for (p, pair) in sent.iter().enumerate() {
        let side = if rem { &pair.0 } else { &pair.1 };
        for &(l, h, _) in side.iter().flatten() {
            if keep(p, l, h) {
                out.insert((p, l));
            }
        }
    }
    out
}

/// Anchor.hs `tryBlock`: extend forward from a block START (interior
/// positions have equal predecessors and fail the start test, so each
/// block is discovered exactly once). The floor counts DISTINCT
/// content values, not lines (attack review F5), and the evidence
/// must include one ANCHOR line >= ANCHOR_FLOOR wide (M5-1c-iii).
fn try_block(sent: &[(Side, Side)], ro: &Occ, ao: &Occ) -> Option<ShadowBlock> {
    let r_run = &sent[ro.pair].0[ro.run];
    let a_run = &sent[ao.pair].1[ao.run];
    let is_start = ro.pos == 0 || ao.pos == 0 || r_run[ro.pos - 1].1 != a_run[ao.pos - 1].1;
    if !is_start {
        return None;
    }
    let (r_tail, a_tail) = (&r_run[ro.pos..], &a_run[ao.pos..]);
    let n = r_tail
        .iter()
        .zip(a_tail)
        .take_while(|(r, a)| r.1 == a.1)
        .count();
    let evidence = &r_tail[..n];
    let hashes: Vec<u64> = evidence.iter().map(|&(_, h, _)| h).collect();
    let distinct: HashSet<u64> = hashes.iter().copied().collect();
    let anchored = evidence.iter().any(|&(_, _, w)| w >= ANCHOR_FLOOR);
    (distinct.len() >= DEST_FLOOR && anchored).then(|| ShadowBlock {
        from_pair: ro.pair,
        from_lines: evidence.iter().map(|&(l, _, _)| l).collect(),
        to_pair: ao.pair,
        to_lines: a_tail[..n].iter().map(|&(l, _, _)| l).collect(),
        hashes,
    })
}

/// Provenance.hs `phase2` — run-scoped destination extension: an
/// unclaimed added line, inside a run that already holds an anchored
/// line, whose hash occurs among the removals of a pair with an
/// established edge into this pair.
fn phase2(sent: &[(Side, Side)], blocks: &[ShadowBlock]) -> BTreeSet<Mark> {
    let anchored: BTreeSet<Mark> = blocks
        .iter()
        .flat_map(|b| b.to_lines.iter().map(|&l| (b.to_pair, l)))
        .collect();
    let mut run_id: HashMap<Mark, usize> = HashMap::new();
    for (q, pair) in sent.iter().enumerate() {
        for (r, run) in pair.1.iter().enumerate() {
            for &(l, _, _) in run {
                run_id.insert((q, l), r);
            }
        }
    }
    let hot: HashSet<Mark> = anchored
        .iter()
        .map(|&(q, l)| (q, run_id[&(q, l)]))
        .collect();
    let edges: HashSet<(usize, usize)> = blocks.iter().map(|b| (b.from_pair, b.to_pair)).collect();
    let rem_hashes: Vec<HashSet<u64>> = sent
        .iter()
        .map(|p| p.0.iter().flatten().map(|&(_, h, _)| h).collect())
        .collect();
    side_marks(sent, false, |q, l, h| {
        let fed = edges
            .iter()
            .any(|&(src, dst)| dst == q && rem_hashes[src].contains(&h));
        !anchored.contains(&(q, l)) && hot.contains(&(q, run_id[&(q, l)])) && fed
    })
}

/// Provenance.hs `phase3` — asymmetric source attribution: a leftover
/// removed line whose content landed at a marked-in line of a
/// DIFFERENT pair is moved-out. `edge_required` is the ablation knob
/// (attack review F4): the core requires only `dst /= pp`; the width
/// probe additionally demands an anchored edge (pp -> dst).
fn phase3(
    sent: &[(Side, Side)],
    in_marks: &BTreeSet<Mark>,
    edges: &HashSet<(usize, usize)>,
    edge_required: bool,
) -> BTreeSet<Mark> {
    let mut add_hash: HashMap<Mark, u64> = HashMap::new();
    for (q, pair) in sent.iter().enumerate() {
        for &(l, h, _) in pair.1.iter().flatten() {
            add_hash.insert((q, l), h);
        }
    }
    let mut landed: HashMap<usize, HashSet<u64>> = HashMap::new();
    for &(q, l) in in_marks {
        if let Some(&h) = add_hash.get(&(q, l)) {
            landed.entry(q).or_default().insert(h);
        }
    }
    side_marks(sent, true, |pp, _, h| {
        landed.iter().any(|(&dst, hs)| {
            dst != pp && hs.contains(&h) && (!edge_required || edges.contains(&(pp, dst)))
        })
    })
}

/// The assembled moved marks, mirroring Provenance.hs `classify`:
/// in = block destinations + phase-2 extension, out = block sources
/// + phase-3 attribution over the full in-mark set.
pub struct Moved {
    pub outs: BTreeSet<Mark>,
    pub ins: BTreeSet<Mark>,
}

pub fn moved(sent: &[(Side, Side)], blocks: &[ShadowBlock], edge_required: bool) -> Moved {
    let mark = |from: bool| -> BTreeSet<Mark> {
        blocks
            .iter()
            .flat_map(|b| {
                let (p, ls) = if from {
                    (b.from_pair, &b.from_lines)
                } else {
                    (b.to_pair, &b.to_lines)
                };
                ls.iter().map(move |&l| (p, l))
            })
            .collect()
    };
    let edges: HashSet<(usize, usize)> = blocks.iter().map(|b| (b.from_pair, b.to_pair)).collect();
    let mut ins = mark(false);
    ins.extend(phase2(sent, blocks));
    let mut outs = mark(true);
    outs.extend(phase3(sent, &ins, &edges, edge_required));
    Moved { outs, ins }
}

/// The shadow's per-file delta in the same (side, file) -> sorted
/// lines shape as parts::delta_lines — the equivalence comparand.
pub fn delta(texts: &parts::Texts, m: &Moved) -> BTreeMap<parts::FileKey, Vec<usize>> {
    let mut out: BTreeMap<parts::FileKey, Vec<usize>> = BTreeMap::new();
    let mut push = |side: &str, p: usize, l: usize| {
        let key = if side == "out" { "before" } else { "after" };
        let file = texts[p].2[key].as_str().expect(key);
        out.entry((side.into(), file.into())).or_default().push(l);
    };
    for &(p, l) in &m.outs {
        push("out", p, l);
    }
    for &(p, l) in &m.ins {
        push("in", p, l);
    }
    out.values_mut().for_each(|v| v.sort_unstable());
    out
}
