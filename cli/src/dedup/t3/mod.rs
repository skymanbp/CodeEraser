//! `ce clone` T3 judgment (design vol.2 §4, M5-3e): the frozen
//! candidate pass picks the pairs, this driver rebuilds each unit's
//! postorder tree (tree.rs), ships chunks of at most PAIR_CAP pairs
//! over one core link, and maps the raw TED scores back to unit
//! identities. Every drop is a ledger line (over-cap units, forest
//! spans, the pairs they strand) — the tally discipline candidates.rs
//! established, carried to the wire.

pub mod tree;
pub mod wire;

use super::candidates::{self, PairRow, TSED_DEN, TSED_NUM, Unit};
use crate::corelink::Link;
use crate::scan::lang::Lang;
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// JSON output schema id; bump on shape change (plan §7.1).
pub const SCHEMA_ID: &str = "ce.clone-report/0.1.0";

/// One unit's judged fate on the way to the wire.
enum Outcome {
    Tree(tree::UnitTree),
    OverCap,
    Forest,
}

#[derive(Serialize)]
pub struct Hit {
    pub a: String,
    pub b: String,
    pub ted: i64,
    pub n1: i64,
    pub n2: i64,
}

#[derive(Serialize)]
pub struct Counts {
    pub units: usize,
    pub over_cap_units: usize,
    pub forest_units: usize,
    pub survivors: u64,
    pub pairs_dropped_over_cap: u64,
    pub pairs_dropped_forest: u64,
    pub sent: u64,
    pub requests: usize,
    pub prefiltered: u64,
    pub judged: u64,
    pub clones: usize,
}

pub struct Report {
    pub clones: Vec<Hit>,
    pub counts: Counts,
}

/// The whole judgment: refresh + identity gate, candidates, trees,
/// chunked clone.requests, verdicts.
pub fn run(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Report> {
    let (mut idx, _db_path) = super::refreshed_index(root, db)?;
    let orphans = super::unitcache::identity_orphans(&idx)?;
    ensure!(
        orphans == 0,
        "{orphans} unitsig rows missing their symbols identity — nth throat drift"
    );
    let cand = candidates::collect(root, &mut idx)?;
    let built = build_trees(root, &cand.units)?;
    let (sendable, dropped_over_cap, dropped_forest) = sendable_pairs(&cand.pairs, &built);
    let out = judge(core, &built, &sendable)?;
    let clones: Vec<Hit> = out
        .rows
        .iter()
        .filter(|&&(_, _, ted, n1, n2)| is_clone(ted, n1, n2))
        .map(|&(a, b, ted, n1, n2)| Hit {
            a: name(&cand.units[a]),
            b: name(&cand.units[b]),
            ted,
            n1,
            n2,
        })
        .collect();
    let (over_cap_units, forest_units) = built.iter().fold((0, 0), |(oc, fo), b| match b {
        Outcome::OverCap => (oc + 1, fo),
        Outcome::Forest => (oc, fo + 1),
        Outcome::Tree(_) => (oc, fo),
    });
    let counts = Counts {
        units: cand.units.len(),
        over_cap_units,
        forest_units,
        survivors: cand.tally.survivors,
        pairs_dropped_over_cap: dropped_over_cap,
        pairs_dropped_forest: dropped_forest,
        sent: sendable.len() as u64,
        requests: out.requests,
        prefiltered: out.prefiltered,
        judged: out.judged,
        clones: clones.len(),
    };
    Ok(Report { clones, counts })
}

fn name(u: &Unit) -> String {
    format!("{}:{}#{}", u.path, u.key, u.nth)
}

/// clone ⇔ (max − ted)·tsedDen ≥ tsedNum·max — integer cross-
/// multiplication with the SAME constants the admissible prunes used;
/// the wire carries raw ted, so the verdict recomputes from one run.
fn is_clone(ted: i64, n1: i64, n2: i64) -> bool {
    let mx = n1.max(n2);
    (mx - ted) * TSED_DEN >= TSED_NUM * mx
}

/// One parse per file, spans in unit order. Over-cap units are never
/// built (their walk is the cost the cap avoids). The node-count
/// ensure per built tree is the same-source counterfactual living in
/// the product path: tree.rs selects by the unit_seq predicate, so a
/// mismatch means the disk drifted from the cache mid-run or the two
/// walks diverged — an error, never a silently wrong judgment.
fn build_trees(root: &Path, units: &[Unit]) -> Result<Vec<Outcome>> {
    let mut out: Vec<Option<Outcome>> = units.iter().map(|_| None).collect();
    let mut by_file: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, u) in units.iter().enumerate() {
        if u.nodes > wire::UNIT_NODE_CAP {
            out[i] = Some(Outcome::OverCap);
        } else {
            by_file.entry(&u.path).or_default().push(i);
        }
    }
    for (path, ids) in by_file {
        // the walkidx read + index.rs text conversion, verbatim — a
        // different decode here would judge text the cache never saw
        let bytes = std::fs::read(root.join(path)).with_context(|| path.to_string())?;
        let text = String::from_utf8_lossy(&bytes);
        let lang = Lang::from_path(Path::new(path)).with_context(|| format!("{path}: no lang"))?;
        let spans: Vec<(usize, usize)> = ids
            .iter()
            .map(|&i| (units[i].start_line as usize, units[i].end_line as usize))
            .collect();
        let trees = tree::file_trees(&text, lang, &spans);
        ensure!(
            trees.len() == spans.len(),
            "{path}: parse failed under cached unitsig rows — disk drifted from the index"
        );
        for (&i, b) in ids.iter().zip(trees) {
            if let tree::Built::Tree(t) = &b {
                ensure!(
                    t.lab.len() as i64 == units[i].nodes,
                    "{path} {}#{}: tree walk found {} nodes, unitsig cached {} — predicate drift",
                    units[i].key,
                    units[i].nth,
                    t.lab.len(),
                    units[i].nodes
                );
            }
            out[i] = Some(match b {
                tree::Built::Tree(t) => Outcome::Tree(t),
                tree::Built::Forest(_) => Outcome::Forest,
            });
        }
    }
    Ok(out.into_iter().map(|o| o.expect("classified")).collect())
}

