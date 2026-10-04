//! What the edge passes do when the core cannot answer resolve/1 (plan
//! v2.33 wave W2a, no-core rule): the Python, Lua, Go and C / C++
//! sites of the batch are stored unresolved (no edge rows), every file
//! holding one is booked in the debt ledger (`resolve_pending`) so the
//! next run with a core re-resolves exactly those, and the meta row
//! `resolve_degraded` names the state until a pass settles clean. The
//! index refresh itself goes on — the clone fingerprints, the hooks'
//! reads, never wait on the graph — while every face that reads edges
//! refuses by name (`refuse_if_owed`) instead of judging a graph with
//! four languages' imports missing.

use super::store::{CachedSite, EdgeRow};
use anyhow::{Result, bail};
use rusqlite::Transaction;
use std::collections::BTreeSet;

/// One batch's answer: each site's edge rows, and — when the core could
/// not answer — why, with the files whose sites wait for it.
pub struct Resolved {
    pub rows: Vec<Vec<EdgeRow>>,
    pub owed: Option<Owed>,
}

pub struct Owed {
    pub reason: String,
    pub files: BTreeSet<String>,
}

/// A per-site resolver as the batch callback the two edge passes take
/// (the store's own tests drive the passes with one).
pub fn each(
    mut one: impl FnMut(&CachedSite) -> Vec<EdgeRow>,
) -> impl FnMut(&[CachedSite]) -> Result<Resolved> {
    move |sites| {
        Ok(Resolved {
            rows: sites.iter().map(&mut one).collect(),
            owed: None,
        })
    }
}

/// Settle the debt ledger after a pass resolved every owed site: clear
/// it, then book what this pass could not answer, and keep the meta
/// row in step — set while anything is owed to an absent core, gone
/// once a pass settles clean.
pub fn settle(tx: &Transaction<'_>, owed: Option<Owed>) -> Result<()> {
    tx.execute("DELETE FROM resolve_pending", [])?;
    let Some(owed) = owed else {
        tx.execute("DELETE FROM meta WHERE k = 'resolve_degraded'", [])?;
        return Ok(());
    };
    let mut book = tx.prepare("INSERT OR IGNORE INTO resolve_pending (path) VALUES (?1)")?;
    for f in &owed.files {
        book.execute((f,))?;
    }
    tx.execute(
        "INSERT INTO meta (k, v) VALUES ('resolve_degraded', ?1)
         ON CONFLICT(k) DO UPDATE SET v = ?1",
        (owed.files.len() as i64,),
    )?;
    *LAST.lock().unwrap_or_else(|p| p.into_inner()) = Some(owed.reason);
    Ok(())
}

/// The last reason this process stored sites unresolved for.
static LAST: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// A face that reads edges refuses while sites wait on an absent core.
pub fn refuse_if_owed(conn: &rusqlite::Connection) -> Result<()> {
    let owed: Option<i64> = conn
        .query_row("SELECT v FROM meta WHERE k = 'resolve_degraded'", [], |r| {
            r.get(0)
        })
        .map(Some)
        .or_else(crate::dedup::schema::ignore_no_rows)?;
    let Some(files) = owed else { return Ok(()) };
    let why = LAST
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
        .unwrap_or_else(|| "an earlier run had no core that answers resolve/1".into());
    bail!(
        "resolve_unavailable: the import sites of {files} file(s) (Python, Lua, Go, C / C++) wait for a core that answers resolve/1 — {why}; re-run with the core named (--core, CE_CORE_BIN)"
    )
}

#[cfg(test)]
#[path = "../../tests/unit/graph/owed.rs"]
mod tests;
