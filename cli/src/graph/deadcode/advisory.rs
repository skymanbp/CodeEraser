//! The advisory half of the deadcode road (graph/1 6.2.0, plan v2.17
//! L round piece (6)): the two request tables read off the refreshed
//! index under `Advisory::Yes`, and the core's `exportUnmentioned`
//! rows put back beside the names the wire carried with its request —
//! one row per name, the code still the core's integer; the document
//! names the code and its reading (CE.Graph.Document, plan v2.32 step
//! 4). Its own state on the judgment — `None` when the road was not
//! asked (`Advisory::No`), `Dropped` when the core said so, `Rows`
//! otherwise — so "not asked" and "asked and clean" never share a
//! shape (W2-F4). A row here is an advisory, never a verdict: nothing
//! below touches `dead`, `fail` or `degraded`.

use super::super::mounts;
use super::super::nodes::Node;
use crate::dedup::index::Index;
use crate::mention::{self, Unmentioned};
use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// One advisory row per name: the declaring node, the declaration,
/// its line and the core's code (CE.Graph.Advisory.code).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    pub node: i64,
    pub symbol: String,
    pub line: i64,
    pub code: i64,
}

/// The states the road can end in once asked. `cut` is the
/// producer's own fact (mention/candidates.rs): the rows are the
/// judged prefix of a larger candidate set, and every face says so.
#[derive(Debug)]
pub enum Advised {
    Rows {
        rows: Vec<Named>,
        cut: bool,
    },
    /// The core judged the graph and dropped the table (soft cap).
    Dropped,
}

/// The two tables under `Advisory::Yes`: the mention pass first (its
/// own walk of the tree; the judged files are read a second time, a
/// measured cost), then the candidates and the mounts off the same
/// index. Every node gets a mounts row (§4's coverage contract is the
/// builder's); every unmentioned key names at least one declaration.
pub(super) fn tables(
    root: &Path,
    idx: &Index,
    nodes: &[Node],
    ids: &BTreeMap<(&str, &str), usize>,
) -> Result<(Unmentioned, BTreeMap<i64, [i64; 3]>)> {
    mention::refresh(root, idx)?;
    let names = mention::candidates::unmentioned(root, idx, ids)?;
    // an empty entry would let `consume` render nothing for a row the
    // core did emit; the builder's single writer makes it impossible,
    // and this says so without opening a hard-failure road here (the
    // release face is consume's own check, W8-F2)
    debug_assert!(
        names.names.values().all(|v| !v.is_empty()),
        "an unmentioned key with no names"
    );
    let facts = mounts::facts(root, idx)?;
    Ok((names, mounts::mount_rows(nodes, &facts)))
}

/// Read the two reply keys. `names` is the wire's own table (None =
/// the road was not asked). A degraded reply carries no advisory keys
/// and answers an empty face — the degraded reason is the report's;
/// any OTHER reply without the key came from a core that does not
/// speak 6.2.0 (minor skew is legal on the wire, but "asked and
/// clean" must never be the reading of "never judged").
pub(super) fn consume(
    reply: &Value,
    nodes: &[Node],
    names: Option<&Unmentioned>,
) -> Result<Option<Advised>> {
    let Some(names) = names else {
        return Ok(None);
    };
    if reply.get("unmentionedDropped").is_some() {
        return Ok(Some(Advised::Dropped));
    }
    let rows: Vec<[i64; 4]> = match reply.get("exportUnmentioned") {
        Some(rows) => serde_json::from_value(rows.clone()).context("exportUnmentioned rows")?,
        None if reply.get("degraded") == Some(&Value::Bool(true)) => Vec::new(),
        None => bail!(
            "core answered the advisory tables without exportUnmentioned — a pre-6.2.0 core cannot judge them"
        ),
    };
    let mut out = Vec::new();
    for row in rows {
        out.extend(named(row, nodes, names)?);
    }
    Ok(Some(Advised::Rows {
        rows: out,
        cut: names.cut,
    }))
}

/// One core row put back beside its names. K38's two legs land on
/// this lookup, each with its own refusal: the key-set subset of
/// 封版后勘误 ⑨ (a core row whose `(node, vis, conv)` the wire never
/// offered) and W8-F2's value side (an offered key with no names — the
/// one silent way to render nothing for a row the core did emit).
fn named(
    [node, vis, conv, code]: [i64; 4],
    nodes: &[Node],
    names: &Unmentioned,
) -> Result<Vec<Named>> {
    let entries = names.names.get(&[node, vis, conv]).with_context(|| {
        format!("core advisory row [{node},{vis},{conv}] is outside the offered table — wire skew")
    })?;
    ensure!(
        !entries.is_empty(),
        "core advisory row [{node},{vis},{conv}] names no local candidate — wire skew"
    );
    ensure!(
        usize::try_from(node).is_ok_and(|i| i < nodes.len()),
        "advisory node out of range"
    );
    Ok(entries
        .iter()
        .map(|n| Named {
            node,
            symbol: n.symbol.clone(),
            line: n.line,
            code,
        })
        .collect())
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/deadcode/advisory.rs"]
mod tests;
