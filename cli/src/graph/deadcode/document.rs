//! The deadcode document (plan v2.32 step 4; design booklet
//! docs/reference/authority-track.md §5): the core lays it out
//! (document/1, CE.Graph.Document) from the judgment's own rows — the
//! dead and reported rows, the kept count, the degraded reason by its
//! code in the package's list, one advisory row per name this side's
//! table holds — and this side puts the paths, the section labels and
//! the names back. `Report` is the document read back for the console,
//! the exit and the readers that act on a verdict (erase, the tests);
//! the codes beside a row are the judgment's, the strings the
//! document's. The graph screen sends the same tables (graph/canvas.rs).

use super::advisory::Advised;
use super::{GraphWire, Judged};
use crate::document::{self, Request, Resolve, Why};
use crate::graph::nodes::Node;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

/// The tables a deadcode request carries.
pub(crate) const TABLES: [&str; 5] = ["kept", "reason", "dead", "reported", "unmentioned"];

/// The deadcode document read back.
#[derive(Debug, Deserialize)]
pub struct Report {
    pub dead: Vec<DeadRow>,
    pub reported: Vec<Reported>,
    pub counts: Counts,
    pub unresolved_sites: i64,
    pub degraded: Option<String>,
    /// The symbol advisory: None when the road was not asked.
    #[serde(default)]
    pub unmentioned: Option<Vec<AdvisoryRow>>,
    #[serde(default)]
    pub unmentioned_dropped: bool,
    #[serde(default)]
    pub unmentioned_cut: bool,
    /// File-tier nodes alone — the console hint's denominator.
    #[serde(skip)]
    pub files: usize,
    /// The core's gate bit (2.18.0).
    #[serde(skip)]
    pub fail: bool,
    /// The bound document itself, the machine faces' print.
    #[serde(skip)]
    pub doc: Value,
}

/// One dead file as the document names it, and the verdict code the
/// judgment gave it (1..4, CE.Graph.Dead).
#[derive(Debug, Deserialize)]
pub struct DeadRow {
    #[serde(rename = "name")]
    pub path: String,
    pub verdict: String,
    pub why: String,
    #[serde(rename = "whyCode")]
    pub why_code: usize,
    #[serde(rename = "confidence")]
    pub conf: Option<i64>,
    #[serde(skip)]
    pub code: i64,
}

/// An aggregate's verdict, reported and never called dead.
#[derive(Debug, Deserialize)]
pub struct Reported {
    pub name: String,
    pub verdict: String,
}

#[derive(Debug, Deserialize)]
pub struct Counts {
    pub nodes: usize,
    pub kept_edges: u64,
}

/// One advisory row as the document names it, and the judgment's code
/// (CE.Graph.Advisory.code) beside it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AdvisoryRow {
    pub name: String,
    pub symbol: String,
    pub line: i64,
    pub code: String,
    pub why: String,
    #[serde(skip)]
    pub code_ix: usize,
}

/// The deadcode request over one judgment: `nodes` the wire's dense
/// assignment, the advisory one row per name, and the strings the
/// document refers to.
pub(crate) fn request<'a>(
    family: &'static str,
    w: &'a GraphWire,
    j: &Judged,
) -> Result<(Request, Names<'a>)> {
    let reasons = crate::tables::get().document.deadcode.reasons;
    let mut req = Request::new(family)
        .range("nodes", w.nodes.len())
        .range("why", 0)
        .fact("unresolvedSites", w.unresolved_sites)
        .rows("dead", &j.dead)
        .rows("reported", &j.reported)
        .single("kept", j.kept);
    if let Some(reason) = &j.degraded {
        let code = reasons.iter().position(|r| r == reason).with_context(|| {
            format!("graph reply degraded for a reason the package does not list: {reason}")
        })?;
        req = req.rows("reason", [[code]]);
    }
    let mut symbols = Vec::new();
    let mut rows: Vec<[i64; 4]> = Vec::new();
    let (mut asked, mut dropped, mut cut) = (false, false, false);
    match &j.advisory {
        None => {}
        Some(Advised::Dropped) => (asked, dropped) = (true, true),
        Some(Advised::Rows {
            rows: named,
            cut: c,
        }) => {
            (asked, cut) = (true, *c);
            for a in named {
                rows.push([rows.len() as i64, a.node, a.line, a.code]);
                symbols.push(a.symbol.clone());
            }
        }
    }
    let req = req
        .range("advisory", rows.len())
        .rows("unmentioned", rows)
        .fact("asked", i64::from(asked))
        .fact("dropped", i64::from(dropped))
        .fact("cut", i64::from(cut))
        .empty(&TABLES);
    let names = Names {
        nodes: &w.nodes,
        symbols,
        why: Why::default(),
    };
    Ok((req, names))
}

/// The bound document read back, the judgment's codes beside its rows.
pub(crate) fn read(doc: Value, w: &GraphWire, j: &Judged) -> Result<Report> {
    let mut r: Report = document::read(&doc, "deadcode")?;
    for (row, raw) in r.dead.iter_mut().zip(&j.dead) {
        row.code = raw[1];
    }
    if let (Some(rows), Some(Advised::Rows { rows: raw, .. })) = (&mut r.unmentioned, &j.advisory) {
        for (row, a) in rows.iter_mut().zip(raw) {
            row.code_ix = a.code as usize;
        }
    }
    r.files = w
        .nodes
        .iter()
        .filter(|n| n.kind == crate::graph::wire::GRAN_FILE)
        .count();
    r.fail = j.fail;
    r.doc = doc;
    Ok(r)
}

/// The deadcode document's strings: the node paths, a section's
/// `path#unit` label, the advisory's names.
pub(crate) struct Names<'a> {
    nodes: &'a [Node],
    symbols: Vec<String>,
    why: Why,
}

impl Resolve for Names<'_> {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        let node = || {
            let [i] = ints else { return None };
            usize::try_from(*i).ok().and_then(|i| self.nodes.get(i))
        };
        match class {
            "path" => node().map(|n| n.path.clone()),
            "node_name" => node().map(|n| {
                if n.unit.is_empty() {
                    n.path.clone()
                } else {
                    format!("{}#{}", n.path, n.unit)
                }
            }),
            "symbol" => document::at(&self.symbols, ints),
            "why" => self.why.at(ints),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/deadcode/document.rs"]
mod tests;
