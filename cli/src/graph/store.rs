//! Graph persistence in the shared `.ce/index.db` (schema v4, design
//! brief §3): the symbols/sites/edges tables and the two-phase
//! invalidation contract. Phase 1 (symbols + sites) is a pure
//! function of one file's bytes and runs inside refresh_file's
//! existing transaction; phase 2 (edges) depends on the WHOLE file
//! set plus the resolver config files, so it is gated by resolve_key
//! — pretending edges were per-file invalidatable would be the bug.
//!
//! Honest limits (design §3, stated not painted over): graph rows and
//! fingerprint rows share one transaction per file, but there is no
//! single atomic whole-tree refresh — each side self-checks via its
//! own gate. A content refresh CASCADE-drops that file's old sites'
//! edges while resolve_key stands still; re-resolving just that file
//! is resolve_refreshed (phase 1.5). Every edge row flows through
//! the one insert throat below, fed by the phase-2 sweep or the
//! phase-1.5 refresh — both driven by the wire.rs ladder bridge.

use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use rusqlite::{Connection, Transaction, TransactionBehavior};
use std::collections::BTreeSet;
use std::path::Path;

/// Bump when site extraction or ladder semantics change: it sits in
/// the meta cache key, so stale graph rows are wiped (RG3 standing
/// cost: a detector change also re-freezes the slice + voids the
/// audited sample).
pub const GRAPH_REV: i64 = 1;

/// CREATE-only DDL (design §3 verbatim); the DROP half belongs to the
/// wipe lifecycle in dedup/schema.rs. `dst_path` is TEXT, not an FK:
/// sites may point at absent or excluded files, and materializing
/// phantom `files` rows to satisfy an FK would corrupt dedup.
pub const GRAPH_SCHEMA: &str = "
CREATE TABLE symbols (id INTEGER PRIMARY KEY,
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  key TEXT NOT NULL, start_line INTEGER NOT NULL, end_line INTEGER NOT NULL,
  flags INTEGER NOT NULL);
CREATE TABLE sites (id INTEGER PRIMARY KEY,
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  kind INTEGER NOT NULL, line INTEGER NOT NULL, spec TEXT NOT NULL, owner TEXT);
CREATE TABLE edges (site_id INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  dst_path TEXT NOT NULL, dst_unit TEXT NOT NULL,
  kind INTEGER NOT NULL, rung INTEGER NOT NULL, granularity INTEGER NOT NULL);
CREATE INDEX idx_sym_file ON symbols(file_id, key);
CREATE INDEX idx_site_file ON sites(file_id);
CREATE INDEX idx_edge_dst ON edges(dst_path);
";

/// Frozen site-kind storage codes: label -> row position. Appending
/// is cheap; renaming or reordering is a GRAPH_REV bump because
/// stored kinds are positions.
const KINDS: &[&str] = &[
    "import",
    "import_from",
    "export_from",
    "use",
    "mod_decl",
    "link",
    "image",
    "ref_link",
    "ref_def",
    "url",
];

fn kind_code(label: &str) -> Result<i64> {
    KINDS
        .iter()
        .position(|k| *k == label)
        .map(|i| i as i64)
        .with_context(|| {
            format!("site kind {label:?} not in store::KINDS — add it and bump GRAPH_REV")
        })
}

pub(crate) fn kind_label(code: i64) -> Option<&'static str> {
    usize::try_from(code)
        .ok()
        .and_then(|i| KINDS.get(i).copied())
}

/// Resolver-relevant config files (design §4 ladder inputs). The
/// tsconfig arm is a basename pattern because `extends` targets
/// conventionally read tsconfig.<flavor>.json and participate in
/// resolution — leaving them out of the key would serve stale edges
/// (2f refinement); an extends target under an arbitrary name stays
/// a documented boundary.
const CONFIG_NAMES: &[&str] = &["Cargo.toml", "go.mod", "package.json", "pyproject.toml"];

pub fn is_resolver_config(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
        CONFIG_NAMES.contains(&n) || (n.starts_with("tsconfig") && n.ends_with(".json"))
    })
}

/// Phase-2 cache key: fnv1a over the sorted in-scope paths plus each
/// resolver config's (path, content hash) — design §3. Adding or
/// removing a file, or touching a config, shifts the key; content
/// edits to ordinary files do not.
pub fn resolve_key(live: &BTreeSet<String>, configs: &[(String, u64)]) -> i64 {
    let mut buf = Vec::new();
    for path in live {
        buf.extend_from_slice(path.as_bytes());
        buf.push(b'\n');
    }
    for (path, hash) in configs {
        buf.extend_from_slice(path.as_bytes());
        buf.push(0);
        buf.extend_from_slice(&hash.to_le_bytes());
    }
    crate::dedup::tokens::fnv1a(&buf) as i64
}

