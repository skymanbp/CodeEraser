//! `ce check` / `ce baseline` (M5-3i, ADR-006): assemble the fact
//! tables — file tier from the SAME graph wire deadcode judges, sim
//! pairs from the T1/T2 blocks, graph positions, optional churn,
//! fingerprinted continuous metrics, the discrete clone-member set —
//! send ONE verdict.request with the committed baseline VERBATIM,
//! and relay the core's judgment. Rust computes no policy: score,
//! tolerance, membership and fail all come back on the wire
//! (ADR-008) — including the degraded case, whose reply carries
//! fail=true from the core itself since P1 (a gate that could not
//! judge must never pass, and the core says so).

pub mod baseline;
pub mod knobs;
pub mod model;
pub mod report;
pub mod wire;

pub use model::{Outcome, SCHEMA_ID};
pub use report::{print, report_json};

use crate::graph::deadcode;
use crate::join::churn_unit::UnitMap;
use crate::{churn, dedup, join, scan};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

pub struct Opts {
    pub db: Option<PathBuf>,
    pub core: String,
    /// Churn window; None = the churn/cochange tables stay empty
    /// (blame costs minutes — an honest absence, not a zero claim).
    pub days: Option<u32>,
    /// --fail-under per-mille floor, straight onto the wire.
    pub floor: Option<u32>,
    /// true = judge with a NULL baseline (re-establish: the current
    /// facts become the new floor wholesale) — the CE_ACCEPT_BASELINE
    /// path; the committed file is otherwise read verbatim.
    pub establish: bool,
    /// Establish, but against a FIXED soft line instead of one
    /// derived from this snapshot. Since 2.14.0 a null baseline does
    /// two things at once: it empties the ratchet AND makes the core
    /// derive the size soft line from the same LOC multiset it is
    /// judging — self-reference the size-advisory contract forbids
    /// ("a bloat commit must not move its own goalpost"; the re-anchor
    /// is CE_ACCEPT_BASELINE, by name). One caller needs the first
    /// meaning without the second: `ce trend` measures every
    /// historical commit this way, and a fence recomputed per point
    /// made the whole trajectory incomparable. A baseline carrying
    /// only this soft line and empty tables says exactly that — no
    /// ceilings to bust, no members to add, one fence for every point.
    pub pinned_soft: Option<u64>,
}

/// The measurement half of a check run — everything read off the
/// repo before ce.toml speaks (split from run at the E01 warn line,
/// the SECOND time: ADR-008 P4's knob tables re-grew the body the
/// M5-close split had repaid).
struct Measured {
    files: Vec<String>,
    idx: HashMap<String, i64>,
    sim: Vec<[i64; 5]>,
    pos: Vec<[i64; 6]>,
    /// The export surface under this universe (6.1.0) — RG10's fact.
    symbols: Vec<[i64; 2]>,
    members: Vec<u64>,
    collapsed: usize,
    skipped_self: usize,
}

fn measure(root: &Path, opts: &Opts) -> Result<Measured> {
    // one snapshot (batch 9 P10): blocks and wire come from ONE
    // walk — build_wire's second full measurement is retired
    let (found, snap, db_path) = dedup::snapshot(root, opts.db.clone())?;
    let w = deadcode::wire_of(root, &snap, &db_path)?;
    drop(snap);
    let fnodes = deadcode::file_nodes(&w);
    let pos_req: Vec<i64> = fnodes.iter().map(|&(i, _)| i).collect();
    let posmap = judged_positions(&opts.core, &w, &pos_req)?;
    let files: Vec<String> = fnodes.iter().map(|&(_, p)| p.to_string()).collect();
    let idx: HashMap<&str, i64> = files
        .iter()
        .enumerate()
        .map(|(i, p)| (p.as_str(), i as i64))
        .collect();
    let mut sim = Vec::new();
    let skipped_self = sim_rows(&found.blocks, &idx, &mut sim);
    let (members, collapsed) = member_set(root, &found.blocks);
    Ok(Measured {
        pos: pos_rows(&files, &posmap),
        symbols: crate::graph::symwire::rekeyed(&w, &idx)?,
        idx: idx.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        files,
        sim,
        members,
        collapsed,
        skipped_self,
    })
}

