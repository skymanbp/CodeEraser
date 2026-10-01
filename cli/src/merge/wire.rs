//! The merge/1 leg (design booklet §3's row, §6.2; plan v2.31 step 7):
//! the request — the groups, their members, one tree per member in
//! clone/1's postorder encoding with the leaf and slot columns — laid
//! out in chunks the core's two caps admit, and the reply consumed
//! strictly. Nothing here judges: the alignment, the holes, the
//! parameters, the feasibility, the kept member and the savings are the
//! core's (CE.Merge.*). Every request is priced here within both caps,
//! so a reply the core degraded is a cap-mirror drift and an error, never
//! a document (step-7 ruling 4, the A9b posture); a reply whose tables
//! disagree with what was sent is wire skew, never a healthy answer
//! (A9f).

use super::groups::Group;
use crate::corelink::{Link, judged};
use crate::dedup::t3::tree::UnitTree;
use crate::dedup::t3::wire::dense;
use serde_json::{Value, json};
use std::ops::Range;

/// The capability the core must offer, and the request kind.
pub const CAP: &str = "merge/1";
pub const KIND: &str = "merge";
const SINCE: &str = "7.5.0";

/// Groups per request — mirror of CE.Merge.Cost.groupCap.
pub const GROUP_CAP: usize = 4096;
/// Tree nodes per request — mirror of CE.Merge.Cost.treeNodeCap.
pub const TREE_NODE_CAP: usize = 1_048_576;

/// The counts object's keys, in the wire's order.
pub const COUNTS: [&str; 6] = [
    "groups",
    "members",
    "nodes",
    "suggestions",
    "holes",
    "feasible",
];

/// The core's reason codes (CE.Merge.Cost): the last one, and the one
/// that says no line is saved (reasonNoSavings).
const REASON_CEIL: i64 = 5;
const REASON_NO_SAVINGS: i64 = 5;

/// One group's answer: [params, kept, savings, feasible, reason].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Suggestion {
    pub params: u64,
    pub kept: usize,
    pub savings: i64,
    pub feasible: bool,
    pub reason: u8,
}

/// One hole row of a group: the hole, its parameter, the member and
/// the member's first and last root of the hole — one node twice for a
/// leaf hole, a gap's forest between them, −1 −1 an empty side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoleRow {
    pub hole: usize,
    pub param: usize,
    pub m: usize,
    pub post: i64,
    pub post_end: i64,
}

/// Consecutive runs of the groups, each within both caps; a group never
/// splits (the caller has not sent one whose nodes alone pass
/// TREE_NODE_CAP).
pub fn chunks(groups: &[Group]) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let (mut start, mut nodes) = (0, 0);
    for (i, g) in groups.iter().enumerate() {
        let n = g.nodes();
        if i > start && (i - start == GROUP_CAP || nodes + n > TREE_NODE_CAP) {
            out.push(start..i);
            (start, nodes) = (i, 0);
        }
        nodes += n;
    }
    if start < groups.len() {
        out.push(start..groups.len());
    }
    out
}

/// A judged chunk: per group sent, in order, its suggestion and hole
/// rows; the counts as the core gave them.
#[derive(Debug, Default)]
pub struct Judged {
    pub groups: Vec<(Suggestion, Vec<HoleRow>)>,
    pub counts: [u64; 6],
}

/// One chunk's request body: dense labels across its trees, `unit` the
/// member's request-local number (echoed, never read), `lines` the lines
/// of the run the member sends (a trimmed fragment's own, not its clone
/// family's span), `fileIndeg` the references to the member's file.
pub fn body(groups: &[&Group], indeg: impl Fn(&str) -> u64) -> Value {
    let trees: Vec<&UnitTree> = groups
        .iter()
        .flat_map(|g| g.members.iter().map(|m| &m.tree))
        .collect();
    let rows: Vec<Value> = dense(&trees)
        .into_iter()
        .zip(&trees)
        .map(|(lab, t)| json!({"lab": lab, "lld": t.lld, "leaf": t.leaf, "slot": t.slot}))
        .collect();
    let mut members = Vec::new();
    for (g, group) in groups.iter().enumerate() {
        for (m, member) in group.members.iter().enumerate() {
            let lines = member.run.1 + 1 - member.run.0;
            let row = [g, m, members.len(), lines, indeg(&member.path) as usize];
            members.push(row);
        }
    }
    let heads: Vec<[usize; 2]> = groups
        .iter()
        .enumerate()
        .map(|(g, x)| [g, usize::from(x.family)])
        .collect();
    json!({"groups": heads, "members": members, "trees": rows})
}

