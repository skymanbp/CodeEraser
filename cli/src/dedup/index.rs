//! SQLite inverted fingerprint index (ADR-005): `files` +
//! `fingerprints`, WAL + busy_timeout per ADR-003. Incremental
//! invalidation is content-hash gated per file: unchanged bytes touch
//! nothing; a change deletes and reinserts only that file's rows. The
//! M2 daemon becomes the sole writer; the batch CLI uses the same
//! code single-threaded.

use super::{Params, tokens, winnow};
use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::collections::BTreeSet;
use std::path::Path;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS files (
  id INTEGER PRIMARY KEY,
  path TEXT UNIQUE NOT NULL,
  content_hash INTEGER NOT NULL,
  token_count INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS fingerprints (
  hash INTEGER NOT NULL,
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  start_line INTEGER NOT NULL,
  end_line INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_fp_hash ON fingerprints(hash);
CREATE INDEX IF NOT EXISTS idx_fp_file ON fingerprints(file_id);
";

/// One fingerprint occurrence, joined back to its file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instance {
    pub hash: u64,
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct Index {
    conn: Connection,
}

impl Index {
    pub fn open(db_path: &Path) -> Result<Self> {
        if let Some(dir) = db_path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        }
        let conn = Connection::open(db_path)
            .with_context(|| format!("open index {}", db_path.display()))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Refresh one file; returns false when the stored content hash
    /// already matches (nothing touched — the incremental fast path).
    pub fn refresh_file(&mut self, rel: &str, src: &[u8], lang: Lang, p: Params) -> Result<bool> {
        let chash = tokens::fnv1a(src) as i64;
        let stored: Option<i64> = self
            .conn
            .query_row(
                "SELECT content_hash FROM files WHERE path = ?1",
                (rel,),
                |r| r.get(0),
            )
            .map(Some)
            .or_else(ignore_no_rows)?;
        if stored == Some(chash) {
            return Ok(false);
        }
        let toks = parse_tokens(src, lang)?;
        let hashes: Vec<u64> = toks.iter().map(|t| t.hash).collect();
        let fps = winnow::fingerprints(&hashes, p);
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO files (path, content_hash, token_count) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET content_hash = ?2, token_count = ?3",
            (rel, chash, toks.len() as i64),
        )?;
        let id: i64 = tx.query_row("SELECT id FROM files WHERE path = ?1", (rel,), |r| r.get(0))?;
        tx.execute("DELETE FROM fingerprints WHERE file_id = ?1", (id,))?;
        insert_fps(&tx, id, &fps, &toks, p)?;
        tx.commit()?;
        Ok(true)
    }

    /// Drop rows for files no longer on disk; returns removed count.
    pub fn remove_missing(&mut self, live: &BTreeSet<String>) -> Result<usize> {
        let paths: Vec<(i64, String)> = self
            .conn
            .prepare("SELECT id, path FROM files")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let mut removed = 0;
        for (id, path) in paths {
            if !live.contains(&path) {
                self.conn
                    .execute("DELETE FROM files WHERE id = ?1", (id,))?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    /// Every fingerprint occurrence, deterministically ordered.
    pub fn all_instances(&self) -> Result<Vec<Instance>> {
        let mut rows: Vec<Instance> = self
            .conn
            .prepare(
                "SELECT f.hash, fl.path, f.start_line, f.end_line
                 FROM fingerprints f JOIN files fl ON fl.id = f.file_id",
            )?
            .query_map([], |r| {
                Ok(Instance {
                    hash: r.get::<_, i64>(0)? as u64,
                    file: r.get(1)?,
                    start_line: r.get::<_, i64>(2)? as usize,
                    end_line: r.get::<_, i64>(3)? as usize,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        rows.sort();
        Ok(rows)
    }
}

fn ignore_no_rows(e: rusqlite::Error) -> rusqlite::Result<Option<i64>> {
    if e == rusqlite::Error::QueryReturnedNoRows {
        Ok(None)
    } else {
        Err(e)
    }
}

fn parse_tokens(src: &[u8], lang: Lang) -> Result<Vec<tokens::Token>> {
    let grammar = lang
        .grammar()
        .context("size-only language has no token stream")?;
    let sp = crate::scan::spec::spec(lang);
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&grammar).context("set_language")?;
    let tree = parser.parse(src, None).context("parse")?;
    Ok(tokens::tokenize(tree.root_node(), sp))
}

fn insert_fps(
    tx: &rusqlite::Transaction<'_>,
    file_id: i64,
    fps: &[winnow::Fingerprint],
    toks: &[tokens::Token],
    p: Params,
) -> Result<()> {
    let mut stmt = tx.prepare(
        "INSERT INTO fingerprints (hash, file_id, start_line, end_line) VALUES (?1, ?2, ?3, ?4)",
    )?;
    for f in fps {
        let start = toks[f.start].start_line;
        let end = toks[f.start + p.kgram - 1].end_line;
        stmt.execute((f.hash as i64, file_id, start as i64, end as i64))?;
    }
    Ok(())
}
