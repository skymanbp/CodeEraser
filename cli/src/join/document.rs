//! The join document (plan v2.32 step 4; design booklet
//! docs/reference/authority-track.md §5): the core lays it out
//! (document/1, CE.Join.Document) from one path table — every path a
//! file row, a unit row or a verdict candidate names, with each
//! path's place in string order — the file pairs' sums and churn, the
//! graph reply's positions, the window's co-change rows, the verdict
//! reply's candidates and severities re-numbered into the table, the
//! unit rows, and each degraded reply's reason by its code in the
//! package's list. This side puts the paths and the unit keys back.

use super::churn_unit::{Lines, UnitRow, UnitSim};
use super::{Pos, verdicts::Judged};
use crate::document::{self, Paths, Request, Resolve, Why};
use anyhow::{Context, Result};
use std::collections::{BTreeMap, HashMap};

/// The tables a join request carries.
const TABLES: [&str; 9] = [
    "rankPaths",
    "files",
    "pos",
    "cochange",
    "candidates",
    "joinSeverity",
    "units",
    "graphReason",
    "verdictReason",
];

/// One file pair's sums: blocks, tokens, near-miss pairs.
pub(super) type Sums = BTreeMap<(String, String), (usize, usize, usize)>;

/// What the document is assembled from, measured and judged.
pub(super) struct Parts<'a> {
    pub days: u32,
    pub churn: &'a crate::churn::Report,
    pub pairs: &'a Sums,
    pub units: &'a [UnitRow],
    pub posmap: &'a HashMap<String, Pos>,
    pub graph_degraded: Option<String>,
    pub judged: &'a Judged,
}

/// The document over `p`, laid out over the verdict's link `held` (a
/// fresh one to `core` when that link is spent).
pub(super) fn assemble(
    core: &str,
    held: document::Held,
    p: &Parts<'_>,
) -> Result<document::Answer> {
    let (req, names) = request(p)?;
    document::assemble_over(core, held, req, &names)
}

fn request(p: &Parts<'_>) -> Result<(Request, Names)> {
    let mut paths = Paths::default();
    let files = file_rows(p, &mut paths);
    let units = unit_rows(p.units, &mut paths);
    let candidates = candidate_rows(p.judged, &mut paths)?;
    let pos: Vec<[i64; 6]> = (0..paths.list.len())
        .filter_map(|i| {
            let [a, b, c, d, e] = *p.posmap.get(&paths.list[i])?;
            Some([i as i64, a, b, c, d, e])
        })
        .collect();
    let cochange: Vec<[i64; 3]> = p
        .churn
        .cochange
        .iter()
        .filter_map(|(x, y, n)| Some([paths.find(x)?, paths.find(y)?, *n as i64]))
        .collect();
    let rank = document::ranks(paths.list.iter().map(String::as_str));
    let rank_rows: Vec<[usize; 2]> = rank.iter().enumerate().map(|(i, r)| [i, *r]).collect();
    let req = Request::new("join")
        .range("paths", paths.list.len())
        .range("units", p.units.len())
        .range("why", 0)
        .fact("days", p.days)
        .fact("commits", p.churn.commits)
        .rows("rankPaths", rank_rows)
        .rows("files", files)
        .rows("pos", pos)
        .rows("cochange", cochange)
        .rows("candidates", candidates)
        .rows("joinSeverity", &p.judged.join_severity)
        .rows("units", units);
    let req = reasons(req, p)?;
    let keys = p
        .units
        .iter()
        .map(|u| [u.a.key.clone(), u.b.key.clone()])
        .collect();
    let names = Names {
        paths: paths.list,
        keys,
        why: Why::default(),
    };
    Ok((req.empty(&TABLES), names))
}

/// Each degraded leg's reason by its code in the package's list.
fn reasons(mut req: Request, p: &Parts<'_>) -> Result<Request> {
    let listed = crate::tables::get().document.join.reasons;
    for (table, why) in [
        ("graphReason", &p.graph_degraded),
        ("verdictReason", &p.judged.degraded),
    ] {
        if let Some(why) = why {
            let code = listed.iter().position(|r| r == why).with_context(|| {
                format!("a join leg degraded for a reason the package does not list: {why}")
            })?;
            req = req.rows(table, [[code]]);
        }
    }
    Ok(req)
}

/// [a, b, blocks, tokens, near-miss, a appended, a rewrote, b
/// appended, b rewrote] per file pair, in pair order.
fn file_rows(p: &Parts<'_>, paths: &mut Paths) -> Vec<[i64; 9]> {
    let churn = file_churn(p.churn);
    let churned = |path: &str| churn.get(path).copied().unwrap_or_default();
    p.pairs
        .iter()
        .map(|((a, b), &(blocks, tokens, near))| {
            let (ca, cb) = (churned(a), churned(b));
            [
                paths.id(a),
                paths.id(b),
                blocks as i64,
                tokens as i64,
                near as i64,
                ca.appended as i64,
                ca.rewrote as i64,
                cb.appended as i64,
                cb.rewrote as i64,
            ]
        })
        .collect()
}

/// The verdict reply's candidates, their two file columns re-numbered
/// from the verdict universe into the path table.
fn candidate_rows(j: &Judged, paths: &mut Paths) -> Result<Vec<[i64; 6]>> {
    j.candidates
        .iter()
        .map(|&[u, v, code, reasons, legs, conf]| {
            let path = |i: i64| {
                usize::try_from(i)
                    .ok()
                    .and_then(|i| j.files.get(i))
                    .context("candidate index outside the file universe — wire skew")
            };
            Ok([
                paths.id(path(u)?),
                paths.id(path(v)?),
                code,
                reasons,
                legs,
                conf,
            ])
        })
        .collect()
}

/// [k, a path, a nth, b path, b nth, family, three metric columns,
/// the two sides' churn].
fn unit_rows(units: &[UnitRow], paths: &mut Paths) -> Vec<[i64; 13]> {
    units
        .iter()
        .enumerate()
        .map(|(k, u)| {
            let (family, m) = match u.sim {
                UnitSim::T1t2 { tokens } => (0, [tokens as i64, 0, 0]),
                UnitSim::T3 { ted, n1, n2 } => (1, [ted, n1, n2]),
            };
            [
                k as i64,
                paths.id(&u.a.path),
                u.a.nth,
                paths.id(&u.b.path),
                u.b.nth,
                family,
                m[0],
                m[1],
                m[2],
                u.churn_a.appended as i64,
                u.churn_a.rewrote as i64,
                u.churn_b.appended as i64,
                u.churn_b.rewrote as i64,
            ]
        })
        .collect()
}

/// Per-file churn sums over the per-unit ledger — derived the same
/// way the report totals are (one bookkeeping, summed two ways).
fn file_churn(ch: &crate::churn::Report) -> HashMap<&str, Lines> {
    let mut map: HashMap<&str, Lines> = HashMap::new();
    for u in &ch.units {
        let e = map.entry(u.path.as_str()).or_default();
        e.appended += u.appended;
        e.rewrote += u.rewrote;
    }
    map
}

/// The join document's strings: the paths and each unit row's keys.
struct Names {
    paths: Vec<String>,
    keys: Vec<[String; 2]>,
    why: Why,
}

impl Resolve for Names {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        match class {
            "path" => document::at(&self.paths, ints),
            "key" => match ints {
                [k, side] => {
                    let row = self.keys.get(usize::try_from(*k).ok()?)?;
                    row.get(usize::try_from(*side).ok()?).cloned()
                }
                _ => None,
            },
            "why" => self.why.at(ints),
            _ => None,
        }
    }
}
