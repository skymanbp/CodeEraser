//! docdup coarse filter (design vol.2 §5.3): live cached segments →
//! the core's coarse filter (docpairs/1, plan v2.33 W3: MinHash/LSH
//! banding ∪ shared-shingle seed pairs, hot groups chained, every shed
//! a tally — CE.Docdup.Coarse) → the shingle sequences of exactly the
//! segments the kept pairs name, re-derived through the ONE doc_facts
//! throat (seqs.rs) so the judgment can measure their verbatim runs.
//! A segment over the package's `doc_set_cap` is counted here and
//! never sent.

mod seqs;

use crate::corelink::{Link, judged};
use crate::dedup::index::Index;
use anyhow::{Context, Result, ensure};
use std::collections::BTreeMap;
use std::path::Path;

/// The coarse filter's capability, request kind, and minting proto.
pub const CAP: &str = "docpairs/1";
const KIND: &str = "docpairs";
const SINCE: &str = "8.4.0";

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
    // an own file's exemptions only — a foreign README's license block
    // is outside this tree's docdup domain like its live prose (step
    // #12; the codex review found the count reading every owner)
    let rows = crate::graph::load::rows(
        idx.raw(),
        "SELECT d.exempt, COUNT(*) FROM docsegs d JOIN files f ON f.id = d.file_id
         WHERE d.exempt != 0 AND f.owner = 0 GROUP BY d.exempt",
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

/// The prose-only files of the index (plan v2.30 step 5b-8): this
/// tree's own `.txt` files — docdup corpus members that are no graph
/// node, which the check score seats after the graph's files.
pub fn prose_files(idx: &Index) -> Result<Vec<String>> {
    let paths: Vec<String> = crate::graph::load::rows(
        idx.raw(),
        "SELECT path FROM files WHERE owner = 0 ORDER BY path",
        |r| r.get(0),
    )?;
    Ok(paths
        .into_iter()
        .filter(|p| crate::scan::lang::Lang::prose_path(Path::new(p)))
        .collect())
}

/// Every LIVE admitted segment of an OWN file in identity order,
/// straight off the docsegs cache (exempt segments are outside the
/// corpus by definition — the D6 zero-survival claim is structural
/// here; a foreign file's prose is its own tree's to judge).
pub fn live_rows(idx: &Index) -> Result<Vec<SegRow>> {
    let rows = crate::graph::load::rows(
        idx.raw(),
        "SELECT f.path, d.kind, d.start_line, d.end_line, d.words, d.shingles
         FROM docsegs d JOIN files f ON f.id = d.file_id
         WHERE d.exempt = 0 AND f.owner = 0 ORDER BY f.path, d.start_line, d.kind",
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

/// The candidate pass output: the kept pairs (segment ids ascending),
/// each pair's segments' shingle sequences, and the tally of every
/// bound that fired.
pub struct Cand {
    pub pairs: Vec<(usize, usize)>,
    pub seqs: BTreeMap<usize, Vec<u64>>,
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

/// The sendable segments (within the package's per-set ceiling, the
/// rest counted) to the core's coarse filter, its kept pairs mapped
/// back to segment ids, then the sequences of the segments they name.
/// A degraded or skewed reply is a named refusal — the judgment never
/// runs on a candidate set the core did not answer whole.
pub fn collect(root: &Path, segs: &[SegRow], link: &mut Link) -> Result<Cand> {
    let mut tally = Tally::default();
    let cap = super::wire::limits().doc_set_cap;
    let sendable: Vec<usize> = (0..segs.len())
        .filter(|&i| {
            let ok = segs[i].set.len() <= cap;
            tally.over_cap_segments += u64::from(!ok);
            ok
        })
        .collect();
    let sets: Vec<&[u64]> = sendable.iter().map(|&i| segs[i].set.as_slice()).collect();
    let keys = ["sets", "lshPairs", "seedPairs", "hotBands", "hotShingles"];
    let (_, local, counts): (_, Vec<[usize; 2]>, _) = judged::whole_pass(
        link,
        (CAP, SINCE, KIND),
        serde_json::json!({ "sets": sets }),
        ("docdup coarse filter", &keys),
    )?;
    let [n_sets, lsh, seed, hot_bands, hot_shingles] = counts[..] else {
        unreachable!("one count per key")
    };
    ensure!(
        n_sets == sendable.len() as u64
            && lsh + seed == local.len() as u64
            && local.windows(2).all(|w| w[0] < w[1])
            && local.iter().all(|&[a, b]| a < b && b < sendable.len()),
        "docpairs/1 reply does not add up: wire skew"
    );
    (
        tally.lsh_pairs,
        tally.seed_pairs,
        tally.hot_bands,
        tally.hot_shingles,
    ) = (lsh, seed, hot_bands, hot_shingles);
    let pairs: Vec<(usize, usize)> = local
        .into_iter()
        .map(|[a, b]| (sendable[a], sendable[b]))
        .collect();
    let seqs = seqs::seqs_for(root, segs, &pairs)?;
    Ok(Cand { pairs, seqs, tally })
}

#[cfg(test)]
#[path = "../../../tests/unit/docdup/judge/candidates.rs"]
mod tests;
