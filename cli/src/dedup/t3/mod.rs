//! `ce clone` T3 judgment (design vol.2 §4, M5-3e): the frozen
//! candidate pass picks the pairs, this driver rebuilds each unit's
//! postorder tree (tree.rs), ships chunks of at most PAIR_CAP pairs
//! over one core link, and maps the raw TED scores back to unit
//! identities. Every drop is a ledger line (over-cap units, forest
//! spans, the pairs they strand) — the tally discipline candidates.rs
//! established, carried to the wire.

pub mod cache;
mod judge;
pub mod tree;
pub mod wire;

use super::candidates::{self, PairRow, TSED_DEN, TSED_NUM, Unit};
use anyhow::{Result, ensure};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// JSON output schema id; bump on shape change (plan §7.1).
/// 0.2.0 = the S5 counts (M5 close); 0.3.0 (plan v2.30 step 5b-9,
/// additive) = `cached`, the pairs the verdict cache answered.
pub const SCHEMA_ID: &str = "ce.clone-report/0.3.0";

/// One unit's judged fate on the way to the wire (merge/1 reads the
/// same fates for its whole-unit members, step 7).
pub(crate) enum Outcome {
    Tree(tree::UnitTree),
    OverCap,
    Forest,
}

/// The family metric block riding each reported pair (report::Pair
/// flattens it, so the JSON row shape is unchanged).
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ted {
    pub ted: i64,
    pub n1: i64,
    pub n2: i64,
}

/// One judged wire row's payload: raw ted, the two sizes, and the
/// core's verdict bit (ADR-008 P1).
type ScoredTed = (i64, i64, i64, bool);

/// Judged rows: (a, b, payload) by built-unit index.
type Scored = Vec<(usize, usize, ScoredTed)>;

pub type Report = crate::report::Report<Ted, Counts>;

#[derive(Serialize, Clone)]
pub struct Counts {
    pub units: usize,
    pub over_cap_units: usize,
    pub forest_units: usize,
    pub survivors: u64,
    pub s5_windowed: u64,
    pub s5_pruned_label: u64,
    pub s5_already: u64,
    pub s5_new: u64,
    pub pairs_dropped_over_cap: u64,
    pub pairs_dropped_forest: u64,
    pub sent: u64,
    pub requests: usize,
    pub prefiltered: u64,
    pub judged: u64,
    /// Sendable pairs the verdict cache answered (cache.rs): `sent`
    /// = `judged` + `prefiltered` + `cached`, every run.
    pub cached: u64,
    pub clones: usize,
}

/// The whole judgment: refresh + identity gate, candidates, trees,
/// chunked clone.requests, verdicts — `ce clone`'s face over its own
/// refreshed index.
pub fn run(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Report> {
    let (idx, _db_path) = super::refreshed_index(root, db)?;
    Ok(judge_index(root, &idx, core)?.report())
}

/// The judged universe: the units, the pairs the core called clones
/// (unit indices, a < b, with the raw metric) and the counts ledger.
/// `ce check` and `ce join` read the pairs off their own snapshot
/// (plan v2.30 step 5b-9); `ce clone` prints the report.
pub struct Judged {
    pub units: Vec<Unit>,
    pub clones: Vec<(usize, usize, Ted)>,
    pub counts: Counts,
}

impl Judged {
    /// The report face: each clone pair under its two unit names.
    pub fn report(&self) -> Report {
        Report {
            hits: self
                .clones
                .iter()
                .map(|&(a, b, m)| crate::report::Pair {
                    a: name(&self.units[a]),
                    b: name(&self.units[b]),
                    m,
                })
                .collect(),
            counts: self.counts.clone(),
        }
    }

    /// The clone pairs as file pairs — the sim table's kind-1 rows
    /// (score::clone_rows), one per clone pair, deduplicated there.
    pub fn file_pairs(&self) -> impl Iterator<Item = (&str, &str)> {
        self.clones
            .iter()
            .map(|&(a, b, _)| (self.units[a].path.as_str(), self.units[b].path.as_str()))
    }
}

/// The judgment over an already-refreshed index: identity gate,
/// candidates, trees, the cache-backed chunked judgment, verdicts.
pub fn judge_index(root: &Path, idx: &super::index::Index, core: &str) -> Result<Judged> {
    let orphans = super::unitcache::identity_orphans(idx)?;
    ensure!(
        orphans == 0,
        "{orphans} unitsig rows missing their symbols identity — nth throat drift"
    );
    let mut cand = candidates::collect(root, idx)?;
    // the product judgment sees the exhaustive S5 extension; the
    // frozen-instrument path calls collect() alone (candidates.rs)
    candidates::extend_exhaustive(&mut cand);
    let units: Vec<&Unit> = cand.units.iter().collect();
    let built = build_trees(root, &units, tree::Extras::With)?;
    let (sendable, dropped_over_cap, dropped_forest) = sendable_pairs(&cand.pairs, &built);
    let (rows, [judged, prefiltered, cached], requests) =
        judge::judge(core, idx.raw(), &built, &sendable)?;
    let clones = reported_clones(&rows)?;
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
        s5_windowed: cand.tally.s5_windowed,
        s5_pruned_label: cand.tally.s5_pruned_label,
        s5_already: cand.tally.s5_already,
        s5_new: cand.tally.s5_new,
        pairs_dropped_over_cap: dropped_over_cap,
        pairs_dropped_forest: dropped_forest,
        sent: sendable.len() as u64,
        requests,
        prefiltered,
        judged,
        cached,
        clones: clones.len(),
    };
    Ok(Judged {
        units: cand.units,
        clones,
        counts,
    })
}