/// The reply consumed against the chunk that was sent: degraded first
/// (a drift of the cap mirrors — this side priced the chunk within
/// both), then the counts and the rows.
pub fn consume(reply: &Value, sent: &[&Group]) -> Result<Judged, String> {
    crate::lockstep::refuse_degraded(reply, "merge/wire.rs vs Merge/Cost.hs")
        .map_err(|e| e.to_string())?;
    let mut counts = [0u64; 6];
    for (slot, key) in counts.iter_mut().zip(COUNTS) {
        *slot = judged::count(reply, key)? as u64;
    }
    let members: usize = sent.iter().map(|g| g.members.len()).sum();
    let nodes: usize = sent.iter().map(|g| g.nodes()).sum();
    if counts[..3] != [sent.len() as u64, members as u64, nodes as u64] {
        return Err(format!("wire skew: counts {counts:?} for what was sent"));
    }
    let mut groups = suggestions(reply, sent)?;
    let holes = holes(reply, sent, &mut groups)?;
    let feasible = groups.iter().filter(|(s, _)| s.feasible).count();
    if counts[3..] != [groups.len() as u64, holes as u64, feasible as u64] {
        return Err("wire skew: counts disagree with the rows".into());
    }
    Ok(Judged { groups, counts })
}

/// One request over a link past its handshake, behind the capability
/// gate (a core without the family is healthy and answers nothing).
pub fn ask(link: &mut Link, body: Value) -> Result<Value, String> {
    judged::ask(link, CAP, SINCE, KIND, body)
}

fn suggestions(reply: &Value, sent: &[&Group]) -> Result<Vec<(Suggestion, Vec<HoleRow>)>, String> {
    let rows: Vec<Vec<i64>> = judged::table(reply, "suggestions")?;
    if rows.len() != sent.len() {
        return Err("wire skew: one suggestion per group sent".into());
    }
    rows.iter()
        .zip(sent)
        .enumerate()
        .map(|(g, (row, group))| match row.as_slice() {
            [id, params, kept, savings, feasible, reason]
                if *id == g as i64
                    && *params >= 0
                    && (0..group.members.len() as i64).contains(kept)
                    && (0..=REASON_CEIL).contains(reason)
                    && (*feasible == 1) == (*reason == 0)
                    && (0..=1).contains(feasible)
                    && (*reason != 0 || *savings > 0)
                    && (*reason != REASON_NO_SAVINGS || *savings <= 0) =>
            {
                let s = Suggestion {
                    params: *params as u64,
                    kept: *kept as usize,
                    savings: *savings,
                    feasible: *feasible == 1,
                    reason: *reason as u8,
                };
                Ok((s, Vec::new()))
            }
            _ => Err(format!(
                "wire skew: suggestion {g} is not [g,params,kept,savings,feasible,reason]"
            )),
        })
        .collect()
}

/// The hole rows, ascending by (g, hole, m), each on a member and two
/// nodes that were sent — both −1, or post ≤ postEnd; every group's
/// parameters numbered 0.. with none skipped — the count its suggestion
/// carries.
fn holes(
    reply: &Value,
    sent: &[&Group],
    groups: &mut [(Suggestion, Vec<HoleRow>)],
) -> Result<usize, String> {
    let rows: Vec<Vec<i64>> = judged::table(reply, "holes")?;
    let mut last: Option<[i64; 3]> = None;
    for row in &rows {
        let [g, hole, param, m, post, post_end] = row.as_slice() else {
            return Err("wire skew: a hole is [g,hole,param,m,post,postEnd]".into());
        };
        let member = usize::try_from(*g)
            .ok()
            .and_then(|g| sent.get(g)?.members.get(usize::try_from(*m).ok()?));
        let nodes = member.map_or(0, |x| x.tree.lab.len() as i64);
        let fits = member.is_some()
            && [post, post_end].iter().all(|p| (-1..nodes).contains(*p))
            && (*post == -1) == (*post_end == -1)
            && post_end >= post;
        if !fits || *hole < 0 || *param < 0 || last >= Some([*g, *hole, *m]) {
            return Err(format!("wire skew: hole row {row:?}"));
        }
        last = Some([*g, *hole, *m]);
        groups[*g as usize].1.push(HoleRow {
            hole: *hole as usize,
            param: *param as usize,
            m: *m as usize,
            post: *post,
            post_end: *post_end,
        });
    }
    for (g, (s, rows)) in groups.iter().enumerate() {
        let params = rows.iter().map(|r| r.param + 1).max().unwrap_or(0);
        if params as u64 != s.params {
            return Err(format!(
                "wire skew: group {g}'s holes number {params} parameters"
            ));
        }
    }
    Ok(rows.len())
}

#[cfg(test)]
#[path = "../../tests/unit/merge/wire.rs"]
mod tests;
