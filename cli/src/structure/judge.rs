//! `ce structure` (M6 S2): the tree-scale judgment — walk the tree
//! once through the scan measurement (ONE walk for every surface),
//! assemble the fact tables through structure::rows, send ONE
//! structure.request, and re-label the core's dense verdicts with
//! the names this side kept (§5.9.2). Report-only by ruling — no
//! score floor (v2.22 close-out, O53): the CLI gates nothing here.

// single-path module imports on purpose: the reference ladder
// resolves them to the sibling FILES, so the graph sees the true
// dependencies instead of a parent-hub cycle (headroom sprint,
// 2026-08-24 — this module family was the cycle axis's own finding)
use super::rows;
use super::tree;
use super::wire;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub use super::report::Report;
use crate::document::Answer;

pub fn run(
    root: &Path,
    db: Option<PathBuf>,
    core: &str,
    (deep, days, split): (bool, Option<u32>, bool),
) -> Result<Answer> {
    // measurement only — the structure verdict is this family's own
    // wire call below, never the scan mirror (batch-7 slice 8)
    let (_config, files) = crate::scan::measure(root)?;
    let t = tree::build(&judged_paths(&files));
    // ONE snapshot for every leg (batch 9 P10): blocks + index
    // measured once, the graph wire from that same index — the "ONE
    // walk" the module doc promises, structural now.
    let (found, idx, db_path) = crate::dedup::snapshot(root, db)?;
    let w = crate::graph::deadcode::wire_of(
        root,
        &idx,
        &db_path,
        crate::graph::deadcode::Advisory::No,
    )?;
    drop(idx);
    let seam_facts = if split {
        Some(super::seams::seam_facts(
            root,
            &files,
            committed_soft(root),
            &found,
        )?)
    } else {
        None
    };
    let req = assemble(root, core, &t, (deep, days), &seam_facts, (&w, &found))?;
    let mut link = crate::lockstep::open_family(core, wire::CAP)?;
    let reply = wire::judge_on(&mut link, &req)?;
    let scale = reply
        .knobs
        .iter()
        .find(|[c, _]| *c == 8)
        .map(|[_, v]| *v)
        .context("knob echo missing the scale row")?;
    // the document the core lays out (plan v2.32 step 4): the reply's
    // rows sent back, the directory ids range-checked by its contract
    let parts = super::document::Parts {
        tree: &t,
        reply: &reply,
        scale,
        declared: req.declared.len(),
        deep,
        days,
        seams: seam_facts.as_ref(),
    };
    super::document::assemble(core, Ok(link), &parts)
}

/// The committed baseline's frozen soft line, falling back to the
/// warn threshold — the SAME resolution the guard's zone observer
/// uses, so the advisory and the hook agree on where the zone opens.
/// The stored value is bounded the way the CORE bounds it
/// (baseline.softLine must be >= 1): `Some(0)` short-circuited the
/// fallback and made every file with any lines at all "past the soft
/// line" — a baseline the core would refuse still drove the advisory.
pub(crate) fn committed_soft(root: &Path) -> u64 {
    let stored = crate::score::baseline::document(root)
        .ok()
        .flatten()
        .and_then(|doc| doc["softLine"].as_u64())
        .filter(|s| *s >= 1);
    stored.unwrap_or_else(|| {
        crate::config::Config::load(root)
            .map(|c| c.thresholds.file_lines_warn as u64)
            .unwrap_or(300)
    })
}

/// Judged languages only (plan v2.5): letting the scan-only arm in
/// would change every axis the TREE feeds — geometry (S0), naming
/// (S1), docs (S4) and both entropy rows all shift with the file
/// population (S2 mixing reads the style distributions the core
/// folds from the shape rows, not file language — the old comment
/// blamed the wrong axis; batch-7 defect sweep).
fn judged_paths(files: &[crate::scan::metrics::FileMetrics]) -> Vec<String> {
    files
        .iter()
        .filter(|f| crate::scan::lang::Lang::judged_path(std::path::Path::new(&f.path)).is_some())
        .map(|f| f.path.clone())
        .collect()
}

