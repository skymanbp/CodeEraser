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
use anyhow::{Context, Result, ensure};
use std::path::{Path, PathBuf};

/// JSON output schema id; bump on shape change (plan §7.1). This is
/// the ONE schema the CLI report and the S4 GUI tree share.
/// 0.2.0 (M6 S3a): + divergence (per-mille χ² or null), deviations
/// [{dir, kind}], declaredDirs. 0.3.0 (S3b): + deep (whether the S6
/// redundancy rollup rode the wire — axis-6 rows exist only then).
/// 0.4.0 (S3c): + days (the staleness window; null = axis 5 off).
/// 0.5.0 (S4a): + tree [{id, parent, name, depth, subdirs, files,
/// axes}] — the flat parent-linked rows the booklet §5 promised the
/// GUI, per-node axes rolled from the findings.
/// 0.6.0 (v0.6 §C): + split (whether the advisory rode) and, only
/// then, splitCandidates [{path, afterLine, unit, benefitMilli,
/// costMilli}] + sizeExempt [{path, benefitMilli, costMilli}].
pub const SCHEMA_ID: &str = "ce.structure-report/0.6.0";

pub struct Report {
    pub score: i64,
    /// The effective scale from the knob echo (row 8) — the review
    /// C17 lesson applied from day one: no /1000 literal anywhere.
    pub scale: i64,
    pub entropy: Vec<[i64; 2]>,
    pub axes: Vec<[i64; 2]>,
    /// (dir path, axis code), re-labelled from the core's dense ids.
    pub findings: Vec<(String, i64)>,
    pub dirs: usize,
    /// A-layer overlay (S3a): the χ² per-mille against ce.toml's
    /// declared layout, the named deviations, and how many dirs the
    /// layout declared (0 = row-C floor alone; divergence None with
    /// declared > 0 means the deviations rows say where the mass
    /// escaped — null is never a silent shrug).
    pub divergence: Option<i64>,
    pub deviations: Vec<(String, i64)>,
    pub declared: usize,
    /// Whether the S6 rollup rode the wire (--deep): axis-6 rows in
    /// axes/findings exist exactly when true.
    pub deep: bool,
    /// The staleness window in days (--days): axis-5 rows exist
    /// exactly when set.
    pub days: Option<u32>,
    /// The flat parent-linked tree (booklet §5): one row per walked
    /// directory, per-node axis codes rolled from the findings —
    /// the GUI's first-screen data face.
    pub(super) tree: Vec<TreeRow>,
    /// The split-ROI advisory (v0.6 §C): None = not armed; rows are
    /// relabelled with the paths and unit names this side kept.
    pub split: Option<SplitReport>,
}

pub struct SplitReport {
    /// (path, boundary line, unit name, benefitMilli, costMilli).
    pub candidates: Vec<(String, u64, String, i64, i64)>,
    /// (path, bestBenefitMilli, bestCostMilli); 0/0 = no seam.
    pub exempt: Vec<(String, i64, i64)>,
}

pub(super) struct TreeRow {
    pub name: String,
    pub parent: usize,
    pub depth: u32,
    pub subdirs: u32,
    pub files: u32,
    pub axes: Vec<i64>,
}

pub fn run(
    root: &Path,
    db: Option<PathBuf>,
    core: &str,
    (deep, days, split): (bool, Option<u32>, bool),
) -> Result<Report> {
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
    let reply = wire::judge(core, &req)?;
    let names = names_by_id(&t);
    // The boundary contract runs on the REQUEST side (Structure.hs
    // dirRow); nothing checked the ids coming BACK, and both faces
    // subscript a Vec with them — a negative wraps, a large one panics.
    let n = names.len() as i64;
    for &[d, _] in reply.findings.iter().chain(&reply.deviations) {
        ensure!(
            (0..n).contains(&d),
            "structure reply: dir id {d} outside 0..{n}"
        );
    }
    let tree = tree_rows(&t, &names, &reply.findings);
    let findings = relabel(&names, &reply.findings);
    let deviations = relabel(&names, &reply.deviations);
    let scale = reply
        .knobs
        .iter()
        .find(|[c, _]| *c == 8)
        .map(|[_, v]| *v)
        .context("knob echo missing the scale row")?;
    let split_report = seam_facts
        .as_ref()
        .map(|sf| split_relabel(sf, &reply))
        .transpose()?;
    Ok(Report {
        score: reply.score,
        scale,
        entropy: reply.entropy,
        axes: reply.axes,
        findings,
        dirs: t.dirs.len(),
        divergence: reply.divergence,
        deviations,
        declared: req.declared.len(),
        deep,
        days,
        tree,
        split: split_report,
    })
}

