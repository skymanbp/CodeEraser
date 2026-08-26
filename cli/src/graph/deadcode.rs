//! `ce deadcode` (M5-2h): refresh the index (the ladder judges every
//! site into the DB on the way), build the graph.request from the
//! cached edges, let the Haskell core judge liveness, and name the
//! verdicts back. An EMPTY graph is an explicit error, never a
//! silent all-dead/all-alive report; a degraded core reply
//! (graph_too_large) lands in the observe feed so doctor/health
//! count it (A9f).
//!
//! Entry standing is measured as ROLE FACTS since 2.28.0 (batch-7
//! slice 3): named mains, executable dirs, test conventions, ce.toml
//! [graph] entry_globs, doc entries, allow claims and declared build
//! targets — the role→entry-bit table is the core's
//! (CE.Graph.Cost.roleBits), this side only measures. The
//! design's "no entry rule = every doc trivially dies" stance is
//! deliberate: an unlinked doc IS reported. Asset edges never count
//! as references (design §4 Markdown row); a package node gets
//! SYNTHETIC containment arcs to every file under it — reaching a
//! package reaches what it holds (the self-repo disposition run
//! found doc→directory edges stranded from the members the walk had
//! already proven alive); section and package verdicts are REPORTED,
//! never called dead (decision 4 / RG9 — aggregates are not code
//! entities). unreferenced_public stays its own class end to end
//! (RG10); the unresolved-site count travels with the report — the
//! reader sees what the graph refuses to know (decision 5:
//! symbol-level indegree stays out while call edges are off).

mod flags;
mod report;
mod targets;

pub use report::print;

use super::load::{GraphEdge, graph_rows};
use super::nodes::{self, Node};
use crate::config::Config;
use crate::dedup;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const VERDICT_NAMES: [&str; 4] = [
    "unref_private",
    "unref_public",
    "unreach_private",
    "unreach_public",
];

