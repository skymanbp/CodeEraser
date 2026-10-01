//! The one document of `ce merge`, the MCP tool and the GUI hub (plan
//! v2.31 step 7, design booklet §6.4): every group merge/1 judged —
//! its family, whether its trees are fragments, its members, the
//! parameters a merged function would take, the member kept, the
//! lines saved, whether the merge is feasible and why not — each
//! parameter with every member's text at it, read back through the
//! tree's spans; the groups not sent, counted by why; the core's counts
//! summed over the chunks, beside them `merged_duplicates` (the groups
//! dropped as a second suggestion for a member set, groups.rs); and,
//! when the core could not judge — none
//! started, none offering the family, none answering — the named
//! reason. A core that answers a chunk degraded is a cap-mirror drift
//! and an error, never a document (step-7 ruling 4). Labelling only:
//! every number is the core's (wire.rs), and a document with a
//! `degraded` reason carries no verdict this side reached. Advisory: no
//! gate reads it.

use super::groups::{self, FAMILY_EXACT, Group, Unsendable};
use super::wire::{self, COUNTS, HoleRow, Judged, Suggestion};
use crate::corelink::Link;
use crate::dedup::{self, t3};
use crate::graph::deadcode::{Advisory, wire_of};
use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const SCHEMA_ID: &str = "ce.merge-report/0.1.0";

/// The count this side adds to the core's: the groups dropped because a
/// stronger group already suggests their member set.
pub const MERGED_DUPLICATES: &str = "merged_duplicates";

/// Reason names by code (CE.Merge.Cost): feasible, a position no
/// parameter can stand for (a statement's or any other that is not an
/// expression or a declared name — hence `position`, not `statement`),
/// a type position, a gap across statements, more parameters than the
/// ceiling, no line saved.
pub const REASONS: [&str; 6] = [
    "ok",
    "position",
    "type",
    "spans_statements",
    "too_many_params",
    "no_savings",
];

/// A member: its file, its unit (None for a fragment), `lines` its
/// clone-family span (its identity) and `run` the lines of the run it
/// sent, which the core priced (a whole unit's are its span).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct MemberFace {
    pub path: String,
    pub unit: Option<String>,
    pub lines: [usize; 2],
    pub run: [usize; 2],
}