/// The whole request from one walked tree: the always-on tables,
/// the ce.toml layout, and the two honestly-optional axes (split
/// from run() when the third optional table pushed it past the
/// repo's own function gate). The two axis switches travel as ONE
/// pair — they are one decision ("which optional axes ride"), and
/// the param gate agrees; the snapshot pair (wire, blocks) travels
/// the same way — it is one measurement (batch 9 P10).
fn assemble(
    root: &Path,
    core: &str,
    t: &tree::Tree,
    (deep, days): (bool, Option<u32>),
    seam_facts: &Option<super::seams::SeamFacts>,
    (w, found): (
        &crate::graph::deadcode::GraphWire,
        &crate::dedup::pairs::Blocks,
    ),
) -> Result<wire::Request> {
    let cfg = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
    let stale_docs = match days {
        Some(d) => Some(rows::stale_doc_rows(root, w, t, d)?),
        None => None,
    };
    let redundancy = if deep {
        Some(rows::redundancy_rows(root, core, w, found, t)?)
    } else {
        None
    };
    // ONE shape end to end: the measurement assembled a SeamTables
    // in place; only the ref rows re-sort here (the wire demands
    // strict ascent independent of the walk order).
    let seams = seam_facts.as_ref().map(|sf| wire::SeamTables {
        refs: sorted_refs(&sf.tables.refs),
        ..sf.tables.clone()
    });
    let knobs = seams
        .as_ref()
        .map_or_else(Vec::new, |_| seam_knobs(root, &cfg));
    // ONE join, two tables (O54): fileRefs and the directed crossing
    // table are drawn from the same edge multiset, which is what lets
    // the core read the intra mass off `inside` instead of asking for
    // it twice.
    let (file_refs, dir_edges) = rows::ref_rows(w, t)?;
    Ok(wire::Request {
        nodes: rows::node_rows(t),
        shapes: rows::shape_rows(t),
        conventions: rows::convention_rows(t),
        file_refs,
        declared: rows::declared_rows(&cfg.structure.layout, t)?,
        stale_docs,
        redundancy,
        dir_edges: Some(dir_edges),
        seams,
        knobs,
    })
}

/// The knob rows the split-ROI advisory rides on, and only it (an
/// unarmed advisory sends none — absent stays absent, which is what
/// keeps an undeclaring repo byte-identical). The seam pricing judges
/// against the SAME numbers the measurement selected seam files by:
/// the committed soft line (code 12) and the config's hard line (13).
/// Sending nothing let the core price at its built-in 300/750 while
/// seams.rs gated on committed_soft — with this repo's frozen 304 the
/// two disagreed on every file in 301..=304, and on a
/// wide-distribution corpus (S clamped to 500) the ROI inflated ~2.8x.
/// Code 14, P_max, rides whenever ce.toml declares one: sending 12 and
/// 13 alone was the same defect one knob further along — the advisory
/// priced at the core's built-in 10 while `verdict/1` got the declared
/// value through score::knobs::ceiling_rows code 3, so a repo setting
/// `[score] size_penalty_max` had its two families disagree about the
/// SAME curve, silently, since both answers are internally consistent.
fn seam_knobs(root: &Path, cfg: &crate::config::Config) -> Vec<[u64; 2]> {
    let mut k = vec![
        [12, committed_soft(root)],
        [13, cfg.thresholds.file_lines_fail as u64],
    ];
    if let Some(p) = cfg.score.size_penalty_max {
        k.push([14, u64::from(p)]);
    }
    k
}

/// The wire demands strictly ascending ref rows; the measurement
/// emits them file-then-unit ordered already, but dedup + sort here
/// keeps the contract independent of that walk order.
fn sorted_refs(rows: &[[u64; 3]]) -> Vec<[u64; 3]> {
    let mut out: Vec<[u64; 3]> = rows.to_vec();
    out.sort_unstable();
    out.dedup();
    out
}

// the report face (the document read back + the bilingual console)
// lives in report.rs since M8-G3b; the document request in
// document.rs since plan v2.32 step 4.
