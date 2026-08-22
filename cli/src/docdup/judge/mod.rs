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

use anyhow::{Context, Result, ensure};
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
    /// Exempted segments by class (batch-7 defect sweep): the
    /// persisted classification, no longer silent in the report.
    pub exempt_license: u64,
    pub exempt_allow: u64,
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
/// docdup.requests, verdicts — rendered as the report's display pairs.
pub fn run(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Report> {
    let (segs, dups, counts) = run_rows(root, db, core)?;
    let hits = dups
        .into_iter()
        .map(|(a, b, m)| crate::report::Pair {
            a: name(&segs[a]),
            b: name(&segs[b]),
            m,
        })
        .collect();
    Ok(Report { hits, counts })
}

/// The structured judgment: segment table, dup pairs as indices
/// into it, counters.
pub type Rows = (Vec<candidates::SegRow>, Vec<(usize, usize, Doc)>, Counts);

/// The structured face of the SAME judgment: the live segment table
/// plus the core-reported duplicate pairs as indices into it. The
/// erase planner consumes the spans and word counts the display
/// strings drop — one judgment, two faces, never a re-derivation.
pub fn run_rows(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Rows> {
    let (idx, _db_path) = crate::dedup::refreshed_index(root, db)?;
    rows_of(root, &idx, core)
}

/// The same judgment from an index the command boundary already
/// refreshed and opened (batch 9 P10) — the erase gather's leg.
pub fn rows_of(root: &Path, idx: &crate::dedup::index::Index, core: &str) -> Result<Rows> {
    let segs = candidates::live_rows(idx)?;
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
    let dups = reported_dups(&rows, &runs)?;
    let (exempt_license, exempt_allow) = candidates::exempt_counts(idx)?;
    let counts = Counts {
        segments: segs.len(),
        sent: cand.pairs.len() as u64,
        tally: cand.tally,
        requests,
        judged,
        jaccard_dups,
        dups: dups.len(),
        exempt_license,
        exempt_allow,
    };
    Ok((segs, dups, counts))
}

/// The reported set from the CORE's verdict bits (ADR-008 P1), with
/// the per-row drift ensure — the pinned mirror must agree or the
/// run dies loudly (formula drift named, never a silently forked
/// verdict) — in one defensive pass (review C20: the runs[..]
/// indexings were the last decode site that panicked instead of
/// erroring on an unexpected pair echo). Split from run() at the
/// E01 line, the t3::reported_clones shape.
fn reported_dups(
    rows: &[(usize, usize, (u64, u64, bool))],
    runs: &BTreeMap<(usize, usize), u64>,
) -> Result<Vec<(usize, usize, Doc)>> {
    let mut dups = Vec::new();
    for &(a, b, (inter, union, v)) in rows {
        let run = *runs
            .get(&(a, b))
            .with_context(|| format!("core echoed pair ({a},{b}) that was never sent"))?;
        ensure!(
            v == is_dup(inter, union, run),
            "core docdup verdict ({v}) disagrees with the pinned mirror at J {inter}/{union} run {run} — formula drift (Docdup/Cost.hs vs judge/mod.rs)"
        );
        if v {
            dups.push((
                a,
                b,
                Doc {
                    inter,
                    union,
                    verbatim: run,
                },
            ));
        }
    }
    Ok(dups)
}

/// Report emission through the ONE shared envelope+console throat.
pub fn print(r: &Report, as_json: bool) {
    crate::report::emit(
        (SCHEMA_ID, "dups"),
        r,
        as_json,
        crate::i18n::t(
            "docdup {a} <-> {b}  J {inter}/{union} verbatim {verbatim}",
            "文档重复 {a} <-> {b}  J {inter}/{union} 逐字 {verbatim}",
        ),
        crate::i18n::t(
            "{dups} duplicate pair(s) over {segments} live segment(s) — {judged} judged, {jaccard_dups} by Jaccard",
            "{dups} 对文档重复 / {segments} 个活段 — 判决 {judged}，其中 Jaccard 命中 {jaccard_dups}",
        ),
    );
}

fn name(s: &candidates::SegRow) -> String {
    // .get, not a subscript: `kind` is a stored db column, and a
    // stale or corrupt `.ce/index.db` carrying a kind past this
    // side's vocabulary would abort a report rather than name the
    // row (the deadcode VERDICT_NAMES sibling, same class).
    let kind = crate::docdup::spec::KIND_NAMES
        .get(s.kind as usize)
        .copied()
        .unwrap_or("kind?");
    format!("{}:{}-{} {}", s.path, s.start_line, s.end_line, kind)
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
