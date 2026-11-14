//! The dead-file gate on the graph wire (plan v2.35, the dead-file
//! amendment, phase 5B). This side forwards three facts it read — every
//! file node's entity id (`score::baseline::file_entity` of its
//! project-relative path, the id its size row has), the committed
//! baseline's `deadPoint` verbatim, and the `[deadcode] block` mask —
//! and reads the core's answer back: every unreferenced file's state,
//! the blocked files (the roots of the unreferenced files whose state the
//! mask names), whether the gate could be answered, and today's point
//! (`deadNow`). The states, the comparison and what blocks are the
//! core's (CE.Graph.Point); nothing here joins a point to a file, decides
//! a state or derives a blocked file.

use super::{GraphWire, Grouping};
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

/// What graph/1 is told: every file node's entity id (ascending by
/// node), the committed point (None = no baseline, or one written
/// before the amendment) and the block mask (bit i = BLOCKABLE[i]).
#[derive(Debug, Clone, Default)]
pub struct Gate {
    pub entities: Vec<(i64, u64)>,
    pub point: Option<Value>,
    pub block: u8,
}

/// The core's answer: every unreferenced file's state ([node, state],
/// one row per dead row in its order; the codes are
/// CE.Graph.Cost.rootStates), the blocked files (ascending nodes),
/// whether a mask rode without a point, and today's point (Some exactly
/// when the entities rode and the judgment was not degraded).
#[derive(Debug, Default)]
pub struct Gated {
    pub states: Vec<[i64; 2]>,
    pub blocked: Vec<i64>,
    pub unanswered: bool,
    pub now: Option<Value>,
}

impl Gate {
    /// The gate over a wire: one entity per file node, keyed to the
    /// project root like the size rows (score::provenance::Keys), so a
    /// scoped run names a file as the root-only baseline does.
    pub fn of(root: &Path, w: &GraphWire, point: Option<Value>, block: u8) -> Gate {
        let keys = crate::score::provenance::Keys::of(root);
        let entities = super::file_nodes(w)
            .into_iter()
            .map(|(n, path)| (n, crate::score::baseline::file_entity(&keys.key(path))))
            .collect();
        Gate {
            entities,
            point,
            block,
        }
    }

    /// The gate's keys on the graph request.
    pub(super) fn insert(&self, obj: &mut serde_json::Map<String, Value>) {
        let rows: Vec<[Value; 2]> = self
            .entities
            .iter()
            .map(|&(n, x)| [json!(n), json!(x)])
            .collect();
        obj.insert("entities".into(), json!(rows));
        if let Some(p) = &self.point {
            obj.insert("deadPoint".into(), p.clone());
        }
        obj.insert("deadBlock".into(), json!(self.block));
    }

    /// A blocked file's entity id, from the table this side sent.
    pub fn id_of(&self, node: i64) -> Option<u64> {
        self.entities
            .binary_search_by_key(&node, |&(n, _)| n)
            .ok()
            .map(|i| self.entities[i].1)
    }
}

/// The committed baseline's point as the gate reads it: the envelope
/// (`score::baseline::document`, every schema), its `deadPoint` key
/// verbatim; no file or no key = None.
pub fn committed_point(root: &Path) -> Result<Option<Value>> {
    Ok(crate::score::baseline::document(root)?.and_then(|d| d.get("deadPoint").cloned()))
}

