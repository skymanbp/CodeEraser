//! The graph reply read at the boundary: the dead and reported rows
//! (moved here from deadcode.rs unchanged) and, since the dead-file
//! amendment (plan v2.35, phase 5A), the core's grouping of the dead
//! rows — `roots` [node, state], `under` [dependent, root] and the
//! `underDropped` marker; since 5B the gate's answer (gate.rs). Every
//! definition is the core's (CE.Graph.Roots, CE.Graph.Point): this side
//! derives no root, no pair and no state, it holds the reply's shape to
//! the dead table and refuses by name what does not fit — the rows are
//! what the document and the console read.

use super::super::nodes::Node;
use super::{Gate, Judged, advisory, gate};
use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;
use std::collections::BTreeSet;

/// The grouping the core answered: the roots with their state (0..3,
/// CE.Graph.Cost.rootStates; untold until a comparison point rides),
/// the pairs ascending, and whether the core dropped them over its cap.
#[derive(Debug, Default)]
pub struct Grouping {
    pub roots: Vec<[i64; 2]>,
    pub under: Vec<[i64; 2]>,
    pub dropped: bool,
}

/// The reply's rows held at the boundary. `names` is the wire's own
/// advisory table (None = the road was not asked), read by
/// advisory::consume; `sent` the gate the request carried (None = the
/// road sent none), read by gate::consume.
pub(super) fn consume(
    reply: &Value,
    nodes: &[Node],
    names: Option<&crate::mention::Unmentioned>,
    sent: Option<&Gate>,
) -> Result<Judged> {
    // The wire's degraded BIT is authoritative (the C9 read-the-
    // real-boolean discipline, contracts/VERSIONING.md); reason
    // is its text, not its signal.
    let degraded = (reply["degraded"].as_bool() == Some(true))
        .then(|| reply["reason"].as_str().unwrap_or("degraded").to_string());
    // the fail bit is the core's (2.18.0): a missing bit is skew,
    // refused by name like every other absent key
    let Some(fail) = reply["fail"].as_bool() else {
        bail!("{}", crate::corelink::skew("graph/1 `fail`"))
    };
    let dead = dead_rows(reply, nodes)?;
    // 5A's refusal order holds: the advisory, then `reported`, then the
    // grouping, and the gate last (it reads the grouping)
    let advisory = advisory::consume(reply, nodes, names)?;
    let reported = reported_rows(reply, nodes)?;
    let grouping = grouping(reply, &dead)?;
    Ok(Judged {
        advisory,
        reported,
        gated: gate::consume(reply, &grouping, sent, (degraded.is_some(), fail))?,
        grouping,
        dead,
        kept: reply["counts"]["kept"].as_i64(),
        degraded,
        fail,
    })
}

/// One core verdict row's node, the verdict inside the four codes
/// (CE.Graph.Dead) — shared by the dead and reported tables. A verdict
/// past them is a wire-version skew, not a panic.
fn verdict_node(nodes: &[Node], idx: i64, verdict: i64) -> Result<&Node> {
    let node = usize::try_from(idx)
        .ok()
        .and_then(|i| nodes.get(i))
        .context("index out of range")?;
    ensure!((1..=4).contains(&verdict), "verdict {verdict} out of range");
    Ok(node)
}

/// The failing table: three columns, the ledger's trust beside each
/// verdict (2.32.0; request_body always sends `unres`, so the core
/// always answers the third) — every row fenced to the four codes,
/// file granularity and 0..2 trust.
fn dead_rows(reply: &Value, nodes: &[Node]) -> Result<Vec<Vec<i64>>> {
    let rows: Vec<Vec<i64>> = serde_json::from_value(reply["dead"].clone()).context("dead rows")?;
    for row in &rows {
        let [idx, verdict, conf] = row[..] else {
            bail!("dead row of arity {} — wire skew", row.len());
        };
        let node = verdict_node(nodes, idx, verdict)?;
        // The RG9 split is the CORE's since 2.18.0; here it survives
        // as a boundary contract: this list is what `--check` fails on
        // and what erase builds its advisory dead-file rows from, so an
        // aggregate in it is wire skew — a directory is never a verdict.
        ensure!(
            node.kind == super::super::wire::GRAN_FILE,
            "core dead row {idx} is not file-granularity — wire skew"
        );
        ensure!(
            (0..=2).contains(&conf),
            "confidence {conf} outside 0..2 — wire skew"
        );
    }
    Ok(rows)
}