#[derive(Debug)]
pub struct Report {
    /// File-tier dead verdicts with their trust column.
    pub dead: Vec<DeadRow>,
    /// Section/package verdicts: reported, never called dead.
    pub reported: Vec<(String, &'static str)>,
    pub nodes: usize,
    /// File-tier nodes alone — `nodes` counts every granularity, so
    /// the share of FILES dead (the plugin-shape signal the console
    /// hint reads) needs its own denominator.
    pub files: usize,
    pub kept: u64,
    pub unresolved_sites: i64,
    pub degraded: Option<String>,
    /// The core's gate bit (2.18.0): any file-tier dead verdict, or
    /// a degraded run, fails — the exit is a relay, not a policy.
    pub fail: bool,
}

pub fn run(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Report> {
    let (idx, db_path) = dedup::refreshed_index(root, db)?;
    let w = wire_of(root, &idx, &db_path)?;
    judge_report(root, core, &w)
}

/// The judgment half-door (batch 9 P10): judge + consume + the
/// degraded observe over a wire already in hand — boundaries
/// holding the one snapshot call this, never a second measurement.
pub fn judge_report(root: &Path, core: &str, w: &GraphWire) -> Result<Report> {
    Ok(judged(root, core, w, &[])?.0)
}

/// The same half-door, answering position rows too — the canvas
/// face (batch 9 P18) needs the verdicts AND the pos table from the
/// ONE judgment; returning the raw reply beside the Report keeps
/// consume private and the observe in one owner.
pub fn judged(root: &Path, core: &str, w: &GraphWire, pos: &[i64]) -> Result<(Report, Value)> {
    let reply = judge(core, w, pos)?;
    let report = consume(&reply, &w.nodes, w.unresolved_sites)?;
    if let Some(reason) = &report.degraded {
        observe(root, reason);
    }
    Ok((report, reply))
}

// The report's JSON face lives in report.rs (deadcode_json) with the
// other shared serializations — lifted out of the binary at M7-P2.

/// Everything one graph.request carries, built once — the request
/// throat deadcode and the M5-3 join share: same node identity, same
/// node rows, same edge wire, so the liveness verdict and the join's
/// position rows stand on one graph by construction (F19), and a
/// second copy of this assembly is exactly what nodes.rs forbids.
pub struct GraphWire {
    pub nodes: Vec<Node>,
    pub rows: Vec<Value>,
    pub edges: BTreeSet<[i64; 4]>,
    pub unresolved_sites: i64,
    /// The per-language site ledger [[lang, unresolved, total]],
    /// langs ascending (2.32.0, H3) — the core judges each dead
    /// row's confidence from it.
    pub unres: Vec<[i64; 3]>,
    /// The export surface [[nodeIdx, visibility]] (4.1.0): which
    /// files declare something and how visibly. The core reads the
    /// public bit off it (graph/symwire.rs).
    pub symbols: BTreeSet<[i64; 2]>,
}

/// One file-tier dead verdict with its trust column (2.32.0, H3):
/// conf is the core's per-row confidence — 0 unvouched (the file's
/// language still carries unresolved sites), 1 vacuous (no site of
/// that language exists), 2 vouched. None only on a legacy reply
/// whose request carried no ledger.
#[derive(Debug)]
pub struct DeadRow {
    pub path: String,
    pub verdict: &'static str,
    pub why: String,
    pub conf: Option<i64>,
}

/// The file-tier slice of the wire's dense assignment: (index, path)
/// in node order — the join and the score assemble their pos
/// requests and tier universes through this ONE selector (the
/// ratchet caught the second copy growing).
pub fn file_nodes(w: &GraphWire) -> Vec<(i64, &str)> {
    w.nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.kind == super::wire::GRAN_FILE)
        .map(|(i, n)| (i as i64, n.path.as_str()))
        .collect()
}

/// The measurement half-door (batch 9 P10): the graph request from
/// an index the command boundary already refreshed and opened.
pub fn wire_of(root: &Path, idx: &dedup::index::Index, db_path: &Path) -> Result<GraphWire> {
    let (files, edges, unresolved_sites, sites) = graph_rows(idx)?;
    if files.is_empty() {
        bail!(
            "empty index at {} — nothing was walked; wrong root?",
            db_path.display()
        );
    }
    let config = Config::load(root).map_err(anyhow::Error::msg)?;
    // identity assignment + containment live in the nodes.rs throat
    // (F19) — the M5-3 join consumes the SAME functions, so both
    // verdicts stand on one id space by construction
    let nodes = nodes::nodes_of(&files, &edges);
    let ids = nodes::ids(&nodes);
    let file_set: BTreeSet<String> = files.iter().cloned().collect();
    let declared = targets::Declared::gather(root, &file_set);
    let rows: Vec<Value> = nodes
        .iter()
        .map(|n| node_row(root, n, &config, &declared))
        .collect();
    let mut wire = edge_wire(&edges, &ids)?;
    nodes::contain(&nodes, &ids, &mut wire);
    let symbols = super::symwire::export_surface(idx, &ids)?;
    Ok(GraphWire {
        nodes,
        rows,
        edges: wire,
        unresolved_sites,
        unres: lang_ledger(&sites),
        symbols,
    })
}

/// Per-path site counts folded to the wire's per-language ledger —
/// BTreeMap keys make the langs ascending, the shape the core's
/// duplicate-free validation demands.
fn lang_ledger(sites: &[(String, i64, i64)]) -> Vec<[i64; 3]> {
    let mut by_lang: BTreeMap<i64, [i64; 2]> = BTreeMap::new();
    for (path, total, unres) in sites {
        if let Some(l) = crate::scan::lang::Lang::from_path(Path::new(path)) {
            let e = by_lang.entry(l as i64).or_insert([0, 0]);
            e[0] += unres;
            e[1] += total;
        }
    }
    by_lang.iter().map(|(&l, &[u, t])| [l, u, t]).collect()
}

/// Cached edges → wire rows — EVERY kind travels since 2.20.0
/// (batch-7 slice 13): the asset exclusion is the core's liveness
/// rule now (CE.Graph.Cost.assetKind), applied where the cut table
/// and the ablation battery can see it; pre-dropping the rows here
/// hid the rule from both. Both endpoints resolve through the dense
/// id map with a NAMED refusal: `ids[..]` on a DB-sourced path
/// panicked on index skew, and a skewed cache row is an error to
/// report, never a crash (the deadcode/lockstep wire-index class).
pub fn edge_wire(
    edges: &[GraphEdge],
    ids: &BTreeMap<(&str, &str), usize>,
) -> Result<BTreeSet<[i64; 4]>> {
    edges
        .iter()
        .map(|e| {
            let of = |path: &str, unit: &str| {
                ids.get(&(path, unit))
                    .map(|&i| i as i64)
                    .with_context(|| format!("edge endpoint {path} not a node — index skew"))
            };
            Ok([
                of(&e.src, "")?,
                of(&e.dst_path, &e.dst_unit)?,
                e.kind,
                e.rung,
            ])
        })
        .collect()
}

/// [lang, kind, roles] — only file nodes carry entry facts. The
/// roles column is the 2.28.0 authority (the core derives the entry
/// bits through its role table, where an ablation can perturb them);
/// the pre-2.28 legacy flags column that used to sit between kind
/// and roles retired at 5.0.0, computed and sent for seven minors
/// after the last core stopped reading it.
fn node_row(root: &Path, n: &Node, config: &Config, declared: &targets::Declared) -> Value {
    // unknown extension = the sentinel code, NOT Python's 0 (RM15:
    // the two were indistinguishable on the wire before 3k)
    let lang = crate::scan::lang::Lang::from_path(Path::new(&n.path))
        .map(|l| l as i64)
        .unwrap_or(crate::scan::lang::Lang::LangUnknown as i64);
    let roles = if n.kind == super::wire::GRAN_FILE {
        flags::roles_of(root, &n.path, config, declared)
    } else {
        0
    };
    json!([lang, n.kind, roles])
}

/// One graph.request over the open core link; a missing capability
/// or a non-result reply is an error, never an empty report. `pos`
/// asks for position rows (the join's leg; deadcode sends none), and
/// a non-degraded reply MUST answer every requested index — a short
/// pos table would silently starve the join, so it refuses here.
pub fn judge(core: &str, w: &GraphWire, pos: &[i64]) -> Result<Value> {
    let mut link = crate::lockstep::open_family(core, "graph/1")?;
    let body = json!({
        "nodes": w.rows,
        "edges": w.edges.iter().collect::<Vec<_>>(),
        "pos": pos,
        "unres": w.unres,
        "symbols": w.symbols.iter().collect::<Vec<_>>(),
    });
    let reply = link.request("graph", body).map_err(anyhow::Error::msg)?;
    let rows = reply["pos"].as_array().map(Vec::len).unwrap_or(0);
    if reply["degraded"] != json!(true) && rows != pos.len() {
        bail!("graph.result answered {rows} of {} pos rows", pos.len());
    }
    Ok(reply)
}

/// Console tag for the trust column — rendering only, the codes
/// are the core's (CE.Graph.Cost.confidence).
pub fn conf_word(conf: Option<i64>) -> &'static str {
    use crate::i18n::t;
    match conf {
        Some(0) => t(
            " [unvouched: unresolved sites in this language]",
            "〔未担保：该语言尚有未解析点位〕",
        ),
        Some(1) => t(" [vacuous]", "〔空担保〕"),
        Some(2) => t(" [vouched]", "〔已担保〕"),
        _ => "",
    }
}

