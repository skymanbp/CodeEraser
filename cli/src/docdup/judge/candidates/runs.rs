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

/// One segment's (shingle sequence, value→positions seed index) — a
/// property of the OPERAND, built once per segment, not per pair (P11).
type Seq = (Vec<u64>, BTreeMap<u64, Vec<usize>>);

fn indexed(seq: Vec<u64>) -> Seq {
    let mut pos = BTreeMap::<u64, Vec<usize>>::new();
    for (j, &y) in seq.iter().enumerate() {
        pos.entry(y).or_default().push(j);
    }
    (seq, pos)
}

/// One exact verbatim run per candidate pair.
pub(super) fn runs_for(
    root: &Path,
    segs: &[SegRow],
    cand: BTreeSet<(usize, usize)>,
) -> Result<Vec<(usize, usize, u64)>> {
    let mut seqs: BTreeMap<usize, Seq> = BTreeMap::new();
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
            seqs.insert(i, indexed(docdup::shingle::shingle_seq(&fact.words)));
        }
    }
    Ok(cand
        .into_iter()
        .map(|(a, b)| (a, b, run_words(&seqs[&a], &seqs[&b])))
        .collect())
}

/// Longest common contiguous shingle run in WORDS (R shingles span
/// R + DOC_SHINGLE − 1 words), by seed-extension: start only where a
/// run cannot extend left, walk right — each maximal run measured
/// once. The oracle's independent DP is what D2 checks this against.
fn run_words((a, _): &Seq, (b, pos): &Seq) -> u64 {
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
#[path = "../../../../tests/unit/docdup/judge/candidates/runs.rs"]
mod tests;
