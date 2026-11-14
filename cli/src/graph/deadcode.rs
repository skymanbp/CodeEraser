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
//! as references (design §4 Markdown row), and an asset NODE — a
//! walked file the index holds no parse of, a page's stylesheet or
//! image (plan v2.30 step 5) — is never a candidate: its role lands
//! on the core's dyn-referenced bit; a package node gets
//! SYNTHETIC containment arcs to every file under it, or to the code
//! its manifest declares when it is an R package — reaching a
//! package reaches what it holds (the self-repo disposition run
//! found doc→directory edges stranded from the members the walk had
//! already proven alive); section and package verdicts are REPORTED,
//! never called dead (decision 4 / RG9 — aggregates are not code
//! entities). unreferenced_public stays its own class end to end
//! (RG10); the unresolved-site count travels with the report — the
//! reader sees what the graph refuses to know (decision 5:
//! symbol-level indegree stays out while call edges are off).

mod advisory;
mod arcs;
mod convention;
pub mod cycles;
mod document;
mod flags;
mod gate;
mod reply;
mod targets;

pub use advisory::{Advised, Named};
pub use convention::Position;
pub use cycles::self_loop_nodes;
pub use document::{AdvisoryRow, Counts, DeadRow, Report, Reported};
pub(crate) use flags::ROLE_NAMES;
pub use gate::{Gate, Gated, committed_point, unjudged};
pub use reply::Grouping;
use reply::consume;

use super::load::{GraphEdge, graph_rows};
use super::nodes::{self, Node};
use crate::config::Config;
use crate::dedup;
use crate::document::Answer;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The judgment's own rows, held at the boundary (a verdict 1..4, a
/// dead row at file granularity with its trust 0..2) before the
/// document names them (plan v2.32 step 4): what the screen's document
/// is sent beside the deadcode tables, the codes the readers that act
/// on a verdict take beside the document, and all structure's rollup
/// needs.
#[derive(Debug)]
pub struct Judged {
    /// [node, verdict] or, when the ledger rode (2.32.0), [node,
    /// verdict, trust] — arity is the road, not noise.
    pub dead: Vec<Vec<i64>>,
    /// [node, verdict]: sections and packages, reported, never dead.
    pub reported: Vec<[i64; 2]>,
    /// The kept edges the reply counted; None on a degraded reply.
    pub kept: Option<i64>,
    pub degraded: Option<String>,
    /// The core's gate bit (2.18.0): any file-tier dead verdict, or
    /// a degraded run, fails — the exit is a relay, not a policy.
    pub fail: bool,
    /// The symbol-level advisory (6.2.0): None when the wire was built
    /// without it (`Advisory::No`); an advisory, never a verdict.
    pub advisory: Option<Advised>,
    /// The core's roots and dependents of the dead rows (plan v2.35).
    pub grouping: Grouping,
    /// The core's answer to the dead-file gate (plan v2.35, 5B).
    pub gated: Gated,
}

/// Whether the wire carries the two advisory tables (6.2.0). ONE
/// parameter on ONE function (`wire_of`): the mention pass, the
/// candidate table and the mounts table are measured together or not
/// at all, and a wire built with them is judged with them — a
/// `wire_of(Yes) + judge(No)` state is not representable (W4-F3).
/// Only the faces that render the advisory ask for it — `ce deadcode`
/// and the GUI's graph screen, once for both of its halves — and the
/// erase, score, structure and join roads pay nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Advisory {
    No,
    Yes,
}