/// One core verdict row resolved to its node and name — shared by
/// the dead and reported loops. A verdict past the four this side
/// knows about is a wire-version skew, not a panic.
fn named(nodes: &[Node], idx: usize, verdict: usize) -> Result<(&Node, &'static str)> {
    let node = nodes.get(idx).context("index out of range")?;
    let name = *VERDICT_NAMES
        .get(verdict.checked_sub(1).context("verdict 0")?)
        .context("verdict out of range")?;
    Ok((node, name))
}

fn consume(reply: &Value, nodes: &[Node], unresolved_sites: i64) -> Result<Report> {
    let mut report = Report {
        dead: Vec::new(),
        reported: Vec::new(),
        nodes: nodes.len(),
        files: nodes
            .iter()
            .filter(|n| n.kind == super::wire::GRAN_FILE)
            .count(),
        kept: reply["counts"]["kept"].as_u64().unwrap_or(0),
        unresolved_sites,
        // The wire's degraded BIT is authoritative (the C9 read-the-
        // real-boolean discipline, contracts/VERSIONING.md); reason
        // is its text, not its signal.
        degraded: (reply["degraded"].as_bool() == Some(true))
            .then(|| reply["reason"].as_str().unwrap_or("degraded").to_string()),
        fail: false,
    };
    // two-column rows on the legacy road, three when the ledger
    // rode (2.32.0) — arity is the road, not noise
    let dead: Vec<Vec<i64>> = serde_json::from_value(reply["dead"].clone()).context("dead rows")?;
    for row in dead {
        let (idx, verdict, conf) = match row[..] {
            [i, v] => (i as usize, v as usize, None),
            [i, v, c] => (i as usize, v as usize, Some(c)),
            _ => anyhow::bail!("dead row of arity {} — wire skew", row.len()),
        };
        let (node, name) = named(nodes, idx, verdict)?;
        // The RG9 split is the CORE's since 2.18.0; here it survives
        // as a boundary contract, because erase's class-0 licence is
        // minted from this list — an aggregate in the failing table
        // is wire skew and must refuse, never license a directory.
        anyhow::ensure!(
            node.kind == super::wire::GRAN_FILE,
            "core dead row {idx} is not file-granularity — wire skew"
        );
        let why = if verdict <= 2 {
            "no kept in-edge and no entry flag"
        } else {
            "referenced only from dead code; no entry flag"
        };
        report.dead.push(DeadRow {
            path: node.path.clone(),
            verdict: name,
            why: why.to_string(),
            conf,
        });
    }
    // pre-2.18 core: no reported table — absent parses as empty
    let reported: Vec<[usize; 2]> =
        serde_json::from_value(reply["reported"].clone()).unwrap_or_default();
    for [idx, verdict] in reported {
        let (node, name) = named(nodes, idx, verdict)?;
        let label = if node.unit.is_empty() {
            node.path.clone()
        } else {
            format!("{}#{}", node.path, node.unit)
        };
        report.reported.push((label, name));
    }
    // pre-2.18 core: no fail bit on the wire — the client's own
    // conjunction stands in, byte-identical to the old exit policy
    report.fail = reply["fail"]
        .as_bool()
        .unwrap_or(report.degraded.is_some() || !report.dead.is_empty());
    for d in &report.dead {
        if let Some(c) = d.conf {
            anyhow::ensure!(
                (0..=2).contains(&c),
                "confidence {c} outside 0..2 — wire skew"
            );
        }
    }
    Ok(report)
}