/// The reply's gate held at the boundary: `states` and `blocked`
/// present (a reply without them is a pre-amendment core); one state row
/// per dead row, in its order, each code inside 0..3; `blocked` strictly
/// ascending, every node an unreferenced file (a dead row; since the
/// gate reads every file's state a blocked file may be a dependent);
/// nothing blocked beside an unanswered gate; the fail bit the gate's
/// answer (degraded, or a blocked file); `deadNow` present exactly when
/// the entities rode on a judged reply, two ascending id lists.
pub(super) fn consume(
    reply: &Value,
    dead: &[Vec<i64>],
    sent: Option<&Gate>,
    (degraded, fail): (bool, bool),
) -> Result<Gated> {
    let states: Vec<[i64; 2]> = serde_json::from_value(reply["states"].clone())
        .context("pre-amendment core: graph/1 offers no `states` (the dead-file gate)")?;
    let blocked: Vec<i64> = serde_json::from_value(reply["blocked"].clone())
        .context("pre-amendment core: graph/1 offers no `blocked` (the dead-file gate)")?;
    ensure!(
        states.iter().map(|s| s[0]).eq(dead.iter().map(|r| r[0])),
        "graph/1 `states` is not one row per dead row, in its order"
    );
    if let Some([n, s]) = states.iter().find(|s| !(0..=3).contains(&s[1])) {
        bail!("graph/1 file {n}: state {s} outside 0..3");
    }
    let deads: BTreeSet<i64> = dead.iter().map(|r| r[0]).collect();
    for (i, w) in blocked.windows(2).enumerate() {
        ensure!(
            w[0] < w[1],
            "graph/1 `blocked` row {} is not strictly ascending",
            i + 1
        );
    }
    if let Some(n) = blocked.iter().find(|n| !deads.contains(n)) {
        bail!("graph/1 blocked node {n} is not an unreferenced file");
    }
    let unanswered = reply["blockUnanswered"].as_bool() == Some(true);
    ensure!(
        !unanswered || blocked.is_empty(),
        "graph/1 `blocked` rows beside `blockUnanswered`"
    );
    ensure!(
        fail == (degraded || !blocked.is_empty()),
        "graph/1 `fail` {fail} is not the gate's answer (degraded {degraded}, {} blocked)",
        blocked.len()
    );
    Ok(Gated {
        now: now_of(reply, sent.is_some() && !degraded)?,
        states,
        blocked,
        unanswered,
    })
}

/// `deadNow`: required when the entities rode on a judged reply,
/// refused otherwise; its two lists strictly ascending.
fn now_of(reply: &Value, expected: bool) -> Result<Option<Value>> {
    let now = &reply["deadNow"];
    match (expected, now.is_null()) {
        (true, true) => bail!("graph/1 offers no `deadNow` for the entities it was sent"),
        (false, false) => bail!("graph/1 answered `deadNow` to a request without entities"),
        (false, true) => return Ok(None),
        (true, false) => {}
    }
    for key in ["files", "dead"] {
        let ids: Vec<u64> = serde_json::from_value(now[key].clone())
            .with_context(|| format!("graph/1 `deadNow.{key}`: not a list of ids"))?;
        ensure!(
            ids.windows(2).all(|w| w[0] < w[1]),
            "graph/1 `deadNow.{key}` is not strictly ascending"
        );
    }
    Ok(Some(now.clone()))
}

/// The gate's rows on the deadcode document request (the face's road):
/// the roots, every unreferenced file's state, the blocked files, the
/// mask, whether a point rode and whether the gate went unanswered.
/// Without a gate the grouping rides as 5A sent it.
pub(super) fn on_request(
    req: crate::document::Request,
    g: &Grouping,
    sent: Option<&Gate>,
    gated: &Gated,
) -> crate::document::Request {
    let req = req.rows("roots", g.roots.iter().map(|&r| [r]).collect::<Vec<_>>());
    let Some(gate) = sent else {
        return req;
    };
    req.rows("states", &gated.states)
        .rows(
            "blocked",
            gated.blocked.iter().map(|&n| [n]).collect::<Vec<_>>(),
        )
        .fact("blockMask", i64::from(gate.block))
        .fact("compared", i64::from(gate.point.is_some()))
        .fact("unanswered", i64::from(gated.unanswered))
}

/// The deadcode face's could-not-judge rule (release 1.10.0): its
/// document is unjudged, and the CLI exits 2, when the document's
/// `degraded` names a reason — the rule `ce flow` / `ce arch` /
/// `ce merge` read — or, under `--check`, when a declared block could not
/// be answered (`ce-baseline.json` holds no comparison point). Exit 1
/// stays the gate's no: a blocked file.
pub fn unjudged(doc: &Value, check: bool) -> bool {
    !doc["degraded"].is_null() || (check && doc["gate"]["answered"] == json!(false))
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/deadcode/gate.rs"]
mod tests;