fn name(u: &Unit) -> String {
    format!("{}:{}#{}", u.path, u.key, u.nth)
}

/// The reported set from the CORE's verdict bits (ADR-008 P1), with
/// the per-row drift ensure: the pinned mirror must agree or the run
/// dies loudly — formula drift named, never a silently forked
/// verdict (the frozen 3f instruments score through is_clone, so
/// this check is what keeps them equal to the product by proof).
/// A row the verdict cache replayed (cache.rs) passes here like a
/// fresh one: the mirror sees every bit the report is built from.
fn reported_clones(rows: &[(usize, usize, ScoredTed)]) -> Result<Vec<(usize, usize, Ted)>> {
    for &(_, _, (ted, n1, n2, v)) in rows {
        ensure!(
            v == is_clone(ted, n1, n2),
            "core clone verdict ({v}) disagrees with the pinned mirror at ted {ted} nodes {n1}/{n2} — formula drift (Clone/Cost.hs vs t3/mod.rs)"
        );
    }
    Ok(rows
        .iter()
        .filter(|&&(_, _, (_, _, _, v))| v)
        .map(|&(a, b, (ted, n1, n2, _))| (a, b, Ted { ted, n1, n2 }))
        .collect())
}

/// clone ⇔ (max − ted)·tsedDen ≥ tsedNum·max — since ADR-008 P1 a
/// MIRROR of the core's verdict (CE.Clone.Cost.cloneDecides), not an
/// authority: the reported set is built from the wire's per-row
/// verdict bits, and this binding remains for the frozen 3f
/// precision instruments plus run()'s per-row drift ensure. Same
/// constants as the admissible prunes; the knobs echo pins them.
pub fn is_clone(ted: i64, n1: i64, n2: i64) -> bool {
    let mx = n1.max(n2);
    (mx - ted) * TSED_DEN >= TSED_NUM * mx
}

/// One parse per file, spans in unit order. Over-cap units are never
/// built (their walk is the cost the cap avoids). The node-count
/// ensure per built tree is the same-source counterfactual living in
/// the product path: tree.rs selects by the unit_seq predicate, so a
/// mismatch means the disk drifted from the cache mid-run or the two
/// walks diverged — an error, never a silently wrong judgment. A tree
/// built without extras (merge/1's) holds that count less its comments:
/// never more.
pub(crate) fn build_trees(
    root: &Path,
    units: &[&Unit],
    extras: tree::Extras,
) -> Result<Vec<Outcome>> {
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
        let (text, lang) = super::walked_text(root, path)?;
        let spans: Vec<(usize, usize)> = ids
            .iter()
            .map(|&i| (units[i].start_line as usize, units[i].end_line as usize))
            .collect();
        let trees = tree::file_trees(&text, lang, &spans, extras);
        ensure!(
            trees.len() == spans.len(),
            "{path}: parse failed under cached unitsig rows — disk drifted from the index"
        );
        for (&i, b) in ids.iter().zip(trees) {
            if let tree::Built::Tree(t) = &b {
                ensure!(
                    extras.agrees(t.lab.len() as i64, units[i].nodes),
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

/// Report emission through the ONE shared envelope+console throat.
pub fn print(r: &Report, as_json: bool) {
    crate::report::emit(
        (SCHEMA_ID, "clones"),
        r,
        as_json,
        crate::i18n::t(
            "clone {a} <-> {b}  ted {ted} (nodes {n1}/{n2})",
            "克隆 {a} <-> {b}  ted {ted}（节点 {n1}/{n2}）",
        ),
        crate::i18n::t(
            "{clones} near-miss clone pair(s) over {units} unit(s) — {judged} judged, {cached} replayed from the verdict cache, {prefiltered} provably below threshold",
            "{clones} 对近似克隆 / {units} 个单元 — 判决 {judged}，判决缓存回放 {cached}，可证低于阈值预滤 {prefiltered}",
        ),
    );
}

#[cfg(test)]
#[path = "../../../tests/unit/dedup/t3.rs"]
mod tests;
