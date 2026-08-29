//! The advisory half of the deadcode road (graph/1 6.2.0, plan v2.17
//! L round piece (6)): the two request tables read off the refreshed
//! index under `Advisory::Yes`, and the core's `exportUnmentioned`
//! rows named back through the table the wire carried beside its
//! request. Its own face on `Report` — `None` when the road was not
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

/// The core's code vocabulary, by position (CE.Graph.Advisory.code).
pub const ADVISORY_NAMES: [&str; 4] = [
    "public_unmentioned",
    "private_unmentioned",
    "restricted_unmentioned",
    "reexported_unmentioned",
];

/// One rendered advisory row: the declaring file, the declaration,
/// its line, the core's code by name, and the reading of that code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvisoryRow {
    pub name: String,
    pub symbol: String,
    pub line: i64,
    pub code: &'static str,
    pub why: &'static str,
}

/// The states the road can end in once asked. `cut` is the
/// producer's own fact (mention/candidates.rs): the rows are the
/// judged prefix of a larger candidate set, and every face says so.
#[derive(Debug)]
pub enum UnmentionedFace {
    Rows {
        rows: Vec<AdvisoryRow>,
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
) -> Result<Option<UnmentionedFace>> {
    let Some(names) = names else {
        return Ok(None);
    };
    if reply.get("unmentionedDropped").is_some() {
        return Ok(Some(UnmentionedFace::Dropped));
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
    Ok(Some(UnmentionedFace::Rows {
        rows: out,
        cut: names.cut,
    }))
}

/// One core row named back. K38's two legs land on this lookup, each
/// with its own refusal: the key-set subset of 封版后勘误 ⑨ (a core row
/// whose `(node, vis, conv)` the wire never offered) and W8-F2's value
/// side (an offered key with no names — the one silent way to render
/// nothing for a row the core did emit).
fn named(
    [node, vis, conv, code]: [i64; 4],
    nodes: &[Node],
    names: &Unmentioned,
) -> Result<Vec<AdvisoryRow>> {
    let entries = names.names.get(&[node, vis, conv]).with_context(|| {
        format!("core advisory row [{node},{vis},{conv}] is outside the offered table — wire skew")
    })?;
    ensure!(
        !entries.is_empty(),
        "core advisory row [{node},{vis},{conv}] names no local candidate — wire skew"
    );
    let path = usize::try_from(node)
        .ok()
        .and_then(|i| nodes.get(i))
        .context("advisory node out of range")?;
    let code_name = usize::try_from(code)
        .ok()
        .and_then(|c| ADVISORY_NAMES.get(c))
        .context("advisory code out of range — wire skew")?;
    Ok(entries
        .iter()
        .map(|n| AdvisoryRow {
            name: path.path.clone(),
            symbol: n.symbol.clone(),
            line: n.line,
            code: code_name,
            why: why_of(code),
        })
        .collect())
}

/// The reading of each code — how far the declaration's own package
/// lets it out, for a name nothing outside its file spells.
fn why_of(code: i64) -> &'static str {
    match code {
        1 => "no other file spells it; reachable only inside its own package",
        2 => "no other file spells it; visible to its crate alone",
        3 => "no other file spells it; a façade re-exports it",
        _ => "no other file spells this exported name",
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/deadcode/advisory.rs"]
mod tests;