/// One member's text at a parameter: the source from the hole's first
/// root to its last on that member (step-7 ruling 6) — a leaf or
/// relabel hole's one node, a gap hole's whole forest, "" on an empty
/// side.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValueFace {
    pub member: usize,
    pub text: String,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ParamFace {
    pub param: usize,
    pub values: Vec<ValueFace>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct GroupFace {
    pub group: usize,
    pub family: &'static str,
    pub fragment: bool,
    pub members: Vec<MemberFace>,
    pub params: u64,
    pub kept: usize,
    pub savings: i64,
    pub feasible: bool,
    pub reason: &'static str,
    pub holes: Vec<ParamFace>,
}

#[derive(Serialize)]
pub struct Report {
    pub counts: BTreeMap<&'static str, u64>,
    pub unsendable: Unsendable,
    pub groups: Vec<GroupFace>,
    pub degraded: Option<String>,
}

impl Report {
    fn empty() -> Self {
        Report {
            counts: COUNTS
                .iter()
                .chain([&MERGED_DUPLICATES])
                .map(|k| (*k, 0))
                .collect(),
            unsendable: Unsendable::default(),
            groups: Vec::new(),
            degraded: None,
        }
    }

    fn degraded(why: String) -> Self {
        Report {
            degraded: Some(why),
            ..Report::empty()
        }
    }
}

/// The whole leg: measured, gathered, judged chunk by chunk, labelled.
pub fn run(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Report> {
    let (blocks, idx, db_path) = dedup::snapshot(root, db)?;
    let mut link = match Link::open(core) {
        Ok((link, _)) if link.has(wire::CAP) => link,
        Ok(_) => return Ok(Report::degraded(format!("core offers no {}", wire::CAP))),
        Err(why) => return Ok(Report::degraded(why)),
    };
    let judged = t3::judge_index(root, &idx, core)?;
    let pairs: Vec<(usize, usize)> = judged.clones.iter().map(|&(a, b, _)| (a, b)).collect();
    let (groups, unsendable, merged) = groups::gather(root, &blocks.groups, &judged.units, &pairs)?;
    let indeg = file_indegree(root, &idx, &db_path)?;
    let texts = texts_of(root, &groups)?;
    let mut r = Report {
        unsendable,
        ..Report::empty()
    };
    r.counts.insert(MERGED_DUPLICATES, merged);
    for range in wire::chunks(&groups) {
        let sent: Vec<&Group> = groups[range].iter().collect();
        let body = wire::body(&sent, |p| indeg.get(p).copied().unwrap_or(0));
        let reply = match wire::ask(&mut link, body) {
            Ok(reply) => reply,
            Err(why) => return Ok(Report::degraded(why)),
        };
        let chunk = wire::consume(&reply, &sent).map_err(|e| anyhow::anyhow!("merge: {e}"))?;
        fold(&mut r, &sent, chunk, &texts);
    }
    Ok(r)
}

/// One judged chunk folded into the report: its counts added, its
/// groups labelled.
fn fold(r: &mut Report, sent: &[&Group], j: Judged, texts: &Texts) {
    for (k, n) in COUNTS.iter().zip(j.counts) {
        *r.counts.get_mut(k).expect("a count key") += n;
    }
    for (g, (s, holes)) in sent.iter().zip(j.groups) {
        r.groups.push(label(r.groups.len(), g, &s, &holes, texts));
    }
}

/// The references to each file: the graph's edges counted by the file
/// node they land on (the same graph `ce deadcode` judges).
fn file_indegree(
    root: &Path,
    idx: &dedup::index::Index,
    db_path: &Path,
) -> Result<BTreeMap<String, u64>> {
    let w = wire_of(root, idx, db_path, Advisory::No)?;
    let mut out = BTreeMap::new();
    for [_, to, _, _] in &w.edges {
        let node = &w.nodes[*to as usize];
        if node.unit.is_empty() {
            *out.entry(node.path.clone()).or_insert(0) += 1;
        }
    }
    Ok(out)
}

/// One judged group labelled: its members, and its parameters in the
/// order the core's holes first reach them, each with every member's
/// text at the parameter's first hole.
fn label(k: usize, g: &Group, s: &Suggestion, holes: &[HoleRow], texts: &Texts) -> GroupFace {
    let mut first: Vec<(usize, usize)> = Vec::new();
    for h in holes {
        if !first.iter().any(|&(p, _)| p == h.param) {
            first.push((h.param, h.hole));
        }
    }
    let exact = g.family == FAMILY_EXACT;
    let value = |h: &HoleRow| {
        let m = &g.members[h.m];
        ValueFace {
            member: h.m,
            text: span_text(&texts[&m.path], &m.tree, h.post, h.post_end),
        }
    };
    GroupFace {
        group: k,
        family: if exact { "t1t2" } else { "t3" },
        fragment: g.fragment,
        members: g
            .members
            .iter()
            .map(|m| MemberFace {
                path: m.path.clone(),
                unit: m.unit.clone(),
                lines: [m.lines.0, m.lines.1],
                run: [m.run.0, m.run.1],
            })
            .collect(),
        params: s.params,
        kept: s.kept,
        savings: s.savings,
        feasible: s.feasible,
        reason: REASONS[usize::from(s.reason)],
        holes: first
            .into_iter()
            .map(|(param, hole)| ParamFace {
                param,
                values: holes.iter().filter(|h| h.hole == hole).map(value).collect(),
            })
            .collect(),
    }
}

/// Every member file's text, read once.
type Texts = BTreeMap<String, String>;

fn texts_of(root: &Path, groups: &[Group]) -> Result<Texts> {
    let mut out = Texts::new();
    for m in groups.iter().flat_map(|g| &g.members) {
        if !out.contains_key(&m.path) {
            out.insert(m.path.clone(), dedup::walked_text(root, &m.path)?.0);
        }
    }
    Ok(out)
}

/// The source text from node `post`'s start to node `post_end`'s end
/// (−1: nothing on this side).
fn span_text(
    text: &str,
    tree: &crate::dedup::t3::tree::UnitTree,
    post: i64,
    post_end: i64,
) -> String {
    let at = |p: i64| usize::try_from(p).ok().and_then(|i| tree.spans.get(i));
    at(post)
        .zip(at(post_end))
        .and_then(|(&(s, _), &(_, e))| text.get(s..e))
        .unwrap_or("")
        .to_string()
}

/// The document: the schema id, then the report's own fields.
pub fn report_json(r: &Report) -> Value {
    #[derive(Serialize)]
    struct Document<'a> {
        schema: &'static str,
        #[serde(flatten)]
        report: &'a Report,
    }
    let doc = Document {
        schema: SCHEMA_ID,
        report: r,
    };
    serde_json::to_value(doc).expect("a merge report is plain data")
}

#[cfg(test)]
#[path = "../../tests/unit/merge/face.rs"]
mod tests;
