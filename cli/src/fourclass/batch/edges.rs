//! The declaration verdict, consumed (plan v2.29 step 10, O48). The
//! core decided which declarations pair; this maps its integers back
//! onto unit keys and drops every edge the LINE stage already named,
//! so one relocation is reported once and the two stages never
//! disagree about the same edge. Every index and hash is checked
//! against what THIS side sent — the reply is an answer, not an
//! authority (delta.rs's stance, same reasons).
//!
//! An absent `unitEdges` key is a core older than 7.1.0 and an
//! `unitEdgesDropped` reply is a batch over the core's declaration
//! cap: both mean "no declaration edges", never an error and never a
//! blocker (A9f) — the informational stage degrades, the answer does
//! not.

use super::super::decls::{Decl, key_hash};
use super::super::model::Classification;
use super::Relocation;
use serde_json::Value;
use std::collections::BTreeMap;

/// The unit key each (pair index, hash) named on one side — the
/// lookup that turns the reply's integers back into names.
fn sent(pairs: &[Classification], added: bool) -> Result<BTreeMap<(usize, u64), &Decl>, String> {
    let mut out = BTreeMap::new();
    for (i, c) in pairs.iter().enumerate() {
        let side: &[Decl] = if added { &c.decls.1 } else { &c.decls.0 };
        for d in side {
            if out.insert((i, key_hash(d)), d).is_some() {
                return Err("unitEdges: ambiguous sent hash".into());
            }
        }
    }
    Ok(out)
}

/// Whether the LINE stage already named this (source, destination,
/// unit): either END may carry the key, exactly the way the
/// relocation register reads a row (a de-duplication rename registers
/// under the surviving name).
fn named_already(line_level: &[Relocation], from: usize, to: usize, key: &str) -> bool {
    line_level.iter().any(|r| {
        r.from_pair == from
            && r.to_pair == to
            && (r.from_unit.as_deref() == Some(key) || r.to_unit.as_deref() == Some(key))
    })
}

fn triple_of(e: &Value) -> Result<(usize, usize, u64), String> {
    let (from, to, h): (usize, usize, u64) =
        serde_json::from_value(e.clone()).map_err(|_| "unitEdges: row shape".to_string())?;
    if from == to {
        return Err("unitEdges: a pair cannot relocate into itself".into());
    }
    Ok((from, to, h))
}

fn mapped(
    triple: (usize, usize, u64),
    from_keys: &BTreeMap<(usize, u64), &Decl>,
    to_keys: &BTreeMap<(usize, u64), &Decl>,
) -> Result<Relocation, String> {
    let (from, to, h) = triple;
    let source = from_keys
        .get(&(from, h))
        .ok_or("unitEdges: unsent source")?;
    let dest = to_keys
        .get(&(to, h))
        .ok_or("unitEdges: unsent destination")?;
    if source.key != dest.key || source.kind != dest.kind {
        return Err("unitEdges: mismatched declaration identity".into());
    }
    Ok(Relocation {
        from_pair: from,
        from_unit: Some(source.key.clone()),
        to_pair: to,
        to_unit: Some(dest.key.clone()),
        lines: 0,
    })
}

/// The accepted declaration edges as relocations with `lines: 0` —
/// zero moved lines is exactly what this stage claims: an edge whose
/// evidence is the declaration, not a station.
pub(super) fn unit_edges(
    reply: &Value,
    pairs: &[Classification],
    line_level: &[Relocation],
) -> Result<Vec<Relocation>, String> {
    let Some(rows) = reply.get("unitEdges") else {
        return Ok(Vec::new()); // pre-7.1.0 core
    };
    let rows = rows
        .as_array()
        .ok_or_else(|| "unitEdges: array".to_string())?;
    if let Some(dropped) = reply.get("unitEdgesDropped") {
        if dropped != &Value::Bool(true) || !rows.is_empty() {
            return Err("unitEdgesDropped: expected true with an empty table".into());
        }
        return Ok(Vec::new());
    }
    let (from_keys, to_keys) = (sent(pairs, false)?, sent(pairs, true)?);
    let mut out: Vec<Relocation> = Vec::new();
    let mut prev: Option<(usize, usize, u64)> = None;
    for e in rows {
        let triple = triple_of(e)?;
        if prev.is_some_and(|p| p >= triple) {
            return Err("unitEdges: rows are not strictly ascending".into());
        }
        prev = Some(triple);
        let edge = mapped(triple, &from_keys, &to_keys)?;
        if !named_already(
            line_level,
            triple.0,
            triple.1,
            edge.to_unit.as_deref().unwrap(),
        ) {
            out.push(edge);
        }
    }
    Ok(out)
}

#[cfg(test)]
#[path = "../../../tests/unit/fourclass/batch/edges.rs"]
mod tests;
