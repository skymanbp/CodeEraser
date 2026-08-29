//! The analyze result cache (K step 10; the PERF ledger's
//! "真提速=结果缓存+失效" debt, honoured at the layer every caller
//! shares). Warm `analyze` on this repository measured ~555 ms, of
//! which the stream reload + matching phases account for ~310 ms
//! while re-deriving byte-identical blocks whenever no indexed
//! content moved. This leaf memoizes ONE `Blocks` result inside the
//! index database, keyed by the files-table content digest and the
//! effective report filter — so the Stop audit, `ce check`'s score
//! leg, the daemon's dedup arm, the GUI/MCP faces and the CLI all
//! hit the same slot through the one `analyze` throat. Params and
//! every algorithm revision already key the WHOLE database
//! (schema::meta_entries): a rev bump wipes this table with the
//! rest, so the slot never spells them. Single slot by design —
//! every production caller runs the default filter; a calibration
//! run with custom knobs simply overwrites and the next default run
//! recomputes once.

use super::pairs::{Blocks, Filter};
use super::tokens;
use anyhow::Result;
use rusqlite::Connection;

/// Aggregate content digest over the refreshed files table, ordered
/// by path: any addition, removal, content change or owner flip
/// moves it (a file that turns foreign leaves every pair it was in,
/// under unchanged bytes). Each row is chained through fnv1a with the
/// running accumulator, so the fold is order-sensitive without
/// duplicating the hash body. Sub-ms at hundreds of rows — the miss
/// path pays noise.
pub(super) fn digest(conn: &Connection) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT path, content_hash, owner FROM files ORDER BY path")?;
    let mut rows = stmt.query([])?;
    let mut acc = tokens::fnv1a(b"rescache/2");
    while let Some(row) = rows.next()? {
        let path: String = row.get(0)?;
        let hash: i64 = row.get(1)?;
        let foreign: i64 = row.get(2)?;
        let mut buf = path.into_bytes();
        buf.extend_from_slice(&hash.to_le_bytes());
        buf.push(foreign as u8);
        buf.extend_from_slice(&acc.to_le_bytes());
        acc = tokens::fnv1a(&buf);
    }
    Ok(acc as i64)
}

/// The cached blocks for exactly this (digest, filter), or None. A
/// missing slot, a key mismatch and an unparseable payload all
/// answer None alike: the caller RECOMPUTES, so this is a cache
/// miss, not a silent degradation — A9f forbids losing an answer,
/// and re-deriving one loses nothing.
pub(super) fn load(conn: &Connection, digest: i64, f: Filter) -> Result<Option<Blocks>> {
    let row = conn.query_row(
        "SELECT blocks FROM result_cache
         WHERE k = 1 AND digest = ?1 AND min_tokens = ?2 AND min_distinct = ?3",
        (digest, f.min_tokens as i64, f.min_distinct as i64),
        |r| r.get::<_, String>(0),
    );
    match row {
        Ok(json) => Ok(serde_json::from_str(&json).ok()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// One-slot REPLACE. The caller re-reads the digest AFTER the stream
/// reload (analyze's D1 re-feed can move file rows mid-run), so the
/// slot always describes the index state its blocks came from.
pub(super) fn store(conn: &Connection, digest: i64, f: Filter, found: &Blocks) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO result_cache (k, digest, min_tokens, min_distinct, blocks)
         VALUES (1, ?1, ?2, ?3, ?4)",
        (
            digest,
            f.min_tokens as i64,
            f.min_distinct as i64,
            serde_json::to_string(found)?,
        ),
    )?;
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/dedup/rescache.rs"]
mod tests;
