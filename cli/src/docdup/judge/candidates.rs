//! docdup coarse filter (design vol.2 §5.3): live cached segments →
//! MinHash/LSH banding ∪ verbatim seed pairs, every shed a tally.
//! The LSH shape and hot cap are the ONE binding dedup::candidates
//! owns (a second copy would let the two estimators drift), the
//! signatures come from the ONE dedup::minhash implementation, and
//! the verbatim runs are computed on shingle sequences re-derived
//! through the ONE doc_facts throat for exactly the files that host
//! candidates.

use crate::dedup::candidates::{HOT_GROUP_CAP, LSH_SHAPE};
use crate::dedup::{index::Index, minhash};
use crate::docdup::{self, spec};
use anyhow::{Context, Result, ensure};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One live cached segment — the judgment's unit of identity.
pub struct SegRow {
    pub path: String,
    pub kind: i64,
    pub start_line: i64,
    pub end_line: i64,
    pub set: Vec<u64>,
}

/// Every LIVE admitted segment in identity order, straight off the
/// docsegs cache (exempt segments are outside the corpus by
/// definition — the D6 zero-survival claim is structural here).
pub fn live_rows(idx: &Index) -> Result<Vec<SegRow>> {
    let rows = crate::graph::load::rows(
        idx.raw(),
        "SELECT f.path, d.kind, d.start_line, d.end_line, d.shingles
         FROM docsegs d JOIN files f ON f.id = d.file_id
         WHERE d.exempt = 0 ORDER BY f.path, d.start_line, d.kind",
        |r| {
            let head: (String, i64, i64, i64) = (r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?);
            Ok((head, r.get::<_, Vec<u8>>(4)?))
        },
    )?;
    Ok(rows
        .into_iter()
        .map(|((path, kind, start_line, end_line), blob)| SegRow {
            path,
            kind,
            start_line,
            end_line,
            set: blob
                .chunks_exact(8)
                .map(|c| u64::from_le_bytes(c.try_into().expect("8-byte chunk")))
                .collect(),
        })
        .collect())
}

/// The candidate pass output: pairs with their exact verbatim runs,
/// plus the tally of every bound that fired.
pub struct Cand {
    pub pairs: Vec<(usize, usize, u64)>,
    pub tally: Tally,
}

/// Serialized flattened into the report Counts — ONE owner for these
/// counters, no re-declaration at the report layer.
#[derive(Default, serde::Serialize)]
pub struct Tally {
    pub over_cap_segments: u64,
    pub lsh_pairs: u64,
    pub seed_pairs: u64,
    pub hot_bands: u64,
    pub hot_shingles: u64,
}

/// LSH bucket pairs ∪ shared-shingle seed pairs, with hot groups
/// chained instead of skipped (pairs.rs HOT_CAP / attack-review D4:
/// skipping hot groups made detection fall to zero) — then one exact
/// verbatim run per candidate.
pub fn collect(root: &Path, segs: &[SegRow]) -> Result<Cand> {
    let mut tally = Tally::default();
    let mut cand: BTreeSet<(usize, usize)> = BTreeSet::new();
    let (perms, bands, rows_per) = LSH_SHAPE;
    let mut buckets: BTreeMap<(usize, u64), Vec<usize>> = BTreeMap::new();
    let sendable: Vec<usize> = (0..segs.len())
        .filter(|&i| {
            let ok = segs[i].set.len() <= super::wire::DOC_SET_CAP;
            tally.over_cap_segments += u64::from(!ok);
            ok
        })
        .collect();
    for &i in &sendable {
        let sig = minhash::signature(&segs[i].set, perms);
        for key in minhash::band_keys(&sig, bands, rows_per) {
            buckets.entry(key).or_default().push(i);
        }
    }
    tally.lsh_pairs = group_pairs(buckets.values(), &mut tally.hot_bands, &mut cand);
    let mut inverted: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for &i in &sendable {
        for h in &segs[i].set {
            inverted.entry(*h).or_default().push(i);
        }
    }
    tally.seed_pairs = group_pairs(inverted.values(), &mut tally.hot_shingles, &mut cand);
    let pairs = runs_for(root, segs, cand)?;
    Ok(Cand { pairs, tally })
}

