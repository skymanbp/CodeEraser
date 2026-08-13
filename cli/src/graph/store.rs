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
//! is the 2f refresh step. Until the resolver exists, every edge row
//! comes from the phase-2 callback below, which 2e callers stub empty.

use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use rusqlite::{Connection, Transaction};
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
        .with_context(|| format!("site kind {label:?} not in store::KINDS — add it and bump GRAPH_REV"))
}

/// Resolver-relevant config files (design §4 ladder inputs). Exact
/// basenames only; a tsconfig `extends` target under another name
/// joins the key as a 2f refinement.
const CONFIG_NAMES: &[&str] = &[
    "Cargo.toml",
    "go.mod",
    "package.json",
    "pyproject.toml",
    "tsconfig.json",
];

pub fn is_resolver_config(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| CONFIG_NAMES.contains(&n))
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
    let mut site = tx
        .prepare("INSERT INTO sites (file_id, kind, line, spec, owner) VALUES (?1, ?2, ?3, ?4, ?5)")?;
    for s in &found {
        site.execute((file_id, kind_code(s.kind)?, s.line as i64, &s.spec, s.owner.as_deref()))?;
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
/// cached sites. Returns whether the sweep fired.
pub fn ensure_resolved(
    conn: &mut Connection,
    key: i64,
    mut resolve: impl FnMut(&CachedSite) -> Vec<EdgeRow>,
) -> Result<bool> {
    let stored: Option<i64> = conn
        .query_row("SELECT v FROM meta WHERE k = 'resolve_key'", [], |r| r.get(0))
        .map(Some)
        .or_else(crate::dedup::schema::ignore_no_rows)?;
    if stored == Some(key) {
        return Ok(false);
    }
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM edges", [])?;
    let sites = cached_sites(&tx)?;
    let mut ins = tx.prepare(
        "INSERT INTO edges (site_id, dst_path, dst_unit, kind, rung, granularity)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;
    for s in &sites {
        for e in resolve(s) {
            ins.execute((s.id, &e.dst_path, &e.dst_unit, e.kind, e.rung, e.granularity))?;
        }
    }
    drop(ins);
    tx.execute(
        "INSERT INTO meta (k, v) VALUES ('resolve_key', ?1)
         ON CONFLICT(k) DO UPDATE SET v = ?1",
        (key,),
    )?;
    tx.commit()?;
    Ok(true)
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
mod tests {
    use super::*;
    use crate::dedup::{Params, schema};

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().expect("mem db");
        conn.pragma_update(None, "foreign_keys", "ON").expect("fk");
        schema::ensure_cache_key(&conn, Params::default()).expect("schema");
        conn
    }

    /// Phase 1 + phase 2 against the real v4 schema: rows land, the
    /// key gate skips on match and fires on change, and a phase-1
    /// re-run cascades the stale edges away with the old sites.
    #[test]
    fn two_phase_lifecycle() {
        let mut conn = mem_db();
        conn.execute(
            "INSERT INTO files (path, content_hash, token_count, has_tokens) VALUES ('a.rs', 1, 0, 1)",
            [],
        )
        .expect("file row");
        let tx = conn.transaction().expect("tx");
        refresh_graph(&tx, 1, "mod alpha;\nfn holder() {\n    use crate::x;\n}\n", Lang::Rust)
            .expect("phase 1");
        tx.commit().expect("commit");
        let mut seen = 0;
        let fired = ensure_resolved(&mut conn, 7, |s| {
            seen += 1;
            vec![EdgeRow {
                dst_path: s.file.clone(),
                dst_unit: String::new(),
                kind: s.kind,
                rung: 1,
                granularity: 0,
            }]
        })
        .expect("sweep");
        assert!(fired, "fresh key fires");
        assert_eq!(seen, 2, "both cached sites visited");
        assert_eq!(edge_count(&conn), 2);
        assert!(
            !ensure_resolved(&mut conn, 7, |_| Vec::new()).expect("skip"),
            "matching key must skip"
        );
        assert_eq!(edge_count(&conn), 2, "skip touches nothing");
        assert!(ensure_resolved(&mut conn, 8, |_| Vec::new()).expect("refire"));
        assert_eq!(edge_count(&conn), 0, "key change replays from zero");
        let tx = conn.transaction().expect("tx2");
        refresh_graph(&tx, 1, "use crate::y;\n", Lang::Rust).expect("phase 1 again");
        tx.commit().expect("commit2");
        let sites: i64 = conn
            .query_row("SELECT COUNT(*) FROM sites", [], |r| r.get(0))
            .expect("sites");
        assert_eq!(sites, 1, "old sites replaced, not stacked");
    }

    fn edge_count(conn: &Connection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0))
            .expect("edge count")
    }

    /// The storage codes are frozen positions; an unregistered label
    /// must fail loudly, never silently invent a code.
    #[test]
    fn kind_codes_frozen_and_loud() {
        for (i, label) in KINDS.iter().enumerate() {
            assert_eq!(kind_code(label).expect(label), i as i64);
        }
        assert!(kind_code("no_such_kind").is_err());
    }

    /// resolve_key moves on file-set and config-byte changes only.
    #[test]
    fn resolve_key_tracks_paths_and_configs() {
        let one: BTreeSet<String> = ["a.rs".to_string()].into();
        let two: BTreeSet<String> = ["a.rs".to_string(), "b.md".to_string()].into();
        let base = resolve_key(&one, &[]);
        assert_eq!(base, resolve_key(&one, &[]), "deterministic");
        assert_ne!(base, resolve_key(&two, &[]), "file set participates");
        assert_ne!(
            base,
            resolve_key(&one, &[("Cargo.toml".to_string(), 5)]),
            "config hash participates"
        );
    }
}