/// The informational table: a mandatory key, its absence skew.
fn reported_rows(reply: &Value, nodes: &[Node]) -> Result<Vec<[i64; 2]>> {
    let rows: Vec<[i64; 2]> = serde_json::from_value(reply["reported"].clone())
        .with_context(|| crate::corelink::skew("graph/1 `reported`"))?;
    for &[idx, verdict] in &rows {
        verdict_node(nodes, idx, verdict)?;
    }
    Ok(rows)
}

/// The grouping held to the dead rows: both keys present (a reply
/// without them is a pre-amendment core), each table strictly
/// ascending, every root a dead row with a state code, every pair a
/// dependent (a dead row that is no root) beside a root, no pair beside
/// the dropped marker, and — unless dropped — no dependent without one.
fn grouping(reply: &Value, dead: &[Vec<i64>]) -> Result<Grouping> {
    let table = |key: &str| -> Result<Vec<[i64; 2]>> {
        serde_json::from_value(reply[key].clone()).with_context(|| {
            format!("pre-amendment core: graph/1 offers no `{key}` (the dead-file grouping)")
        })
    };
    let (roots, under) = (table("roots")?, table("under")?);
    let dropped = reply["underDropped"].as_bool() == Some(true);
    let deads: BTreeSet<i64> = dead.iter().map(|r| r[0]).collect();
    ascending("roots", roots.iter().map(|r| r[0]))?;
    ascending("under", under.iter().map(|&[d, r]| (d, r)))?;
    let rooted: BTreeSet<i64> = roots.iter().map(|r| r[0]).collect();
    for &[n, state] in &roots {
        ensure!(deads.contains(&n), "graph/1 root {n} is not a dead row");
        ensure!(
            (0..=3).contains(&state),
            "graph/1 root {n}: state {state} outside 0..3"
        );
    }
    ensure!(
        !dropped || under.is_empty(),
        "graph/1 `under` rows beside `underDropped`"
    );
    for &[d, r] in &under {
        ensure!(
            !rooted.contains(&d),
            "graph/1 pair ({d}, {r}): the dependent is a root"
        );
        ensure!(
            deads.contains(&d),
            "graph/1 pair ({d}, {r}): the dependent is not a dead row"
        );
        ensure!(
            rooted.contains(&r),
            "graph/1 pair ({d}, {r}): the root is not in `roots`"
        );
    }
    let paired: BTreeSet<i64> = under.iter().map(|p| p[0]).collect();
    if let Some(n) = deads
        .iter()
        .find(|n| !dropped && !rooted.contains(n) && !paired.contains(n))
    {
        bail!("graph/1 dead row {n} is neither a root nor under one");
    }
    Ok(Grouping {
        roots,
        under,
        dropped,
    })
}

/// A table's keys strictly ascending: no row out of order, none twice.
fn ascending<T: Ord + std::fmt::Debug>(key: &str, keys: impl Iterator<Item = T>) -> Result<()> {
    let mut last: Option<T> = None;
    for (i, k) in keys.enumerate() {
        if let Some(prev) = &last {
            ensure!(
                prev < &k,
                "graph/1 `{key}` row {i} is not strictly ascending ({k:?} after {prev:?})"
            );
        }
        last = Some(k);
    }
    Ok(())
}

/// The grouping's rows on the deadcode document request (the face's
/// road, `deadcode::said`): `roots` (with their states when a gate rode,
/// gate::on_request), `under` [dependent, root] and the dropped marker —
/// optional in the document's statement, sent here always.
pub(super) fn on_request(
    req: crate::document::Request,
    j: &Judged,
    sent: Option<&Gate>,
) -> crate::document::Request {
    gate::on_request(req, &j.grouping, sent, &j.gated)
        .rows("under", &j.grouping.under)
        .fact("underDropped", i64::from(j.grouping.dropped))
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/deadcode/reply.rs"]
mod tests;