/// All-pairs under the hot cap, adjacent chains above it (counted);
/// returns how many pairs this source contributed.
fn group_pairs<'v>(
    groups: impl Iterator<Item = &'v Vec<usize>>,
    hot: &mut u64,
    cand: &mut BTreeSet<(usize, usize)>,
) -> u64 {
    let mut added = 0;
    for list in groups {
        let pairs: Vec<(usize, usize)> = if list.len() <= HOT_GROUP_CAP {
            (0..list.len())
                .flat_map(|a| ((a + 1)..list.len()).map(move |b| (list[a], list[b])))
                .collect()
        } else {
            *hot += 1;
            list.windows(2).map(|w| (w[0], w[1])).collect()
        };
        for (a, b) in pairs {
            if a != b && cand.insert((a.min(b), a.max(b))) {
                added += 1;
            }
        }
    }
    added
}

/// Exact verbatim runs for the candidate set: shingle SEQUENCES are
/// re-derived per hosting file through the product doc_facts throat
/// (the cache stores only the deduped set), then each pair's longest
/// common contiguous run is measured by seed-extension.
fn runs_for(
    root: &Path,
    segs: &[SegRow],
    cand: BTreeSet<(usize, usize)>,
) -> Result<Vec<(usize, usize, u64)>> {
    let mut seqs: BTreeMap<usize, Vec<u64>> = BTreeMap::new();
    let mut by_file: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for &(a, b) in &cand {
        for i in [a, b] {
            by_file.entry(&segs[i].path).or_default().push(i);
        }
    }
    for (path, ids) in by_file {
        let (text, lang) = crate::dedup::walked_text(root, path)?;
        let facts = docdup::doc_facts(&text, lang);
        for i in ids {
            let s = &segs[i];
            let fact = facts
                .segs
                .iter()
                .find(|f| (f.kind, f.start_line, f.end_line) == (s.kind, s.start_line, s.end_line))
                .with_context(|| {
                    format!(
                        "{path}:{} — disk drifted from the docsegs cache",
                        s.start_line
                    )
                })?;
            // same-source counterfactual in the product path: the
            // re-derived set must equal the cached one byte for byte
            ensure!(
                fact.shingles == s.set,
                "{path}:{}: re-derived shingle set differs from the cache",
                s.start_line
            );
            seqs.insert(i, docdup::shingle::shingle_seq(&fact.words));
        }
    }
    Ok(cand
        .into_iter()
        .map(|(a, b)| (a, b, run_words(&seqs[&a], &seqs[&b])))
        .collect())
}

/// Longest common contiguous shingle run in WORDS (R shingles span
/// R + DOC_SHINGLE − 1 words), by seed-extension: start only where a
/// run cannot extend left, walk right — each maximal run is measured
/// exactly once. This is the product's own computation; the oracle's
/// independent DP is what D2 checks it against.
fn run_words(a: &[u64], b: &[u64]) -> u64 {
    let mut pos: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for (j, &y) in b.iter().enumerate() {
        pos.entry(y).or_default().push(j);
    }
    let mut best = 0usize;
    for (i, &x) in a.iter().enumerate() {
        for &j in pos.get(&x).map_or(&Vec::new(), |v| v) {
            if i > 0 && j > 0 && a[i - 1] == b[j - 1] {
                continue; // not a run start; counted from its start
            }
            let n = a[i..]
                .iter()
                .zip(&b[j..])
                .take_while(|(x2, y2)| x2 == y2)
                .count();
            best = best.max(n);
        }
    }
    if best == 0 {
        0
    } else {
        (best + spec::DOC_SHINGLE - 1) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The run measure against hand-derived cases: disjoint, full
    /// overlap, and an interior run — in words, not shingles.
    #[test]
    fn run_words_measures_maximal_runs_in_words() {
        assert_eq!(run_words(&[1, 2, 3], &[4, 5, 6]), 0);
        let k = spec::DOC_SHINGLE as u64;
        assert_eq!(run_words(&[1, 2, 3], &[1, 2, 3]), 3 + k - 1);
        assert_eq!(run_words(&[9, 1, 2, 8], &[7, 1, 2, 6]), 2 + k - 1);
        // repeated values: extension must not double-count seeds
        assert_eq!(run_words(&[5, 5, 5], &[5, 5]), 2 + k - 1);
    }

    /// Hot groups chain instead of skipping (D4) and the union dedups
    /// across sources.
    #[test]
    fn hot_groups_chain_and_pairs_dedup() {
        let mut cand = BTreeSet::new();
        let mut hot = 0;
        let big: Vec<usize> = (0..HOT_GROUP_CAP + 2).collect();
        let n = group_pairs([big.clone()].iter(), &mut hot, &mut cand);
        assert_eq!(hot, 1);
        assert_eq!(n as usize, HOT_GROUP_CAP + 1, "adjacent chain");
        let again = group_pairs([big].iter(), &mut hot, &mut cand);
        assert_eq!(again, 0, "second source adds nothing new");
    }
}
