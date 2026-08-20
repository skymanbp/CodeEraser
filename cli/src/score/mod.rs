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
pub mod report;
pub mod wire;

pub use report::{print, report_json};

use crate::graph::deadcode;
use crate::join::churn_unit::UnitMap;
use crate::{churn, dedup, join, scan};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

/// 0.2.0 (review C12): ratchet.fail grew degraded/dedup semantics
/// at proto 2.5/2.6 and the ratchet object gains `failed` (held
/// condition names) + top-level `scoreScale` — plan §7.1 demands
/// the bump.
pub const SCHEMA_ID: &str = "ce.check-report/0.2.0";

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
}

pub struct Outcome {
    pub reply: wire::Reply,
    pub files: usize,
    pub sim_pairs: usize,
    pub members: usize,
    /// Distinct blocks that collapsed into an already-present member
    /// id (same unit pair, second block) — reported, never silent.
    pub collapsed: usize,
    /// Intra-file block pairs the sim table cannot carry (u < v is
    /// the wire contract); their members still enter the set.
    pub skipped_self: usize,
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
    members: Vec<u64>,
    collapsed: usize,
    skipped_self: usize,
}

fn measure(root: &Path, opts: &Opts) -> Result<Measured> {
    let (found, _summary) = dedup::analyze(root, opts.db.clone(), None, None)?;
    let w = deadcode::build_wire(root, opts.db.clone())?;
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
    let (continuous, judged_loc) = size_facts(root)?;
    let req = wire::Request {
        sim: m.sim,
        pos: m.pos,
        churn: churn_t,
        cochange: cochange_t,
        continuous,
        discrete: m.members,
        baseline: if opts.establish {
            serde_json::Value::Null
        } else {
            baseline::read(root)?.unwrap_or(serde_json::Value::Null)
        },
        floor: opts.floor,
        ceilings: knobs::ceiling_rows(&cfg.thresholds, &cfg.score),
        weights: knobs::weight_rows(&cfg.score)?,
        thresholds: knobs::threshold_rows(&cfg.score),
        tolerance: knobs::tolerance_rows(&cfg.score),
        // the dedup pair is `ce dedup --check`'s leg alone (P2) —
        // this road stays byte-identical
        dedup: None,
        judged_loc,
        files: m.files,
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

fn pos_rows(files: &[String], posmap: &HashMap<String, join::Pos>) -> Vec<[i64; 6]> {
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
fn sim_rows(
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
    Ok(size_facts(root)?.0)
}

/// One walk, both size facts (plan v2.6 §B): the continuous rows
/// over the SCAN set (the v2.5 scan-only arm stays size-gated), and
/// the judged-language LOC multiset the core derives the soft line
/// from — two universes on purpose, the boundary being
/// Lang::judged_path (the ONE v2.5 authority).
fn size_facts(root: &Path) -> Result<(Vec<[u64; 3]>, Vec<u64>)> {
    let (files, _findings, _summary) = scan::analyze(root)?;
    let mut rows: Vec<[u64; 3]> = files.iter().flat_map(baseline::continuous_rows).collect();
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
    Ok((rows, locs))
}

/// Per-file churn sums over the per-unit ledger plus the co-change
/// table. survived is 0 = not-claimed (per-entity survival is not
/// tracked; nothing reads it in 3i).
type ChurnTables = (Vec<[i64; 5]>, Vec<[i64; 3]>);

fn churn_rows(root: &Path, days: u32, idx: &HashMap<&str, i64>) -> Result<ChurnTables> {
    let ch = churn::run(root, days)?;
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
        .map(|(u, (rw, ap))| [u, rw, ap, rw + ap, 0])
        .collect();
    let mut coch: BTreeSet<[i64; 3]> = BTreeSet::new();
    for (a, b, n) in &ch.cochange {
        if let (Some(&x), Some(&y)) = (idx.get(a.as_str()), idx.get(b.as_str())) {
            coch.insert([x.min(y), x.max(y), *n as i64]);
        }
    }
    Ok((churn_t, coch.into_iter().collect()))
}