/// Judge the repo against the committed baseline: measure, window
/// churn when asked, then ce.toml speaks its knob tables onto the
/// wire (ADR-008: the first step sent the ceilings; P4 added
/// weights, thresholds and tolerance through score::knobs' declared
/// tables — the reply echoes the effective set and judge() asserts
/// the round trip).
pub fn run(root: &Path, opts: Opts) -> Result<Outcome> {
    let m = measure(root, &opts)?;
    let idx: HashMap<&str, i64> = m.idx.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let (churn_t, cochange_t) = match opts.days {
        Some(days) => churn_rows(root, days, &idx)?,
        None => (Vec::new(), Vec::new()),
    };
    let cfg = crate::config::Config::load(root).map_err(anyhow::Error::msg)?;
    let (continuous, judged_loc, classed) = size_facts(root)?;
    let req = wire::Request {
        sim: m.sim,
        pos: m.pos,
        // the export surface (6.1.0): the candidates' RG10 guard has
        // no other way to learn which flank is a public API, and a
        // `delete` proposed for one is the exact false positive the
        // four-way dead code was split to prevent
        symbols: m.symbols,
        churn: churn_t,
        cochange: cochange_t,
        continuous,
        classed,
        class_knobs: knobs::class_knob_rows(&cfg.rules),
        knobs_digest: cfg.knobs_digest(),
        discrete: m.members,
        baseline: match (opts.establish, opts.pinned_soft) {
            (true, Some(soft)) => serde_json::json!({
                "continuous": [], "discrete": [], "softLine": soft,
            }),
            (true, None) => serde_json::Value::Null,
            (false, _) => baseline::read(root)?.unwrap_or(serde_json::Value::Null),
        },
        floor: opts.floor,
        ceilings: knobs::ceiling_rows(&cfg.thresholds, &cfg.score),
        weights: knobs::weight_rows(&cfg.score)?,
        thresholds: knobs::threshold_rows(&cfg.score),
        tolerance: knobs::tolerance_rows(&cfg.score),
        // the dedup pair is `ce dedup --check`'s leg alone (P2) —
        // this road stays byte-identical
        dedup: None,
        dedup_distinct: Vec::new(),
        dedup_min_distinct: None,
        judged_loc,
        doc_files: doc_file_indices(&m.files),
        files: m.files,
        judged_mask: crate::scan::lang::Lang::judged_mask(),
    };
    let reply = wire::judge(&opts.core, &req)?;
    Ok(Outcome {
        files: req.files.len(),
        sim_pairs: req.sim.len(),
        members: req.discrete.len(),
        collapsed: m.collapsed,
        skipped_self: m.skipped_self,
        reply,
    })
}

pub(crate) fn doc_file_indices(files: &[String]) -> Vec<i64> {
    files
        .iter()
        .enumerate()
        .filter_map(|(i, path)| {
            (crate::scan::lang::Lang::from_path(Path::new(path))
                == Some(crate::scan::lang::Lang::Markdown))
            .then_some(i as i64)
        })
        .collect()
}

/// Graph judgment + the degraded refusal in one leg (split from run
/// at the E01 warn line): a degraded reply judged nothing, and its
/// empty pos table would silently drop every pos row downstream —
/// the sibling of the deadcode --check hole (clearance review; "a
/// gate that could not judge must never pass").
fn judged_positions(
    core: &str,
    w: &deadcode::GraphWire,
    pos_req: &[i64],
) -> Result<HashMap<String, join::Pos>> {
    let reply = deadcode::judge(core, w, pos_req)?;
    anyhow::ensure!(
        reply["degraded"] != serde_json::json!(true),
        "graph judgment degraded ({}) — refusing to score on it",
        reply["reason"].as_str().unwrap_or("?")
    );
    join::pos_map(&reply, w)
}

pub(crate) fn pos_rows(files: &[String], posmap: &HashMap<String, join::Pos>) -> Vec<[i64; 6]> {
    files
        .iter()
        .enumerate()
        .filter_map(|(u, path)| {
            posmap.get(path).map(|[indeg, outdeg, scc, size, reach]| {
                [u as i64, *indeg, *outdeg, *scc, *size, *reach]
            })
        })
        .collect()
}

/// File pairs with at least one verified block, kind 0 (t1t2) at the
/// exact-run ratio — deduplicated, ascending, u < v (self pairs are
/// counted out, the wire cannot carry them).
pub(crate) fn sim_rows(
    blocks: &[dedup::pairs::Block],
    idx: &HashMap<&str, i64>,
    out: &mut Vec<[i64; 5]>,
) -> usize {
    let mut pairs: BTreeSet<(i64, i64)> = BTreeSet::new();
    let mut skipped_self = 0;
    for b in blocks {
        let (Some(&a), Some(&bb)) = (idx.get(b.a_file.as_str()), idx.get(b.b_file.as_str())) else {
            continue;
        };
        if a == bb {
            skipped_self += 1;
            continue;
        }
        pairs.insert((a.min(bb), a.max(bb)));
    }
    out.extend(pairs.into_iter().map(|(u, v)| [u, v, 0, 100, 100]));
    skipped_self
}

