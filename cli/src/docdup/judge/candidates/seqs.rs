//! The shingle SEQUENCES of the segments a candidate set names (split
//! from candidates.rs when the shingle-shape refusal pushed it past its
//! ratchet ceiling): the cache stores only each segment's deduplicated
//! set and a verbatim run needs order, so the sequences are re-derived
//! per hosting file through the product doc_facts throat, each checked
//! against the cached set byte for byte. The judgment sends them to the
//! core, which measures each pair's longest common contiguous run by
//! seed-extension (CE.Docdup.Runs, plan v2.33 W3).

use super::SegRow;
use crate::docdup;
use anyhow::{Context, Result, ensure};
use std::collections::BTreeMap;
use std::path::Path;

/// One sequence per segment either side of a candidate pair.
pub(super) fn seqs_for(
    root: &Path,
    segs: &[SegRow],
    pairs: &[(usize, usize)],
) -> Result<BTreeMap<usize, Vec<u64>>> {
    let mut seqs: BTreeMap<usize, Vec<u64>> = BTreeMap::new();
    let mut by_file: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for &(a, b) in pairs {
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
    Ok(seqs)
}
