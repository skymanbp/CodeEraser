//! unitsig — the per-unit structural-fact cache (schema v5, design
//! vol.1 §3): named-node count, structural shingle set, kind
//! histogram, keyed (file, key, nth) with nth from the ONE assignment
//! throat (`fourclass::units::with_nth`) the graph `symbols` rows
//! also use. Refreshed inside refresh_file's transaction — this is
//! the honest FOURTH per-file parse (design F11: the token stream
//! drops its tree, so T3 facts cost a parse; the PERF-BUDGET ledger
//! carries the number instead of the docs claiming "no re-parse").

use super::struct_fp;
use crate::fourclass::units;
use crate::scan::ast;
use crate::scan::lang::Lang;
use anyhow::Result;
use rusqlite::Transaction;
use std::collections::{BTreeMap, BTreeSet};

/// CREATE-only DDL (the DROP half lives in dedup/schema.rs — one
/// wipe lifecycle). `sig` = sorted shingle u64s LE; `hist` = sorted
/// (kind u64, count u32) pairs LE.
pub const UNITSIG_SCHEMA: &str = "
CREATE TABLE unitsig (
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  key TEXT NOT NULL, nth INTEGER NOT NULL,
  nodes INTEGER NOT NULL, sig BLOB NOT NULL, hist BLOB NOT NULL);
CREATE INDEX idx_unitsig_file ON unitsig(file_id);
";

/// Replace one file's unitsig rows. Markdown (no grammar) and parse
/// failures leave zero rows — T3 judges code units only; text
/// duplication is docdup's domain.
pub fn refresh_units(tx: &Transaction<'_>, file_id: i64, text: &str, lang: Lang) -> Result<()> {
    tx.execute("DELETE FROM unitsig WHERE file_id = ?1", (file_id,))?;
    let Some(grammar) = lang.grammar() else {
        return Ok(());
    };
    let Some(tree) = ast::parse(text, &grammar) else {
        return Ok(());
    };
    let spine = struct_fp::spine(tree.root_node());
    let segments = units::segments(text, lang);
    let mut ins = tx.prepare(
        "INSERT INTO unitsig (file_id, key, nth, nodes, sig, hist)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;
    for (u, nth) in units::with_nth(&segments) {
        let seq = struct_fp::unit_seq(&spine, u.start_line, u.end_line);
        ins.execute((
            file_id,
            &u.key,
            nth,
            seq.len() as i64,
            sig_blob(&struct_fp::shingles(&seq)),
            hist_blob(&struct_fp::histogram(&seq)),
        ))?;
    }
    Ok(())
}

fn sig_blob(set: &BTreeSet<u64>) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 * set.len());
    for v in set {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn hist_blob(hist: &BTreeMap<u64, u32>) -> Vec<u8> {
    let mut out = Vec::with_capacity(12 * hist.len());
    for (k, c) in hist {
        out.extend_from_slice(&k.to_le_bytes());
        out.extend_from_slice(&c.to_le_bytes());
    }
    out
}

/// One row of the unit universe, as `ce clone --units` lists it.
pub struct UnitRow {
    pub path: String,
    pub key: String,
    pub nth: i64,
    pub nodes: i64,
}

/// Every cached unit, deterministically ordered (the wire/freeze
/// bytes downstream must be a function of the tree — G11 posture).
pub fn unit_rows(idx: &super::index::Index) -> Result<Vec<UnitRow>> {
    Ok(crate::graph::load::rows(
        idx.raw(),
        "SELECT f.path, u.key, u.nth, u.nodes
         FROM unitsig u JOIN files f ON f.id = u.file_id
         ORDER BY f.path, u.key, u.nth",
        crate::graph::load::t4,
    )?
    .into_iter()
    .map(|(path, key, nth, nodes)| UnitRow {
        path,
        key,
        nth,
        nodes,
    })
    .collect())
}

/// Count of unitsig rows with NO matching symbols row on
/// (file, key, nth) — the two persisted views of one nth throat must
/// agree, and `ce clone --units` asserts zero instead of assuming it.
pub fn identity_orphans(idx: &super::index::Index) -> Result<i64> {
    Ok(idx.raw().query_row(
        "SELECT COUNT(*) FROM unitsig u
         WHERE NOT EXISTS (SELECT 1 FROM symbols s
           WHERE s.file_id = u.file_id AND s.key = u.key AND s.nth = u.nth)",
        [],
        |r| r.get(0),
    )?)
}
