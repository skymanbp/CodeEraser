//! The one document of `ce merge`, the MCP tool and the GUI hub (plan
//! v2.31 step 7, design booklet §6.4): every group merge/1 judged —
//! its family, whether its trees are fragments, its members, the
//! parameters a merged function would take, the member kept, the
//! lines saved, whether the merge is feasible and why not — each
//! parameter with every member's text at it, read back through the
//! tree's spans; the groups not sent, counted by why; the counts,
//! beside them `merged_duplicates` (the groups dropped as a second
//! suggestion for a member set, groups.rs); and, when the core could
//! not judge — none started, none offering the family, none answering
//! — the named reason. A core that answers a chunk degraded is a
//! cap-mirror drift and an error, never a document (step-7 ruling 4).
//! The core lays the document out (document/1, CE.Merge.Document)
//! over every chunk's answer joined here, and spells in the paths,
//! unit names and member texts this side sends (crate::document; a
//! hole's text is this side's cut of the tree's span, sent one per
//! hole row). Advisory: no gate reads it.

use super::groups::{self, FAMILY_EXACT, Group, Unsendable};
use super::wire::{self, HoleRow, Judged, Suggestion};
use crate::dedup::{self, t3};
use crate::document::{self, Answer, Held, Request, Why};
use crate::graph::deadcode::{Advisory, wire_of};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Reason names by code (CE.Merge.Cost): feasible, a position no
/// parameter can stand for (a statement's or any other that is not an
/// expression or a declared name — hence `position`, not `statement`),
/// a type position, a gap across statements, more parameters than the
/// ceiling, no line saved. The document's names are the core's; the
/// review instrument reads this copy.
pub const REASONS: [&str; 6] = [
    "ok",
    "position",
    "type",
    "spans_statements",
    "too_many_params",
    "no_savings",
];

/// The document request's tables and facts.
const TABLES: [&str; 3] = ["groups", "members", "holes"];
const FACTS: [&str; 6] = [
    "nodes",
    "merged_duplicates",
    "not_isomorphic",
    "no_slot_table",
    "unbuilt",
    "over_cap",
];

/// The whole leg: measured, gathered, judged chunk by chunk, laid out.
/// `only` is the console's `--group` (its lines show that group alone;
/// the document is always whole).
pub fn run(root: &Path, db: Option<PathBuf>, core: &str, only: Option<usize>) -> Result<Answer> {
    let only = only.map_or(0, |g| g + 1);
    let (blocks, idx, db_path) = dedup::snapshot(root, db)?;
    let mut link = match document::open(core) {
        Ok(link) if link.has(wire::CAP) => link,
        Ok(link) => {
            let why = format!("core offers no {}", wire::CAP);
            return degraded((core, only), Ok(link), why);
        }
        Err(why) => return degraded((core, only), Err(why.clone()), why),
    };
    let judged = t3::judge_index(root, &idx, core)?;
    let pairs: Vec<(usize, usize)> = judged.clones.iter().map(|&(a, b, _)| (a, b)).collect();
    let (groups, unsendable, merged) = groups::gather(root, &blocks.groups, &judged.units, &pairs)?;
    let indeg = file_indegree(root, &idx, &db_path)?;
    let mut answers: Vec<Judged> = Vec::new();
    for range in wire::chunks(&groups) {
        let sent: Vec<&Group> = groups[range].iter().collect();
        let body = wire::body(&sent, |p| indeg.get(p).copied().unwrap_or(0));
        let reply = match wire::ask(&mut link, body) {
            Ok(reply) => reply,
            Err(why) => return degraded((core, only), Err(why.clone()), why),
        };
        answers.push(wire::consume(&reply, &sent).map_err(|e| anyhow::anyhow!("merge: {e}"))?);
    }
    let texts = texts_of(root, &groups)?;
    let req = request(&groups, answers, (unsendable, merged), &texts).fact("only", only);
    document::assemble_over(core, Ok(link), req)
}

/// The judgment did not happen: no row, every fact zero, the reason;
/// laid out over `held` when the link is still whole.
fn degraded((core, only): (&str, usize), held: Held, why: String) -> Result<Answer> {
    let mut reasons = Why::default();
    let reason = reasons.add(why);
    let req = FACTS
        .iter()
        .fold(Request::new("merge").empty(&TABLES), |q, k| q.fact(k, 0));
    document::assemble_over(
        core,
        held,
        req.range("members", 0)
            .range("why", 1)
            .fact("only", only)
            .degraded(reason)
            .text("why", reasons.list()),
    )
}

/// Every chunk's answer joined: the suggestion rows numbered across
/// chunks with the family and fragment bits, one row per member
/// numbered across groups, the hole rows, the facts this side counted,
/// and the strings: each member's path and unit, and each hole row's
/// text — the member's source from the row's first node to its last,
/// cut here off the tree's spans (`span_text`).
fn request(
    groups: &[Group],
    answers: Vec<Judged>,
    (u, merged): (Unsendable, u64),
    texts: &Texts,
) -> Request {
    let (mut rows, mut members, mut holes) = (Vec::new(), Vec::new(), Vec::new());
    let mut cut: Vec<String> = Vec::new();
    let mut nodes = 0;
    let suggested = answers.iter().flat_map(|j| &j.groups);
    for (g, (group, (s, hs))) in groups.iter().zip(suggested).enumerate() {
        rows.push(group_row(g, group, s));
        for (m, x) in group.members.iter().enumerate() {
            let (a, b, ra, rb) = (x.lines.0, x.lines.1, x.run.0, x.run.1);
            members.push([
                members.len(),
                g,
                m,
                a,
                b,
                ra,
                rb,
                usize::from(x.unit.is_some()),
            ]);
        }
        holes.extend(hs.iter().map(|h| hole_row(g, h)));
        cut.extend(hs.iter().map(|h| match group.members.get(h.m) {
            Some(m) => span_text(&texts[&m.path], &m.tree, h.post, h.post_end),
            None => String::new(),
        }));
    }
    for j in &answers {
        nodes += j.counts[2];
    }
    let all: Vec<&groups::Member> = groups.iter().flat_map(|g| &g.members).collect();
    Request::new("merge")
        .range("members", all.len())
        .range("why", 0)
        .rows("groups", rows)
        .rows("members", members)
        .rows("holes", holes)
        .fact("nodes", nodes)
        .fact("merged_duplicates", merged)
        .fact("not_isomorphic", u.not_isomorphic)
        .fact("no_slot_table", u.no_slot_table)
        .fact("unbuilt", u.unbuilt)
        .fact("over_cap", u.over_cap)
        .text_columns(("path", "unit"), all.iter().map(|m| (&m.path, &m.unit)))
        .text("holeText", cut)
        .text("why", [""; 0])
}

/// [g, params, kept, savings, feasible, reason, family, fragment].
fn group_row(g: usize, group: &Group, s: &Suggestion) -> [i64; 8] {
    let t3 = i64::from(group.family != FAMILY_EXACT);
    let (params, kept) = (s.params as i64, s.kept as i64);
    let (feasible, fragment) = (i64::from(s.feasible), i64::from(group.fragment));
    [
        g as i64,
        params,
        kept,
        s.savings,
        feasible,
        i64::from(s.reason),
        t3,
        fragment,
    ]
}

/// [g, hole, param, m, post, postEnd].
fn hole_row(g: usize, h: &HoleRow) -> [i64; 6] {
    [
        g as i64,
        h.hole as i64,
        h.param as i64,
        h.m as i64,
        h.post,
        h.post_end,
    ]
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

#[cfg(test)]
#[path = "../../tests/unit/merge/face.rs"]
mod tests;
