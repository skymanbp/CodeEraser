//! `ce docdup` judgment (design vol.2 §5.3, M5-3g): live cached
//! segments → coarse candidates (LSH ∪ verbatim seeds) → chunks of at
//! most DOC_PAIR_CAP pairs over one core link for the exact Haskell
//! Jaccard re-check. Raw inter/union cross the wire, never a ratio,
//! and since ADR-008 P1 each score row comes back with the CORE's
//! full verdict bit (CE.Docdup.Cost.dupVerdict: Jaccard ∨ verbatim —
//! the runs computed here ride the request as verdict inputs, F26);
//! the reported set is the core's decision, cross-checked per row
//! against the pinned is_dup mirror.

pub mod candidates;
pub mod wire;

use anyhow::{Result, ensure};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// JSON output schema id; bump on shape change (plan §7.1).
pub const SCHEMA_ID: &str = "ce.docdup-report/0.1.0";

#[derive(Serialize)]
pub struct Counts {
    pub segments: usize,
    /// The candidate pass's own tally rides flattened — one struct
    /// owns those counters, nobody re-declares them.
    #[serde(flatten)]
    pub tally: candidates::Tally,
    pub sent: u64,
    pub requests: usize,
    pub judged: u64,
    pub jaccard_dups: u64,
    pub dups: usize,
}

/// The family metric block riding each reported pair (report::Pair
/// flattens it, so the JSON row shape is unchanged).
#[derive(Serialize)]
pub struct Doc {
    pub inter: u64,
    pub union: u64,
    pub verbatim: u64,
}

pub type Report = crate::report::Report<Doc, Counts>;

/// dup ⇔ inter·JACCARD_DEN ≥ JACCARD_NUM·union ∨ verbatim ≥
/// VERBATIM_FLOOR — since ADR-008 P1 a MIRROR of the core's verdict
/// (CE.Docdup.Cost.dupVerdict), not an authority: the reported set
/// is built from the wire's per-row bits, and this binding remains
/// for run()'s per-row drift ensure. All three numbers are pinned by
/// the knobs echo.
pub fn is_dup(inter: u64, union: u64, verbatim: u64) -> bool {
    inter * wire::JACCARD_DEN >= wire::JACCARD_NUM * union
        || verbatim >= crate::docdup::spec::VERBATIM_FLOOR as u64
}

/// The whole judgment: refresh, live rows, coarse candidates, chunked
/// docdup.requests, verdicts.
pub fn run(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Report> {
    let (idx, _db_path) = crate::dedup::refreshed_index(root, db)?;
    let segs = candidates::live_rows(&idx)?;
    let cand = candidates::collect(root, &segs)?;
    // the family's lockstep bindings, inline: this judge is thin
    // enough that a separate fn was pure scaffolding (bite 17 tail)
    let (rows, judged, jaccard_dups, requests) = crate::lockstep::lockstep_scores(
        &wire::family(core),
        &cand.pairs,
        |chunk| wire::chunk_request(chunk, |g| &segs[g].set),
        wire::parse_result,
    )?;
    let runs: BTreeMap<(usize, usize), u64> =
        cand.pairs.iter().map(|&(a, b, r)| ((a, b), r)).collect();
    // ADR-008 P1: the reported set is built from the CORE's verdict
    // bits; the pinned mirror must agree per row or the run dies
    // loudly — formula drift named, never a silently forked verdict
    for &(a, b, (inter, union, v)) in &rows {
        ensure!(
            v == is_dup(inter, union, runs[&(a, b)]),
            "core docdup verdict ({v}) disagrees with the pinned mirror at J {inter}/{union} run {} — formula drift (Docdup/Cost.hs vs judge/mod.rs)",
            runs[&(a, b)]
        );
    }
    let dups: Vec<crate::report::Pair<Doc>> = rows
        .iter()
        .filter(|&&(_, _, (_, _, v))| v)
        .map(|&(a, b, (inter, union, _))| crate::report::Pair {
            a: name(&segs[a]),
            b: name(&segs[b]),
            m: Doc {
                inter,
                union,
                verbatim: runs[&(a, b)],
            },
        })
        .collect();
    let counts = Counts {
        segments: segs.len(),
        sent: cand.pairs.len() as u64,
        tally: cand.tally,
        requests,
        judged,
        jaccard_dups,
        dups: dups.len(),
    };
    Ok(Report { hits: dups, counts })
}

/// Report emission through the ONE shared envelope+console throat.
pub fn print(r: &Report, as_json: bool) {
    crate::report::emit(
        (SCHEMA_ID, "dups"),
        r,
        as_json,
        "docdup {a} <-> {b}  J {inter}/{union} verbatim {verbatim}",
    );
}

fn name(s: &candidates::SegRow) -> String {
    format!(
        "{}:{}-{} {}",
        s.path,
        s.start_line,
        s.end_line,
        crate::docdup::spec::KIND_NAMES[s.kind as usize]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The verdict boundary in BOTH directions at the 80/100 ratio,
    /// and the verbatim disjunct alone.
    #[test]
    fn verdict_sits_exactly_on_the_threshold() {
        assert!(is_dup(4, 5, 0));
        assert!(!is_dup(3, 4, 0));
        assert!(is_dup(0, 100, 50), "verbatim hard hit ignores jaccard");
        assert!(!is_dup(0, 100, 49));
    }
}