/// The discrete clone-member set: every block's sides attributed to
/// their owning units (the join's own UnitMap throat), hashed per
/// §7.2. Returns (ascending set, collapse count).
fn member_set(root: &Path, blocks: &[dedup::pairs::Block]) -> (Vec<u64>, usize) {
    let mut map = UnitMap::new(root);
    let mut set: BTreeSet<u64> = BTreeSet::new();
    let mut collapsed = 0;
    for b in blocks {
        let a = map.id_of(&b.a_file, b.a_start, b.a_end);
        let z = map.id_of(&b.b_file, b.b_start, b.b_end);
        let side = |u: &crate::join::churn_unit::UnitId| (u.path.clone(), u.key.clone(), u.nth);
        if !set.insert(baseline::member_id("clone", &side(&a), &side(&z))) {
            collapsed += 1;
        }
    }
    (set.into_iter().collect(), collapsed)
}

/// Continuous fact rows for the whole tree: every scanned file's
/// size and function complexity. The 3j-era side-walk for `.hs`
/// sizes is gone — the scanner speaks Haskell since 3k, and the
/// collision ensure below is what would have caught the two paths
/// double-emitting a file. pub: the 3j gate test asserts per-file
/// coverage through this same throat.
pub fn continuous_rows(root: &Path) -> Result<Vec<[u64; 3]>> {
    Ok(size_facts(root)?
        .0
        .into_iter()
        .map(|[u, c, v, _]| [u, c, v])
        .collect())
}

/// One walk, both size facts (plan v2.6 §B): the continuous rows
/// over the SCAN set (the v2.5 scan-only arm stays size-gated), and
/// the judged-language LOC multiset the core derives the soft line
/// from — two universes on purpose, the boundary being
/// Lang::judged_path (the ONE v2.5 authority). Measurement only —
/// no scan verdict is consumed here (batch-7 slice 8). Each row's
/// fourth column is the file's rulepack class (3.1.0: first declared
/// match, 0 = none), assigned HERE where the path is still known;
/// the flag says whether any class was declared at all.
fn size_facts(root: &Path) -> Result<(Vec<[u64; 4]>, Vec<u64>, bool)> {
    let (config, files) = scan::measure(root)?;
    let classes =
        scan::classes::Classes::compile(root, &config.rules).map_err(anyhow::Error::msg)?;
    let mut rows: Vec<[u64; 4]> = files
        .iter()
        .flat_map(|f| {
            let class = classes.class_of(&f.path);
            baseline::continuous_rows(f)
                .into_iter()
                .map(move |[u, c, v]| [u, c, v, class])
        })
        .collect();
    rows.sort_unstable();
    // a fingerprint collision would silently merge two entities —
    // refuse loudly instead (never observed; FNV64 over short paths)
    anyhow::ensure!(
        rows.windows(2).all(|w| w[0][..2] != w[1][..2]),
        "continuous entity fingerprint collision"
    );
    let mut locs: Vec<u64> = files
        .iter()
        .filter(|f| crate::scan::lang::Lang::judged_path(Path::new(&f.path)).is_some())
        .map(|f| f.total_lines as u64)
        .collect();
    locs.sort_unstable(); // the wire demands non-descending
    Ok((rows, locs, classes.declared()))
}

/// Per-file churn sums over the per-unit ledger plus the co-change
/// table. Three columns since 3.0.0 (I round D3): the constant added
/// and the never-measured survived rode unread by the core, and left.
pub(crate) type ChurnTables = (Vec<[i64; 3]>, Vec<[i64; 3]>);

fn churn_rows(root: &Path, days: u32, idx: &HashMap<&str, i64>) -> Result<ChurnTables> {
    let ch = churn::run(root, days)?;
    Ok(churn_tables(&ch, idx))
}

/// The churn wire tables from a report already in hand — the join
/// face measures churn once for display AND judgment (2.33.0, H4;
/// the P10 one-measurement stance).
pub(crate) fn churn_tables(ch: &churn::Report, idx: &HashMap<&str, i64>) -> ChurnTables {
    let mut per_file: BTreeMap<i64, (i64, i64)> = BTreeMap::new();
    for u in &ch.units {
        if let Some(&i) = idx.get(u.path.as_str()) {
            let e = per_file.entry(i).or_default();
            e.0 += u.rewrote as i64;
            e.1 += u.appended as i64;
        }
    }
    let churn_t = per_file
        .into_iter()
        .map(|(u, (rw, ap))| [u, rw, ap])
        .collect();
    let mut coch: BTreeSet<[i64; 3]> = BTreeSet::new();
    for (a, b, n) in &ch.cochange {
        if let (Some(&x), Some(&y)) = (idx.get(a.as_str()), idx.get(b.as_str())) {
            coch.insert([x.min(y), x.max(y), *n as i64]);
        }
    }
    (churn_t, coch.into_iter().collect())
}