/// Pairs whose BOTH endpoints have wire trees; the rest land in the
/// two drop ledgers (an over-cap endpoint claims the pair first).
fn sendable_pairs<'p>(pairs: &'p [PairRow], built: &[Outcome]) -> (Vec<&'p PairRow>, u64, u64) {
    let (mut over_cap, mut forest) = (0u64, 0u64);
    let sendable = pairs
        .iter()
        .filter(|p| match (&built[p.a], &built[p.b]) {
            (Outcome::Tree(_), Outcome::Tree(_)) => true,
            (Outcome::OverCap, _) | (_, Outcome::OverCap) => {
                over_cap += 1;
                false
            }
            _ => {
                forest += 1;
                false
            }
        })
        .collect();
    (sendable, over_cap, forest)
}

struct JudgeOut {
    rows: Vec<(usize, usize, i64, i64, i64)>,
    judged: u64,
    prefiltered: u64,
    requests: usize,
}

/// Chunked lockstep judging over ONE core link. Request-local tree
/// indices are the chunk's unit ids by sorted rank — the monotone
/// map keeps the wire's strictly-ascending pair rows for free.
fn judge(core: &str, built: &[Outcome], sendable: &[&PairRow]) -> Result<JudgeOut> {
    let (mut link, _hello) = Link::open(core).map_err(anyhow::Error::msg)?;
    ensure!(
        link.has(wire::CAP),
        "ce-core offers no {} capability — upgrade the core",
        wire::CAP
    );
    let mut out = JudgeOut {
        rows: Vec::new(),
        judged: 0,
        prefiltered: 0,
        requests: 0,
    };
    for chunk in sendable.chunks(wire::PAIR_CAP) {
        let order: Vec<usize> = chunk
            .iter()
            .flat_map(|p| [p.a, p.b])
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let rank: BTreeMap<usize, usize> = order.iter().enumerate().map(|(r, &g)| (g, r)).collect();
        let trees: Vec<&tree::UnitTree> = order
            .iter()
            .map(|&g| match &built[g] {
                Outcome::Tree(t) => t,
                _ => unreachable!("sendable pairs reference built trees only"),
            })
            .collect();
        let local: Vec<[usize; 2]> = chunk.iter().map(|p| [rank[&p.a], rank[&p.b]]).collect();
        let reply = link
            .request("clone", wire::request_body(&trees, &local))
            .map_err(anyhow::Error::msg)?;
        let scores = wire::parse_result(&reply)?;
        out.judged += scores.judged;
        out.prefiltered += scores.prefiltered;
        out.requests += 1;
        for (i, j, ted, n1, n2) in scores.rows {
            out.rows.push((order[i], order[j], ted, n1, n2));
        }
    }
    out.rows.sort_unstable();
    Ok(out)
}

/// Report emission (churn precedent: printing lives with the report,
/// main_cmds stays a router under its 300-line gate).
pub fn print(r: &Report, as_json: bool) {
    if as_json {
        let doc = json!({"schema": SCHEMA_ID, "clones": r.clones, "counts": r.counts});
        println!("{doc}");
        return;
    }
    for h in &r.clones {
        println!(
            "clone {} <-> {}  ted {} (nodes {}/{})",
            h.a, h.b, h.ted, h.n1, h.n2
        );
    }
    let c = &r.counts;
    println!(
        "t3: {} units ({} over cap, {} forest), {} candidate pairs, {} sent in {} requests \
         ({}+{} dropped), {} prefiltered, {} judged — {} clones",
        c.units,
        c.over_cap_units,
        c.forest_units,
        c.survivors,
        c.sent,
        c.requests,
        c.pairs_dropped_over_cap,
        c.pairs_dropped_forest,
        c.prefiltered,
        c.judged,
        c.clones
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The verdict boundary in BOTH directions at max = 100: ted 15
    /// leaves similarity exactly 85/100 (clone), ted 16 falls below.
    #[test]
    fn verdict_sits_exactly_on_the_threshold() {
        assert!(is_clone(15, 100, 90));
        assert!(!is_clone(16, 100, 90));
        assert!(is_clone(0, 3, 3));
    }
}