/// Phase 1: replace one file's symbol + site rows (stale edges go
/// with the sites via the FK cascade). Runs inside the caller's
/// transaction — the content-hash gate that protects fingerprints
/// protects these rows too.
pub fn refresh_graph(tx: &Transaction<'_>, file_id: i64, text: &str, lang: Lang) -> Result<()> {
    tx.execute("DELETE FROM symbols WHERE file_id = ?1", (file_id,))?;
    tx.execute("DELETE FROM sites WHERE file_id = ?1", (file_id,))?;
    let (found, segments) = crate::graph::sites::detect_with_units(text, lang);
    // flags stay 0 until the wire build (2g) computes the bit set —
    // storing a guess would be inventing entry-point facts
    let mut sym = tx.prepare(
        "INSERT INTO symbols (file_id, key, start_line, end_line, flags) VALUES (?1, ?2, ?3, ?4, 0)",
    )?;
    for u in &segments {
        sym.execute((file_id, &u.key, u.start_line as i64, u.end_line as i64))?;
    }
    let mut site = tx.prepare(
        "INSERT INTO sites (file_id, kind, line, spec, owner) VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for s in &found {
        site.execute((
            file_id,
            kind_code(s.kind)?,
            s.line as i64,
            &s.spec,
            s.owner.as_deref(),
        ))?;
    }
    Ok(())
}

/// One cached site as phase 2 sees it (no re-parse — design §3).
pub struct CachedSite {
    pub id: i64,
    pub file: String,
    pub kind: i64,
    pub line: i64,
    pub spec: String,
    pub owner: Option<String>,
}

/// One resolved edge, produced by the 2f resolver callback.
pub struct EdgeRow {
    pub dst_path: String,
    pub dst_unit: String,
    pub kind: i64,
    pub rung: i64,
    pub granularity: i64,
}

/// Phase-2 gate: a matching stored key touches nothing; otherwise one
/// transaction drops every edge and replays the resolver over the
/// cached sites. Returns whether the sweep fired. The key check
/// lives INSIDE an IMMEDIATE transaction: a deferred check-then-sweep
/// pair interleaving with a concurrent sweep is the same
/// check-then-rebuild race class the schema wipe was caught with on
/// CI (and WAL turns the late lock upgrade into busy_snapshot).
pub fn ensure_resolved(
    conn: &mut Connection,
    key: i64,
    mut resolve: impl FnMut(&CachedSite) -> Vec<EdgeRow>,
) -> Result<bool> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let stored: Option<i64> = tx
        .query_row("SELECT v FROM meta WHERE k = 'resolve_key'", [], |r| {
            r.get(0)
        })
        .map(Some)
        .or_else(crate::dedup::schema::ignore_no_rows)?;
    if stored == Some(key) {
        return Ok(false); // tx drops: rollback of zero writes
    }
    tx.execute("DELETE FROM edges", [])?;
    let sites = cached_sites(&tx)?;
    insert_edges(&tx, &sites, &mut resolve)?;
    tx.execute(
        "INSERT INTO meta (k, v) VALUES ('resolve_key', ?1)
         ON CONFLICT(k) DO UPDATE SET v = ?1",
        (key,),
    )?;
    tx.commit()?;
    Ok(true)
}

/// Phase 1.5 (module header contract): a content refresh cascade-
/// dropped that file's edges while resolve_key stood still — re-
/// resolve JUST the refreshed files' cached sites. Callers pass only
/// files NOT covered by a phase-2 sweep this run; a swept file here
/// would double-insert its edges.
pub fn resolve_refreshed(
    conn: &mut Connection,
    dirty: &BTreeSet<String>,
    mut resolve: impl FnMut(&CachedSite) -> Vec<EdgeRow>,
) -> Result<()> {
    if dirty.is_empty() {
        return Ok(());
    }
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let sites: Vec<CachedSite> = cached_sites(&tx)?
        .into_iter()
        .filter(|s| dirty.contains(&s.file))
        .collect();
    insert_edges(&tx, &sites, &mut resolve)?;
    tx.commit()?;
    Ok(())
}

/// The single edge-insert throat shared by the phase-2 sweep and the
/// phase-1.5 per-file refresh.
fn insert_edges(
    tx: &Transaction<'_>,
    sites: &[CachedSite],
    resolve: &mut impl FnMut(&CachedSite) -> Vec<EdgeRow>,
) -> Result<()> {
    let mut ins = tx.prepare(
        "INSERT INTO edges (site_id, dst_path, dst_unit, kind, rung, granularity)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;
    for s in sites {
        for e in resolve(s) {
            ins.execute((
                s.id,
                &e.dst_path,
                &e.dst_unit,
                e.kind,
                e.rung,
                e.granularity,
            ))?;
        }
    }
    Ok(())
}

/// Every cached site joined to its path, deterministically ordered.
fn cached_sites(tx: &Transaction<'_>) -> Result<Vec<CachedSite>> {
    Ok(tx
        .prepare(
            "SELECT s.id, f.path, s.kind, s.line, s.spec, s.owner
             FROM sites s JOIN files f ON f.id = s.file_id
             ORDER BY f.path, s.id",
        )?
        .query_map([], |r| {
            Ok(CachedSite {
                id: r.get(0)?,
                file: r.get(1)?,
                kind: r.get(2)?,
                line: r.get(3)?,
                spec: r.get(4)?,
                owner: r.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?)
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
