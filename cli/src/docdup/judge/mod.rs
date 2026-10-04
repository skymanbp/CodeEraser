//! `ce docdup` judgment (design vol.2 §5.3, M5-3g): live cached
//! segments → the core's coarse candidates (LSH ∪ shared-shingle seeds,
//! docpairs/1) → chunks of at most the package's docdup `doc_pair_cap`
//! pairs over the same core link for the exact Haskell Jaccard
//! re-check. Each chunk carries its segments' shingle sequences; the
//! core derives the sets, measures each pair's verbatim run (plan v2.33
//! W3) and answers raw inter/union, the run and, since ADR-008 P1, the
//! CORE's full verdict bit (CE.Docdup.Cost.dupVerdict: Jaccard ∨
//! verbatim); the reported set is the core's decision, and this side
//! holds no copy of the threshold. The report is the core's
//! (document.rs).

pub mod candidates;
mod document;
pub mod wire;

use anyhow::{Result, ensure};
use std::path::{Path, PathBuf};

/// The judgment's counters, sent to the core as facts.
pub struct Counts {
    /// The candidate pass's own tally — one struct owns those
    /// counters, nobody re-declares them.
    pub tally: candidates::Tally,
    pub sent: u64,
    pub requests: usize,
    pub judged: u64,
    pub jaccard_dups: u64,
    /// Exempted segments by class (batch-7 defect sweep): the
    /// persisted classification, no longer silent in the report.
    pub exempt_license: u64,
    pub exempt_allow: u64,
}

/// A duplicate pair's metric: raw intersection and union and the
/// verbatim run.
pub struct Doc {
    pub inter: u64,
    pub union: u64,
    pub verbatim: u64,
}

/// The judged universe: the live segments, the pairs the core called
/// duplicates (segment indices with their metric), every judged row as
/// the report sends it — [a, b, inter, union, run, verdict] — and the
/// counters.
pub struct Judged {
    pub segs: Vec<candidates::SegRow>,
    pub dups: Vec<(usize, usize, Doc)>,
    judged: Vec<[i64; 6]>,
    pub counts: Counts,
}

/// The whole judgment: refresh, live rows, coarse candidates, chunked
/// docdup.requests, verdicts.
pub fn run(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Judged> {
    let (idx, _db_path) = crate::dedup::refreshed_index(root, db)?;
    Ok(judged_over(root, &idx, core)?.0)
}

/// `ce docdup`: the judgment, its report laid out by the core over the
/// link it was judged over (plan v2.32 step 5); `check` is the
/// console's `--check`.
pub fn answer(
    root: &Path,
    db: Option<PathBuf>,
    core: &str,
    check: bool,
) -> Result<crate::document::Answer> {
    let (idx, _db_path) = crate::dedup::refreshed_index(root, db)?;
    let (judged, link) = judged_over(root, &idx, core)?;
    document::report(core, &judged, check, Ok(link))
}

/// The structured judgment: segment table, dup pairs as indices
/// into it, counters.
pub type Rows = (Vec<candidates::SegRow>, Vec<(usize, usize, Doc)>, Counts);

/// The same judgment from an index the command boundary already
/// refreshed and opened (batch 9 P10) — the erase gather's, the
/// check's and the query facts' leg.
pub fn rows_of(root: &Path, idx: &crate::dedup::index::Index, core: &str) -> Result<Rows> {
    let (j, _link) = judged_over(root, idx, core)?;
    Ok((j.segs, j.dups, j.counts))
}

/// The judgment and the link it was judged over.
fn judged_over(
    root: &Path,
    idx: &crate::dedup::index::Index,
    core: &str,
) -> Result<(Judged, crate::corelink::Link)> {
    let segs = candidates::live_rows(idx)?;
    // the family's lockstep bindings, inline: this judge is thin
    // enough that a separate fn was pure scaffolding (bite 17 tail)
    let fam = wire::family(core);
    let mut link = crate::lockstep::open_family(fam.core, fam.cap)?;
    let cand = candidates::collect(root, &segs, &mut link)?;
    let (rows, judged, jaccard_dups, requests) = crate::lockstep::lockstep_scores(
        &mut link,
        &fam,
        &cand.pairs,
        |chunk| wire::chunk_request(chunk, |g| &cand.seqs[&g]),
        wire::parse_result,
    )?;
    let judged_rows = reported_rows(&rows, &cand.pairs)?;
    let dups = judged_rows
        .iter()
        .filter(|r| r[5] == 1)
        .map(|r| {
            let m = |k: usize| r[k] as u64;
            (
                r[0] as usize,
                r[1] as usize,
                Doc {
                    inter: m(2),
                    union: m(3),
                    verbatim: m(4),
                },
            )
        })
        .collect();
    let (exempt_license, exempt_allow) = candidates::exempt_counts(idx)?;
    let counts = Counts {
        sent: cand.pairs.len() as u64,
        tally: cand.tally,
        requests,
        judged,
        jaccard_dups,
        exempt_license,
        exempt_allow,
    };
    let judged = Judged {
        segs,
        dups,
        judged: judged_rows,
        counts,
    };
    Ok((judged, link))
}

/// Every judged row with the run the core measured, the CORE's verdict
/// bit last (ADR-008 P1) — the reported set is the rows whose bit is
/// set — in one defensive pass (review C20: an echoed pair that was
/// never sent is an error, never a row). Split from run() at the E01
/// line, the t3::reported_clones shape.
fn reported_rows(
    rows: &[(usize, usize, (u64, u64, u64, bool))],
    sent: &[(usize, usize)],
) -> Result<Vec<[i64; 6]>> {
    let mut out = Vec::new();
    for &(a, b, (inter, union, run, v)) in rows {
        ensure!(
            sent.binary_search(&(a, b)).is_ok(),
            "core echoed pair ({a},{b}) that was never sent"
        );
        let n = |x: u64| x as i64;
        out.push([a as i64, b as i64, n(inter), n(union), n(run), i64::from(v)]);
    }
    Ok(out)
}

fn name(s: &candidates::SegRow) -> String {
    // .get, not a subscript: `kind` is a stored db column, and a
    // stale or corrupt `.ce/index.db` carrying a kind past this
    // side's vocabulary would abort a report rather than name the
    // row (the deadcode VERDICT_NAMES sibling, same class).
    let kind = crate::docdup::spec::table()
        .kind_names
        .get(s.kind as usize)
        .copied()
        .unwrap_or("kind?");
    format!("{}:{}-{} {}", s.path, s.start_line, s.end_line, kind)
}
