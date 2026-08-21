//! docdup coarse filter (design vol.2 §5.3): live cached segments →
//! MinHash/LSH banding ∪ verbatim seed pairs, every shed a tally.
//! The LSH shape and hot cap are the ONE binding dedup::candidates
//! owns (a second copy would let the two estimators drift), the
//! signatures come from the ONE dedup::minhash implementation, and
//! the verbatim runs are computed on shingle sequences re-derived
//! through the ONE doc_facts throat for exactly the files that host
//! candidates.

mod runs;

use crate::dedup::candidates::{HOT_GROUP_CAP, LSH_SHAPE};
use crate::dedup::{index::Index, minhash};
use anyhow::{Context, Result, ensure};
use runs::runs_for;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One live cached segment — the judgment's unit of identity.
/// `words` is the admitted post-strip word count (the cached column
/// the erase planner's full-segment test divides by; verbatim runs
/// are measured in the same unit, F26).
pub struct SegRow {
    pub path: String,
    pub kind: i64,
    pub start_line: i64,
    pub end_line: i64,
    pub words: i64,
    pub set: Vec<u64>,
}

/// The exempted segments by class, off the same cache — surfaced in
/// the report counts since the batch-7 defect sweep: a license
/// exemption used to be SILENT in `ce docdup` output even though
/// the classification was persisted all along (the bare-marker
/// rejections, allow_missing_why, are ledger-only and not persisted
/// — surfacing them is a schema change, ledgered for batch 8).
pub fn exempt_counts(idx: &Index) -> Result<(u64, u64)> {
    let rows = crate::graph::load::rows(
        idx.raw(),
        "SELECT exempt, COUNT(*) FROM docsegs WHERE exempt != 0 GROUP BY exempt",
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
    )?;
    let pick = |code: i64| {
        rows.iter()
            .find(|(c, _)| *c == code)
            .map(|(_, n)| *n as u64)
            .unwrap_or(0)
    };
    Ok((
        pick(crate::docdup::exempt::EXEMPT_LICENSE),
        pick(crate::docdup::exempt::EXEMPT_ALLOW),
    ))
}

/// Every LIVE admitted segment in identity order, straight off the
/// docsegs cache (exempt segments are outside the corpus by
/// definition — the D6 zero-survival claim is structural here).
pub fn live_rows(idx: &Index) -> Result<Vec<SegRow>> {
    let rows = crate::graph::load::rows(
        idx.raw(),
        "SELECT f.path, d.kind, d.start_line, d.end_line, d.words, d.shingles
         FROM docsegs d JOIN files f ON f.id = d.file_id
         WHERE d.exempt = 0 ORDER BY f.path, d.start_line, d.kind",
        |r| {
            let head: (String, i64, i64, i64, i64) =
                (r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?);
            Ok((head, r.get::<_, Vec<u8>>(5)?))
        },
    )?;
    rows.into_iter()
        .map(|((path, kind, start_line, end_line, words), blob)| {
            let set =
                shingle_set(&blob).with_context(|| format!("{path}:{start_line} docsegs row"))?;
            Ok(SegRow {
                path,
                kind,
                start_line,
                end_line,
                words,
                set,
            })
        })
        .collect()
}

/// Decode one docsegs shingle blob (u64 LE rows): `chunks_exact`
/// silently DROPS a truncated tail, and fewer shingles = fewer
/// candidates = silently missed duplication — so a non-whole row
/// shape is refused by name (review 2026-08-19, codex lane).
fn shingle_set(blob: &[u8]) -> Result<Vec<u64>> {
    let rows = blob.chunks_exact(8);
    ensure!(
        rows.remainder().is_empty(),
        "shingle blob is {} bytes — not whole u64 rows",
        blob.len()
    );
    Ok(rows
        .map(|c| u64::from_le_bytes(c.try_into().expect("8 bytes")))
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
///
/// The chain is a RECALL BOUND, stated the way pairs.rs states its
/// own: above HOT_GROUP_CAP only the n-1 adjacent pairs of a bucket
/// ever reach the judge, so a duplicate pair that shares this bucket
/// and nothing else is never judged and never reported. It is a
/// bound, not a leak: every chained bucket ticks `hot_bands` /
/// `hot_shingles`, and the D4 alternative — dropping the bucket
/// whole — zeroed detection exactly when duplication was highest.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A truncated cache blob is refused, never silently shortened
    /// (the whole-row decode is what every docdup run already rides).
    #[test]
    fn truncated_shingle_blob_is_refused_not_shortened() {
        let err = shingle_set(&[0; 17]).expect_err("truncated").to_string();
        assert!(err.contains("17 bytes"), "{err}");
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