pub fn run(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Report> {
    judge_report(root, core, &face_wire(root, db)?)
}

/// The judgment and its document over a wire already in hand (batch 9
/// P10 half-door): the core judges, then lays the deadcode document
/// out (plan v2.32 step 4) — boundaries holding the one snapshot call
/// this, never a second measurement.
pub fn judge_report(root: &Path, core: &str, w: &GraphWire) -> Result<Report> {
    let (answer, j) = said(root, core, w, false)?;
    document::read(answer.document, &j)
}

/// `ce deadcode`, the MCP tool and the GUI's face (plan v2.32 step 5):
/// the document and the console lines the core lays out, and its veto
/// — `check` is the console's `--check`, a fact only the lines and the
/// veto read.
pub fn answer(root: &Path, db: Option<PathBuf>, core: &str, check: bool) -> Result<Answer> {
    Ok(said(root, core, &face_wire(root, db)?, check)?.0)
}

/// The deadcode face's wire, the one its doors and the graph screen
/// judge (`run`, the library's; `answer`, the console's, the MCP tool's
/// and the GUI's deadcode command; graph::canvas, the screen, since plan
/// v2.35 5C): the advisory asked, and the dead-file gate (plan v2.35,
/// 5B) — the committed point and the declared mask. The erase, score,
/// structure and join roads send no gate here (score::run sends its
/// own).
pub(crate) fn face_wire(root: &Path, db: Option<PathBuf>) -> Result<GraphWire> {
    let (idx, db_path) = dedup::refreshed_index(root, db)?;
    let mut w = wire_of(root, &idx, &db_path, Advisory::Yes)?;
    let block = Config::load(root)
        .map_err(anyhow::Error::msg)?
        .deadcode
        .mask();
    w.gate = Some(Gate::of(root, &w, committed_point(root)?, block));
    Ok(w)
}

/// The judgment, then its document and lines over the judgment's link.
fn said(root: &Path, core: &str, w: &GraphWire, check: bool) -> Result<(Answer, Judged)> {
    let (j, _, held) = judged(root, core, w, &[])?;
    let req = face_request(("deadcode", check), w, &j)?;
    Ok((crate::document::assemble_over(core, held, req)?, j))
}

/// The deadcode document's request over one judgment: the tables, then
/// the grouping and the gate the reply carried (reply::on_request) — the
/// one request the deadcode face and the graph screen's deadcode half
/// both send, so the screen's half is the face's document (plan v2.35
/// 5C). `face` is the family asked and the console's `--check`.
pub(crate) fn face_request(
    face: (&'static str, bool),
    w: &GraphWire,
    j: &Judged,
) -> Result<crate::document::Request> {
    Ok(reply::on_request(
        document::request(face, w, j)?,
        j,
        w.gate.as_ref(),
    ))
}

/// The judgment alone, answering position rows too — the canvas
/// face (batch 9 P18) needs the verdicts AND the pos table from the
/// ONE judgment, and structure's rollup reads the dead nodes and no
/// document; returning the raw reply beside the rows keeps consume
/// private and the observe in one owner. The link comes back whole,
/// so a face's document rides the judgment's core process (plan v2.32
/// step 4).
pub fn judged(
    root: &Path,
    core: &str,
    w: &GraphWire,
    pos: &[i64],
) -> Result<(Judged, Value, crate::document::Held)> {
    let mut link = crate::corelink::judged::open(core, "graph/1")?;
    let reply = judge_on(&mut link, w, pos)?;
    let j = consume(&reply, &w.nodes, w.unmentioned.as_ref(), w.gate.as_ref())?;
    if let Some(reason) = &j.degraded {
        observe(root, reason);
    }
    Ok((j, reply, Ok(link)))
}

/// A raw reply held at the boundary over the wire that asked it (the
/// check road's read: the positions and the gate's answer off one
/// judgment, without the face's observe line).
pub fn read(reply: &Value, w: &GraphWire) -> Result<Judged> {
    consume(reply, &w.nodes, w.unmentioned.as_ref(), w.gate.as_ref())
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
    /// The per-language site ledger [[lang, unresolved, total]],
    /// langs ascending (2.32.0, H3) — the core judges each dead
    /// row's confidence from it.
    pub unres: Vec<[i64; 3]>,
    /// The export surface [[nodeIdx, visibility]] (4.1.0): which
    /// files declare something and how visibly. The core reads the
    /// public bit off it (graph/symwire.rs).
    pub symbols: BTreeSet<[i64; 2]>,
    /// The `unmentioned` table (6.2.0) keyed `[node, vis, conv]` with
    /// the names behind each key and the producer's cut flag — the
    /// wire carries the KEYS, the renderer reads the names; Some only
    /// under `Advisory::Yes` (mention/candidates.rs builds it: keys
    /// ascending and deduped by the map, every value non-empty by its
    /// one writer).
    pub unmentioned: Option<crate::mention::Unmentioned>,
    /// The `mounts` table (6.2.0): node → [private, total, bits] for
    /// EVERY node (graph/mounts.rs builds it over the whole node
    /// list; the core reads a missing row as zeros, so none may be
    /// missing); Some only under `Advisory::Yes`.
    pub mounts: Option<BTreeMap<i64, [i64; 3]>>,
    /// `[graph] scc_floor` as declared (6.4.0, O59), sent as `sccFloor`
    /// when Some — the verdict road sends the same number as threshold
    /// code 7, so one config key drives both faces.
    pub scc_floor: Option<u32>,
    /// The dead-file gate (plan v2.35, 5B): None from `wire_of`; the
    /// faces that judge the gate set it (`answer`, score::run).
    pub gate: Option<Gate>,
}

/// The file-tier slice of the wire's dense assignment: (index, path)
/// in node order, foreign readers included — the canvas draws the
/// whole graph through this selector.
pub fn file_nodes(w: &GraphWire) -> Vec<(i64, &str)> {
    files_where(w, |_| true)
}

/// The MEASURED file tier: the file nodes that are this tree's own —
/// the join, the score and the structure family assemble their pos
/// requests and tier universes through this ONE selector (the ratchet
/// caught the second copy growing), so a foreign reader is never a
/// row of a verdict here (plan v2.18 step #12), and neither is a
/// walked asset (plan v2.30 step 5): the index holds no parse of it,
/// the size and clone tiers never saw it, and the structure family
/// has no directory row to place it in (it refused the first one as
/// "outside the walked tree").
pub fn measured_nodes(w: &GraphWire) -> Vec<(i64, &str)> {
    files_where(w, |n| !n.foreign && !n.asset)
}

fn files_where(w: &GraphWire, keep: impl Fn(&Node) -> bool) -> Vec<(i64, &str)> {
    w.nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.kind == super::wire::GRAN_FILE && keep(n))
        .map(|(i, n)| (i as i64, n.path.as_str()))
        .collect()
}

/// The measurement half-door (batch 9 P10): the graph request from
/// an index the command boundary already refreshed and opened. Under
/// `Advisory::Yes` the two advisory tables ride beside it
/// (advisory::tables — the mention pass and the mounts off the same
/// index).
pub fn wire_of(
    root: &Path,
    idx: &dedup::index::Index,
    db_path: &Path,
    advisory: Advisory,
) -> Result<GraphWire> {
    let (files, edges, unresolved_sites, sites) = graph_rows(idx)?;
    // the refusal guards a command run in the wrong directory, so it asks
    // the index, not the graph: a tree whose files the graph reads none
    // of (prose only) is judged on an empty graph (release 1.10.0)
    if files.is_empty() && idx.file_count()? == 0 {
        bail!(
            "empty index at {} — nothing was walked; wrong root?",
            db_path.display()
        );
    }
    let config = Config::load(root).map_err(anyhow::Error::msg)?;
    let entries = arcs::entry_set(root, &config)?;
    // identity assignment + containment live in the nodes.rs throat
    // (F19) — the M5-3 join consumes the SAME functions, so both
    // verdicts stand on one id space by construction; the foreign
    // mark is the index's own per-file fact (walk::Walked)
    let nodes = nodes::nodes_of(&files, &edges, &idx.foreign_paths()?);
    let ids = nodes::ids(&nodes);
    let (rows, wire) =
        arcs::rows_and_arcs(root, (&config, &entries), (&files, &edges), (&nodes, &ids))?;
    let symbols = super::symwire::export_surface(idx, &ids)?;
    let (unmentioned, mounts) = match advisory {
        Advisory::No => (None, None),
        Advisory::Yes => {
            let (names, rows) = advisory::tables(root, idx, &nodes, &ids)?;
            (Some(names), Some(rows))
        }
    };
    Ok(GraphWire {
        nodes,
        rows,
        edges: wire,
        unresolved_sites,
        unres: lang_ledger(&sites),
        symbols,
        unmentioned,
        mounts,
        scc_floor: config.graph.scc_floor,
        gate: None,
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

/// [lang, kind, roles] — only file nodes carry entry facts, with TWO
/// exceptions: since 6.3.0 a foreign node of ANY kind carries role 7
/// alone (a submodule's package or section is a reader exactly like
/// its files, and must never surface as a `reported` row here), and
/// since 7.2.0 (plan v2.30 step 5) a walked asset carries role 9
/// alone (a file the index holds no parse of is nobody's main or
/// test, and the allow-claim read would only cost the bytes). The
/// roles column is the 2.28.0 authority (the core derives the entry
/// bits through its role table, where an ablation can perturb them);
/// the pre-2.28 legacy flags column that used to sit between kind
/// and roles retired at 5.0.0, computed and sent for seven minors
/// after the last core stopped reading it.
fn node_row(
    root: &Path,
    n: &Node,
    entries: &crate::scan::globs::Inclusions,
    declared: &targets::Declared,
) -> Value {
    // unknown extension = the sentinel code, NOT Python's 0 (RM15:
    // the two were indistinguishable on the wire before 3k)
    let lang = crate::scan::lang::Lang::from_path(Path::new(&n.path))
        .map(|l| l as i64)
        .unwrap_or(crate::scan::lang::Lang::LangUnknown as i64);
    let roles = if n.foreign {
        flags::ROLE_FOREIGN
    } else if n.asset {
        flags::ROLE_ASSET
    } else if n.kind == super::wire::GRAN_FILE {
        flags::roles_of(root, &n.path, entries, declared)
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
    judge_on(&mut crate::corelink::judged::open(core, "graph/1")?, w, pos)
}

/// The graph.request over a link already open.
fn judge_on(link: &mut crate::corelink::Link, w: &GraphWire, pos: &[i64]) -> Result<Value> {
    let reply = link
        .request("graph", request_body(w, pos))
        .map_err(anyhow::Error::msg)?;
    let rows = reply["pos"].as_array().map(Vec::len).unwrap_or(0);
    if reply["degraded"] != json!(true) && rows != pos.len() {
        bail!("graph.result answered {rows} of {} pos rows", pos.len());
    }
    Ok(reply)
}

/// The graph.request body — ONE producer, shared with the K6/K16
/// legs so a key added here lands in their assertions. The five
/// legacy keys are unconditional (the judged-language set is the
/// core's own language table since 8.0.0); the two advisory tables ride only when the wire
/// carries them (6.2.0), inserted rather than spelled in a second
/// `json!` literal. Key order is not a wire fact:
/// serde_json's Map is a BTreeMap (no `preserve_order` feature), so
/// an inserted key lands where a literal one would (K16 (a)). The
/// mounts map flattens to `[node, private, total, bits]` rows the way
/// `lang_ledger` flattens its map — a `BTreeMap` collected as pairs
/// would serialize nested (F6-18) — and the unmentioned table sends
/// its KEYS alone: the names beside them never cross (K6).
pub fn request_body(w: &GraphWire, pos: &[i64]) -> Value {
    let mut body = json!({
        "nodes": w.rows,
        "edges": w.edges.iter().collect::<Vec<_>>(),
        "pos": pos,
        "unres": w.unres,
        "symbols": w.symbols.iter().collect::<Vec<_>>(),
    });
    let obj = body.as_object_mut().expect("json! object literal");
    // the one `[graph] scc_floor` (6.4.0, O59): the core's cycle table
    // and the verdict's cycle axis read the same floor, so a singleton
    // SCC is a cycle on both faces or on neither; absent = the shipped 2
    if let Some(f) = w.scc_floor {
        obj.insert("sccFloor".into(), json!(f));
    }
    if let Some(m) = &w.mounts {
        let rows: Vec<[i64; 4]> = m.iter().map(|(&n, &[p, t, b])| [n, p, t, b]).collect();
        obj.insert("mounts".into(), json!(rows));
    }
    if let Some(u) = &w.unmentioned {
        obj.insert(
            "unmentioned".into(),
            json!(u.names.keys().collect::<Vec<_>>()),
        );
    }
    if let Some(g) = &w.gate {
        g.insert(obj);
    }
    body
}

/// Those nodes re-keyed to the verdict universe — the measured nodes'
/// own order, so the table is ascending by construction; the score
/// and join roads both send it as `cycleSelfLoops` at floor 1.
pub fn self_loop_rows(w: &GraphWire, loops: &[i64]) -> Vec<i64> {
    measured_nodes(w)
        .iter()
        .enumerate()
        .filter(|(_, (n, _))| loops.contains(n))
        .map(|(u, _)| u as i64)
        .collect()
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
#[path = "../../tests/unit/graph/deadcode.rs"]
mod tests;
