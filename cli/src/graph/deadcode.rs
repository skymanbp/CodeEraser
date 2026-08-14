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

use super::load::graph_rows;
use super::nodes::{self, Node};
use crate::config::Config;
use crate::corelink::Link;
use crate::dedup;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::collections::BTreeSet;
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
}

pub fn run(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Report> {
    let w = build_wire(root, db)?;
    let reply = judge(core, &w, &[])?;
    let report = consume(&reply, &w.nodes, w.unresolved_sites)?;
    if let Some(reason) = &report.degraded {
        observe(root, reason);
    }
    Ok(report)
}

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

pub fn build_wire(root: &Path, db: Option<PathBuf>) -> Result<GraphWire> {
    let (idx, db_path) = dedup::refreshed_index(root, db)?;
    let (files, edges, unresolved_sites) = graph_rows(&idx)?;
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
    let rows: Vec<Value> = nodes.iter().map(|n| node_row(n, &config)).collect();
    let mut wire: BTreeSet<[i64; 4]> = edges
        .iter()
        .filter(|e| e.kind != super::wire::EDGE_ASSET)
        .map(|e| {
            let s = ids[&(e.src.as_str(), "")] as i64;
            let d = ids[&(e.dst_path.as_str(), e.dst_unit.as_str())] as i64;
            [s, d, e.kind, e.rung]
        })
        .collect();
    nodes::contain(&nodes, &ids, &mut wire);
    Ok(GraphWire {
        nodes,
        rows,
        edges: wire,
        unresolved_sites,
    })
}

/// [lang, kind, flags] — only file nodes carry entry flags.
fn node_row(n: &Node, config: &Config) -> Value {
    // unknown extension = the sentinel code, NOT Python's 0 (RM15:
    // the two were indistinguishable on the wire before 3k)
    let lang = crate::scan::lang::Lang::from_path(Path::new(&n.path))
        .map(|l| l as i64)
        .unwrap_or(crate::scan::lang::Lang::LangUnknown as i64);
    let flags = if n.kind == super::wire::GRAN_FILE {
        flags_of(&n.path, config)
    } else {
        0
    };
    json!([lang, n.kind, flags])
}

/// Mechanical entry conventions (module header); bit 0 (exported)
/// stays unset at file granularity — public-ness is a symbol fact.
fn flags_of(path: &str, config: &Config) -> i64 {
    let base = path.rsplit('/').next().unwrap_or(path);
    let mut f = 0i64;
    if matches!(base, "main.rs" | "main.go" | "__main__.py" | "build.rs")
        || ["src/bin/", "examples/", "benches/", "cmd/"]
            .iter()
            .any(|p| path.starts_with(p))
    {
        f |= 1 << 1;
    }
    if is_test(path, base) {
        f |= 1 << 2;
    }
    if config
        .graph
        .entry_globs
        .iter()
        .any(|g| glob_hit(g, path, base))
    {
        f |= 1 << 3;
    }
    if matches!(base, "README.md" | "CLAUDE.md")
        || (path.starts_with("docs/") && matches!(base, "index.md" | "README.md"))
    {
        f |= 1 << 5;
    }
    f
}

fn is_test(path: &str, base: &str) -> bool {
    base.ends_with("_test.go")
        || base.ends_with(".test.ts")
        || (base.starts_with("test_") && base.ends_with(".py"))
        || path.starts_with("tests/")
        || path.contains("/tests/")
        || path.contains("/__tests__/")
}

/// entry_globs matching: an exact path, a `dir/` prefix, or a
/// `*.ext` basename pattern — the declarative subset the config
/// documents (full glob syntax is not promised).
fn glob_hit(glob: &str, path: &str, base: &str) -> bool {
    if let Some(ext) = glob.strip_prefix("*.") {
        return base.ends_with(&format!(".{ext}"));
    }
    glob == path || glob == base || (glob.ends_with('/') && path.starts_with(glob))
}

/// One graph.request over the open core link; a missing capability
/// or a non-result reply is an error, never an empty report. `pos`
/// asks for position rows (the join's leg; deadcode sends none), and
/// a non-degraded reply MUST answer every requested index — a short
/// pos table would silently starve the join, so it refuses here.
pub fn judge(core: &str, w: &GraphWire, pos: &[i64]) -> Result<Value> {
    let (mut link, _hello) = Link::open(core).map_err(anyhow::Error::msg)?;
    if !link.has("graph/1") {
        bail!("ce-core offers no graph/1 capability — upgrade the core");
    }
    let body = json!({
        "nodes": w.rows,
        "edges": w.edges.iter().collect::<Vec<_>>(),
        "pos": pos,
    });
    let reply = link.request("graph", body).map_err(anyhow::Error::msg)?;
    if reply["type"] != json!("graph.result") {
        bail!("core replied {}: {reply}", reply["type"]);
    }
    let rows = reply["pos"].as_array().map(Vec::len).unwrap_or(0);
    if reply["degraded"] != json!(true) && rows != pos.len() {
        bail!("graph.result answered {rows} of {} pos rows", pos.len());
    }
    Ok(reply)
}

fn consume(reply: &Value, nodes: &[Node], unresolved_sites: i64) -> Result<Report> {
    let mut report = Report {
        dead: Vec::new(),
        reported: Vec::new(),
        nodes: nodes.len(),
        kept: reply["counts"]["kept"].as_u64().unwrap_or(0),
        unresolved_sites,
        degraded: reply["reason"].as_str().map(str::to_string),
    };
    let dead: Vec<[usize; 2]> =
        serde_json::from_value(reply["dead"].clone()).context("dead rows")?;
    for [idx, verdict] in dead {
        let node = nodes.get(idx).context("index out of range")?;
        let name = VERDICT_NAMES[verdict.checked_sub(1).context("verdict 0")?];
        if node.kind == super::wire::GRAN_FILE {
            let why = if verdict <= 2 {
                "no kept in-edge and no entry flag"
            } else {
                "referenced only from dead code; no entry flag"
            };
            report.dead.push((node.path.clone(), name, why.to_string()));
        } else if node.unit.is_empty() {
            report.reported.push((node.path.clone(), name));
        } else {
            report
                .reported
                .push((format!("{}#{}", node.path, node.unit), name));
        }
    }
    Ok(report)
}

/// A degraded judgment is a visible event (A9f): one observe-feed
/// line, same contract as the guard/audit producers.
fn observe(root: &Path, reason: &str) {
    let line = json!({"hook": "deadcode", "degraded": true, "reason": reason});
    let path = root.join(".ce/observe.ndjson");
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        use std::io::Write;
        let _ = writeln!(f, "{line}");
    }
}

#[cfg(test)]
mod tests {
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
}
