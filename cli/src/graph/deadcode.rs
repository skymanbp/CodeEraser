//! `ce deadcode` (M5-2h): refresh the index (the ladder judges every
//! site into the DB on the way), build the graph.request from the
//! cached edges, let the Haskell core judge liveness, and name the
//! verdicts back. An EMPTY graph is an explicit error, never a
//! silent all-dead/all-alive report; a degraded core reply
//! (graph_too_large) lands in the observe feed so doctor/health
//! count it (A9f).
//!
//! Entry flags are mechanical conventions plus user config:
//! main-shaped files and executable directories (bit 1), test
//! conventions (bit 2), ce.toml [graph] entry_globs (bit 3), and the
//! doc entries README.md / CLAUDE.md / docs indexes (bit 5). The
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
    /// (name, verdict name, why) — file nodes only.
    pub dead: Vec<(String, &'static str, String)>,
    /// Section/package verdicts: reported, never called dead.
    pub reported: Vec<(String, &'static str)>,
    pub nodes: usize,
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
    let (files, edges, unresolved_sites) = graph_rows(idx)?;
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
    let rows: Vec<Value> = nodes.iter().map(|n| node_row(root, n, &config)).collect();
    let mut wire = edge_wire(&edges, &ids)?;
    nodes::contain(&nodes, &ids, &mut wire);
    Ok(GraphWire {
        nodes,
        rows,
        edges: wire,
        unresolved_sites,
    })
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

/// [lang, kind, flags] — only file nodes carry entry flags.
fn node_row(root: &Path, n: &Node, config: &Config) -> Value {
    // unknown extension = the sentinel code, NOT Python's 0 (RM15:
    // the two were indistinguishable on the wire before 3k)
    let lang = crate::scan::lang::Lang::from_path(Path::new(&n.path))
        .map(|l| l as i64)
        .unwrap_or(crate::scan::lang::Lang::LangUnknown as i64);
    let flags = if n.kind == super::wire::GRAN_FILE {
        flags::flags_of(root, &n.path, config)
    } else {
        0
    };
    json!([lang, n.kind, flags])
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
    });
    let reply = link.request("graph", body).map_err(anyhow::Error::msg)?;
    let rows = reply["pos"].as_array().map(Vec::len).unwrap_or(0);
    if reply["degraded"] != json!(true) && rows != pos.len() {
        bail!("graph.result answered {rows} of {} pos rows", pos.len());
    }
    Ok(reply)
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
        kept: reply["counts"]["kept"].as_u64().unwrap_or(0),
        unresolved_sites,
        degraded: reply["reason"].as_str().map(str::to_string),
        fail: false,
    };
    let dead: Vec<[usize; 2]> =
        serde_json::from_value(reply["dead"].clone()).context("dead rows")?;
    for [idx, verdict] in dead {
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
        report.dead.push((node.path.clone(), name, why.to_string()));
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
        let reply = json!({
            "dead": [[0, 1]], "reported": [[1, 3], [2, 1]],
            "fail": true, "counts": {"kept": 7}
        });
        let r = super::consume(&reply, &nodes, 0).expect("consume");
        assert_eq!(
            r.dead,
            vec![(
                "a.rs".into(),
                "unref_private",
                "no kept in-edge and no entry flag".into()
            )]
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
