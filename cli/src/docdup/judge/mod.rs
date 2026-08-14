//! `ce docdup` judgment (design vol.2 §5.3, M5-3g): live cached
//! segments → coarse candidates (LSH ∪ verbatim seeds) → chunks of at
//! most DOC_PAIR_CAP pairs over one core link for the exact Haskell
//! Jaccard re-check → the ONE is_dup verdict throat. Raw inter/union
//! cross the wire, never a ratio; the verbatim half of the verdict is
//! Rust-owned (the runs are computed here), the Jaccard half is the
//! core's 80/100 pinned by the knobs echo.

pub mod candidates;
pub mod wire;

use anyhow::Result;
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
/// VERBATIM_FLOOR — integer cross-multiplication with the mirrored
/// constants the knobs echo pins, OR the Rust-owned verbatim floor.
/// Public: the precision instrument scores census pairs through THIS
/// binding — a re-derived formula would be a second threshold.
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
    let (rows, judged, jaccard_dups, requests) = judge(core, &segs, &cand.pairs)?;
    let runs: BTreeMap<(usize, usize), u64> =
        cand.pairs.iter().map(|&(a, b, r)| ((a, b), r)).collect();
    let dups: Vec<crate::report::Pair<Doc>> = rows
        .iter()
        .filter(|&&(a, b, (inter, union))| is_dup(inter, union, runs[&(a, b)]))
        .map(|&(a, b, (inter, union))| crate::report::Pair {
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

fn name(s: &candidates::SegRow) -> String {
    format!(
        "{}:{}-{} {}",
        s.path,
        s.start_line,
        s.end_line,
        crate::docdup::spec::KIND_NAMES[s.kind as usize]
    )
}

/// Family bindings for the ONE lockstep machine; counter0 = judged,
/// counter1 = jaccardDups.
fn judge(
    core: &str,
    segs: &[candidates::SegRow],
    pairs: &[(usize, usize, u64)],
) -> Result<crate::lockstep::Judged<(u64, u64)>> {
    crate::lockstep::lockstep_scores(
        &wire::family(core),
        pairs,
        |chunk| wire::chunk_request(chunk, |g| &segs[g].set),
        wire::parse_result,
    )
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
