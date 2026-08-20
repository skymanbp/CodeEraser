//! Exact verbatim-run measurement for the candidate set (split from
//! candidates.rs when the shingle-shape refusal pushed it past its
//! ratchet ceiling): shingle SEQUENCES are re-derived per hosting
//! file through the product doc_facts throat (the cache stores only
//! the deduped set), then each pair's longest common contiguous run
//! is measured by seed-extension.

use super::SegRow;
use crate::docdup::{self, spec};
use anyhow::{Context, Result, ensure};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One exact verbatim run per candidate pair.
pub(super) fn runs_for(
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
}
