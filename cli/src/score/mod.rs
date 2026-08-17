//! `ce check` / `ce baseline` (M5-3i, ADR-006): assemble the fact
//! tables — file tier from the SAME graph wire deadcode judges, sim
//! pairs from the T1/T2 blocks, graph positions, optional churn,
//! fingerprinted continuous metrics, the discrete clone-member set —
//! send ONE verdict.request with the committed baseline VERBATIM,
//! and relay the core's judgment. Rust computes no policy: score,
//! tolerance, membership and fail all come back on the wire
//! (ADR-008). A degraded reply FAILS the check — a gate that could
//! not judge must never pass.

pub mod baseline;
pub mod wire;

use crate::graph::deadcode;
use crate::join::churn_unit::UnitMap;
use crate::{churn, dedup, join, scan};
use anyhow::Result;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

pub const SCHEMA_ID: &str = "ce.check-report/0.1.0";

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

pub fn run(root: &Path, opts: Opts) -> Result<Outcome> {
    let (found, _summary) = dedup::analyze(root, opts.db.clone(), None, None)?;
    let w = deadcode::build_wire(root, opts.db)?;
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
    let (churn_t, cochange_t) = match opts.days {
        Some(days) => churn_rows(root, days, &idx)?,
        None => (Vec::new(), Vec::new()),
    };
    // ce.toml speaks its size/coc ceilings onto the wire (ADR-008
    // first step); the reply echoes the effective pair and judge()
    // asserts the round trip — the 300/15 mirror is retired
    let t = crate::config::Config::load(root)
        .map_err(anyhow::Error::msg)?
        .thresholds;
    let req = wire::Request {
        sim,
        pos: pos_rows(&files, &posmap),
        churn: churn_t,
        cochange: cochange_t,
        continuous: continuous_rows(root)?,
        discrete: members,
        baseline: if opts.establish {
            serde_json::Value::Null
        } else {
            baseline::read(root)?.unwrap_or(serde_json::Value::Null)
        },
        floor: opts.floor,
        ceilings: vec![[0, t.file_lines_warn as i64], [1, t.cognitive_warn as i64]],
        files,
    };
    let reply = wire::judge(&opts.core, &req)?;
    Ok(Outcome {
        files: req.files.len(),
        sim_pairs: req.sim.len(),
        members: req.discrete.len(),
        collapsed,
        skipped_self,
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
    let (files, _findings, _summary) = scan::analyze(root)?;
    let mut rows: Vec<[u64; 3]> = files.iter().flat_map(baseline::continuous_rows).collect();
    rows.sort_unstable();
    // a fingerprint collision would silently merge two entities —
    // refuse loudly instead (never observed; FNV64 over short paths)
    anyhow::ensure!(
        rows.windows(2).all(|w| w[0][..2] != w[1][..2]),
        "continuous entity fingerprint collision"
    );
    Ok(rows)
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

pub fn report_json(o: &Outcome) -> serde_json::Value {
    let r = &o.reply;
    json!({
        "schema": SCHEMA_ID,
        "score": r.score,
        "axes": r.axes,
        "candidates": r.candidates,
        "ratchet": {
            "added": r.added, "removed": r.removed, "over": r.over,
            "toleranceDrawn": r.tolerance_drawn, "fail": r.fail,
        },
        "counts": {
            "files": o.files, "simPairs": o.sim_pairs, "members": o.members,
            "collapsed": o.collapsed, "skippedSelf": o.skipped_self,
        },
        "degraded": r.degraded,
    })
}

pub fn print(o: &Outcome, as_json: bool) {
    if as_json {
        println!("{}", report_json(o));
        return;
    }
    let r = &o.reply;
    let axes: Vec<String> = r.axes.iter().map(|[c, p]| format!("{c}:{p}")).collect();
    println!(
        "check score {}/1000 | axes {} | {} candidates",
        r.score,
        axes.join(" "),
        r.candidates.len()
    );
    println!(
        "ratchet: {} added, {} removed, {} over, {} tolerance drawn -> {}",
        r.added.len(),
        r.removed.len(),
        r.over.len(),
        r.tolerance_drawn.len(),
        if r.fail { "FAIL" } else { "pass" }
    );
    if o.collapsed > 0 || o.skipped_self > 0 {
        println!(
            "note: {} blocks collapsed into existing members, {} intra-file pairs off the sim table",
            o.collapsed, o.skipped_self
        );
    }
    if let Some(reason) = &r.degraded {
        println!("check degraded: {reason} -> FAIL (a gate that cannot judge must not pass)");
    }
}