/// A degraded judgment is a visible event (A9f): one observe-feed
/// line, through the SAME writer as the guard/audit producers. It
/// used to keep its own copy of the append and stamped none of the
/// feed's contract fields — `hook` where every sibling writes `event`,
/// and no schema / session_id / ts_ms — so the M4 evaluation set
/// counted a line it could not partition, from a producer its golden
/// does not describe (review 2026-08-19). Session is None honestly:
/// `ce deadcode` runs in a terminal, like `ce precommit`.
fn observe(root: &Path, reason: &str) {
    crate::hookio::observe_append(
        root,
        None,
        json!({"event": "deadcode", "degraded": true, "reason": reason}),
    );
}

#[cfg(test)]
mod tests {
    use super::super::nodes::Node;
    use serde_json::json;

    /// The degradation loop closes: a stamped deadcode degradation
    /// is COUNTED by the same health surface `ce doctor` prints —
    /// asserted, not assumed (2h exit row).
    #[test]
    fn degraded_stamp_reaches_the_health_counter() {
        let root = crate::testutil::scratch("dc-observe");
        assert_eq!(crate::health::degraded_runs(&root), (0, 0));
        super::observe(&root, "graph_too_large");
        assert_eq!(crate::health::degraded_runs(&root), (1, 1));
        std::fs::remove_dir_all(&root).ok();
    }

    fn node(path: &str, unit: &str, kind: i64) -> Node {
        Node {
            path: path.into(),
            unit: unit.into(),
            kind,
        }
    }

    /// The 2.18.0 split consumed whole (batch-7 slice 4, the fixture
    /// the inventory found missing): reported rows label sections as
    /// path#unit, the core's fail bit is relayed, and an aggregate
    /// smuggled into the FAILING table refuses as wire skew — that
    /// table licenses erase's class-0 rows and must never carry a
    /// directory.
    #[test]
    fn reported_rows_and_fail_bit_consume_and_skew_refuses() {
        let nodes = vec![
            node("a.rs", "", super::super::wire::GRAN_FILE),
            node("docs/x.md", "Intro", super::super::wire::GRAN_SECTION),
            node("pkg", "", super::super::wire::GRAN_PACKAGE),
        ];
        // the confidence road: a 3-column dead row carries the
        // trust column, a 2-column (legacy) row answers None below
        let reply = json!({
            "dead": [[0, 1, 2]], "reported": [[1, 3], [2, 1]],
            "fail": true, "counts": {"kept": 7}
        });
        let r = super::consume(&reply, &nodes, 0).expect("consume");
        assert_eq!(r.dead.len(), 1);
        let d = &r.dead[0];
        assert_eq!(
            (d.path.as_str(), d.verdict, d.why.as_str(), d.conf),
            (
                "a.rs",
                "unref_private",
                "no kept in-edge and no entry flag",
                Some(2)
            )
        );
        assert_eq!(
            r.reported,
            vec![
                ("docs/x.md#Intro".into(), "unreach_private"),
                ("pkg".into(), "unref_private"),
            ]
        );
        assert!(r.fail && r.kept == 7);
        let skew = json!({"dead": [[2, 1]], "counts": {}});
        let err = super::consume(&skew, &nodes, 0).expect_err("aggregate in dead");
        assert!(err.to_string().contains("wire skew"), "{err}");
        // pre-2.18 core: no fail bit — the old conjunction stands in
        let old = json!({"dead": [[0, 1]], "counts": {}});
        assert!(super::consume(&old, &nodes, 0).expect("old core").fail);
    }
}