/// The committed baseline's frozen soft line, falling back to the
/// warn threshold — the SAME resolution the guard's zone observer
/// uses, so the advisory and the hook agree on where the zone opens.
/// The stored value is bounded the way the CORE bounds it
/// (baseline.softLine must be >= 1): `Some(0)` short-circuited the
/// fallback and made every file with any lines at all "past the soft
/// line" — a baseline the core would refuse still drove the advisory.
pub(crate) fn committed_soft(root: &Path) -> u64 {
    let stored = crate::score::baseline::read(root)
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

/// Dense reply rows → named advisory rows, ids range-checked before
/// any subscript (the findings/deviations lesson applied here from
/// day one).
fn split_relabel(sf: &super::seams::SeamFacts, reply: &wire::Reply) -> Result<SplitReport> {
    let n = sf.files.len() as i64;
    let file = |id: i64| -> Result<usize> {
        ensure!(
            (0..n).contains(&id),
            "split reply: file id {id} outside 0..{n}"
        );
        Ok(id as usize)
    };
    let mut candidates = Vec::new();
    for &[fid, unit, benefit, cost] in &reply.split_candidates {
        let f = file(fid)?;
        let units = &sf.unit_names[f];
        let u = usize::try_from(unit).ok().filter(|u| *u < units.len());
        let (name, end) = match u {
            Some(u) => (units[u].0.clone(), units[u].1),
            None => anyhow::bail!("split reply: unit {unit} outside file {fid}"),
        };
        candidates.push((sf.files[f].0.clone(), end, name, benefit, cost));
    }
    let mut exempt = Vec::new();
    for &[fid, benefit, cost] in &reply.size_exempt {
        let f = file(fid)?;
        exempt.push((sf.files[f].0.clone(), benefit, cost));
    }
    Ok(SplitReport { candidates, exempt })
}

/// Judged languages only (plan v2.5): letting the scan-only arm in
/// would change every axis the TREE feeds — geometry (S0), naming
/// (S1), docs (S4) and both entropy rows all shift with the file
/// population (S2 mixing reads pattern distributions, not file
/// language — the old comment blamed the wrong axis; batch-7 defect
/// sweep).
fn judged_paths(files: &[crate::scan::metrics::FileMetrics]) -> Vec<String> {
    files
        .iter()
        .filter(|f| crate::scan::lang::Lang::judged_path(std::path::Path::new(&f.path)).is_some())
        .map(|f| f.path.clone())
        .collect()
}

/// One row per directory, the findings convolved back onto their
/// nodes — the report keeps the DENSE ids so the GUI can link
/// parents without re-deriving anything.
fn tree_rows(t: &tree::Tree, names: &[String], findings: &[[i64; 2]]) -> Vec<TreeRow> {
    let mut rows: Vec<TreeRow> = t
        .dirs
        .iter()
        .enumerate()
        .map(|(i, d)| TreeRow {
            name: names[i].clone(),
            parent: d.parent,
            depth: d.depth,
            subdirs: d.subdirs,
            files: d.files,
            axes: Vec::new(),
        })
        .collect();
    for &[d, axis] in findings {
        rows[d as usize].axes.push(axis);
    }
    rows
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
    // The seam pricing judges against the SAME numbers the
    // measurement selected seam files by: the committed soft line
    // (codes 12) and the config's hard line (13). Sending nothing let
    // the core price at its built-in 300/750 while seams.rs gated on
    // committed_soft — with this repo's frozen 304 the two disagreed
    // on every file in 301..=304, and on a wide-distribution corpus
    // (S clamped to 500) the ROI inflated ~2.8x.
    // ... and code 14, P_max, whenever ce.toml declares one. Sending
    // 12 and 13 alone was the same defect one knob further along: the
    // advisory priced at the core's built-in 10 while `verdict/1` got
    // the declared value through score::knobs::ceiling_rows code 3,
    // so a repo setting `[score] size_penalty_max` had its two
    // families disagree about the SAME curve — silently, since both
    // answers are internally consistent. Absent stays absent: the
    // core default judges, which is what keeps an undeclaring repo
    // byte-identical.
    let knobs = if seams.is_some() {
        let mut k = vec![
            [12, committed_soft(root)],
            [13, cfg.thresholds.file_lines_fail as u64],
        ];
        if let Some(p) = cfg.score.size_penalty_max {
            k.push([14, u64::from(p)]);
        }
        k
    } else {
        Vec::new()
    };
    Ok(wire::Request {
        nodes: rows::node_rows(t),
        patterns: rows::pattern_rows(t),
        conventions: rows::convention_rows(t),
        file_refs: rows::file_ref_rows(w, t)?,
        declared: rows::declared_rows(&cfg.structure.layout, t)?,
        stale_docs,
        redundancy,
        seams,
        knobs,
    })
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

fn names_by_id(t: &tree::Tree) -> Vec<String> {
    let mut names = vec![String::from("."); t.dirs.len()];
    for (path, &id) in &t.ids {
        if !path.is_empty() {
            names[id] = path.clone();
        }
    }
    names
}

/// Dense [dirId, code] rows re-labelled with the names this side
/// kept — findings and deviations ride the same throat (§5.9.2).
fn relabel(names: &[String], rows: &[[i64; 2]]) -> Vec<(String, i64)> {
    rows.iter()
        .map(|&[d, code]| (names[d as usize].clone(), code))
        .collect()
}

// report faces (report_json + the bilingual console) live in
// report.rs since M8-G3b — re-exported above so callers keep the
// structure::judge::{print, report_json} paths.
