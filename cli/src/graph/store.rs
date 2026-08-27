//! Graph persistence in the shared `.ce/index.db` (one schema
//! lifecycle owned by dedup/schema.rs — v8 at the §4 R5 amendment;
//! design brief §3): the symbols/sites/edges tables and the two-phase
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
//! is resolve_refreshed (phase 1.5). The drop and its repair sit in
//! DIFFERENT transactions, so the debt between them is a ROW, not
//! process memory (`resolve_pending`, schema v13): the refresh that
//! drops commits the debt in the same transaction, and any later
//! sweep or phase-1.5 settles it — a run that dies in between (the
//! v1.2.0 release-night daemon amputation, CI 32964681934) leaves a
//! ledger entry the next run pays, never a silent permanent hole.
//! Every edge row flows through the one insert throat below, fed by
//! the phase-2 sweep or the phase-1.5 refresh — both driven by the
//! wire.rs ladder bridge.

use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use rusqlite::{Connection, Transaction, TransactionBehavior};
use std::collections::BTreeSet;

// callers keep the store:: spelling (walkidx, tests) — the key
// functions moved to keys.rs at the E01 300-line cap
pub use crate::graph::keys::{is_resolver_config, resolve_key};

/// Bump when site extraction or ladder semantics change: it sits in
/// the meta cache key, so stale graph rows are wiped (RG3 standing
/// cost: a detector change also re-freezes the slice + voids the
/// audited sample). 2 = the 2h ladder refinements (inline-module
/// self/super anchoring, R4 member-tree descent) — ladder-only, so
/// the frozen site universe stands. 3 = 3l Haskell sites: cached .hs
/// rows hold ZERO sites (the pre-3l table was empty) and a content-
/// hash gate would keep serving them; the frozen five-language docs
/// stand — their scope constant never included .hs (SCOPE_EXTS).
/// 4 = the M5-close Go arity repayment: unit KEYS embed params
/// (`name/params`), and cached symbols rows for Go methods carried
/// the receiver-collapsed arity. 5 = the `#[path]` R1 remap (design
/// §4 Rust row, R5 column: the literal answers at R1) — ladder-only,
/// so the frozen site universe stands and the audited sample keeps
/// its identity keys. 6 = the remap's inline-module habitat
/// (child_dir + enclosing mod names, rustc reference) — the depth>0
/// refusal becomes an edge, same ladder-only class as 5. 7 = the §4
/// R5 amendment (2026-08-18, user-ratified): `pub use` binds ≤1 hop
/// to the DEFINITION file with the via_reexport mark — ladder-only
/// plus one additive edges column (schema v8 wipes the db anyway).
/// 8 = the inert ref-def edge (H1 slice 16, 2.29.0): an unused
/// reference definition's in-scope target now travels as
/// EDGE_REFDEF_UNUSED and the CORE excludes it from liveness — a
/// new stored edge-kind code, so cached sweeps re-run.
/// 10 = sites persisted their candidate BINDINGS (plan v2.14) — a
/// stored fact retired again at index schema v14 together with the
/// symbol-edge module (plan v2.17 L round piece (1), user ruling:
/// delete); the schema bump wipes every database, which is why this
/// counter did not have to move a second time for the removal.
/// 9 = symbols.flags stops being reserved (plan v2.14): the column
/// now stores the declaration's own visibility bits instead of a
/// constant 0. The MEANING of a stored value changed, which is
/// exactly what this counter is for — without the bump every index
/// built before today would keep answering "private" for every
/// symbol, silently, and no gate would see it.
/// 11 = the visibility word widens and bit 0 is repaired (plan v2.17
/// L round piece (2)): bits 1 (scope-exported) and 2 (restricted) are
/// new stored facts, and bit 0 itself changes value for three shapes
/// — a TS declarator-wrapped export the old two-hop rule stopped one
/// hop short of (`export const f = () => {}`), a Haskell binding
/// below top level sharing an exported name, and a Haskell export
/// list holding a comment or an operator (differential in CHANGELOG)
/// — so every cached symbols row is re-derived. The wire masks the
/// word to bit 0 (graph/symwire.rs): the core sees the repairs alone.
/// 12 = `symbols.conv` (plan v2.17 L round piece (4)): the AST half of
/// the convention-category word (mention/conv — test cfg, FFI,
/// decorator registration, class membership, default export, ambient,
/// dead-code allowance, other cfg) stored beside the visibility word;
/// a new column and a new stored fact, so every row is re-derived.
/// 13 = the `export_star` site kind (plan v2.17 L round piece (5)): a
/// TS `export * from` / `export * as ns from` statement opens a site
/// of its own label instead of `export_from`, the fact the `mounts`
/// table's re-export bit reads (graph/mounts.rs) — a new stored kind
/// code and a relabel of cached rows, so every site is re-detected.
pub const GRAPH_REV: i64 = 13;

/// CREATE-only DDL (design §3 verbatim); the DROP half belongs to the
/// wipe lifecycle in dedup/schema.rs. `dst_path` is TEXT, not an FK:
/// sites may point at absent or excluded files, and materializing
/// phantom `files` rows to satisfy an FK would corrupt dedup.
/// `nth` (schema v5, F2): occurrence order by start_line within one
/// (file, key) — `(path, key)` alone is NOT an identity (a Rust
/// method key carries no impl qualifier: `impl A { fn add }` and
/// `impl B { fn add }` collide in one file). Known degradation,
/// stated not painted over: deleting an EARLIER same-key sibling
/// shifts the survivors' nth, which reads as one removal plus one
/// addition downstream.
pub const GRAPH_SCHEMA: &str = "
CREATE TABLE symbols (id INTEGER PRIMARY KEY,
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  key TEXT NOT NULL, nth INTEGER NOT NULL,
  start_line INTEGER NOT NULL, end_line INTEGER NOT NULL,
  flags INTEGER NOT NULL, conv INTEGER NOT NULL);
CREATE TABLE sites (id INTEGER PRIMARY KEY,
  file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  kind INTEGER NOT NULL, line INTEGER NOT NULL, spec TEXT NOT NULL, owner TEXT);
CREATE TABLE edges (site_id INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  dst_path TEXT NOT NULL, dst_unit TEXT NOT NULL,
  kind INTEGER NOT NULL, rung INTEGER NOT NULL, granularity INTEGER NOT NULL,
  via_reexport INTEGER NOT NULL DEFAULT 0);
CREATE TABLE resolve_pending (path TEXT PRIMARY KEY);
CREATE INDEX idx_sym_file ON symbols(file_id, key);
CREATE UNIQUE INDEX idx_sym_ident ON symbols(file_id, key, nth);
CREATE INDEX idx_site_file ON sites(file_id);
CREATE INDEX idx_edge_dst ON edges(dst_path);
CREATE INDEX idx_edge_site ON edges(site_id);
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
    "export_star",
];

/// The frozen code for one site kind. KINDS is the single owner of
/// these positions: the writer below looks a code up here rather than
/// spelling an integer where the table cannot see it drift, and the
/// bridge reads the inverse (`kind_label`). The one outside reader is
/// the mounts producer (graph/mounts.rs), which selects sites by kind.
pub(crate) fn kind_code(label: &str) -> Result<i64> {
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

/// Phase 1: replace one file's symbol + site rows (stale edges go
/// with the sites via the FK cascade). Runs inside the caller's
/// transaction — the content-hash gate that protects fingerprints
/// protects these rows too.
pub fn refresh_graph(tx: &Transaction<'_>, file_id: i64, text: &str, lang: Lang) -> Result<()> {
    tx.execute("DELETE FROM symbols WHERE file_id = ?1", (file_id,))?;
    tx.execute("DELETE FROM sites WHERE file_id = ?1", (file_id,))?;
    // the site delete above cascade-dropped this file's edges; their
    // repair lives in a LATER transaction (phase 1.5 / the sweep), so
    // the debt commits WITH the drop — a run that dies between the two
    // leaves this row, and the next run settles it (module header)
    tx.execute(
        "INSERT INTO resolve_pending (path)
         SELECT path FROM files WHERE id = ?1
         ON CONFLICT(path) DO NOTHING",
        (file_id,),
    )?;
    let (found, segments) = crate::graph::sites::detect_with_units(text, lang);
    // flags carry the declaration's OWN visibility word since the
    // v2.14 producer landed (fourclass::visibility; bits 0-2 since
    // GRAPH_REV 11) and conv the AST half of its convention word
    // (mention::conv, GRAPH_REV 12) — local syntactic facts, never a
    // guess. flags was reserved and zero until a producer existed,
    // which is why nothing read it before; the wire reads bit 0 of it
    // (symwire), and conv waits for the advisory table (piece (6)).
    let mut sym = tx.prepare(
        "INSERT INTO symbols (file_id, key, nth, start_line, end_line, flags, conv)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    // nth comes from the ONE assignment throat (units::with_nth) the
    // unitsig cache also calls; the UNIQUE(file_id, key, nth) index
    // makes any future divergence loud, not silent
    for (u, nth) in crate::fourclass::units::with_nth(&segments) {
        sym.execute((
            file_id,
            &u.key,
            nth,
            u.start_line as i64,
            u.end_line as i64,
            u.vis,
            u.conv,
        ))?;
    }
    write_sites(tx, file_id, &found)
}

/// Phase 1's site half: one row per site. Split from refresh_graph at
/// the E01 fn-length line.
fn write_sites(
    tx: &Transaction<'_>,
    file_id: i64,
    found: &[crate::graph::sites::RawSite],
) -> Result<()> {
    let mut site = tx.prepare(
        "INSERT INTO sites (file_id, kind, line, spec, owner) VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for s in found {
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
    /// 1 = the answer crossed a terminal file's re-export surface
    /// (§4 R5 amendment 2026-08-18) — the via_reexport mark.
    pub via_reexport: i64,
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
    // the sweep just replayed EVERY file's edges, so it retires the
    // whole debt ledger in the same commit
    tx.execute("DELETE FROM resolve_pending", [])?;
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
/// resolve the refreshed files' cached sites, UNIONED with the
/// persisted debt ledger so an amputated earlier run's drop is paid
/// here even when this run's own walk found nothing dirty. IDEMPOTENT
/// (delete-then-insert in one lock): a CONCURRENT writer's phase-2
/// sweep may have already attached edges to these very site rows and
/// set resolve_key so our own sweep skipped — the old "callers pass
/// only files no sweep covered" rule was per-process knowledge and
/// could not see that interleaving (nightly run 31991997431 caught
/// the duplicated edge; plan v1.7 requires every write path
/// idempotent). Skipping already-swept files remains a caller-side
/// work saving, not a correctness obligation.
pub fn resolve_refreshed(
    conn: &mut Connection,
    dirty: &BTreeSet<String>,
    mut resolve: impl FnMut(&CachedSite) -> Vec<EdgeRow>,
) -> Result<()> {
    if dirty.is_empty() && !debt_standing(conn)? {
        return Ok(()); // the every-warm-run fast path stays a read
    }
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    // the ledger re-read INSIDE the lock is authoritative; the peek
    // above only spares the lock when there is provably nothing owed
    let mut owed: BTreeSet<String> = tx
        .prepare("SELECT path FROM resolve_pending")?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    owed.extend(dirty.iter().cloned());
    let sites: Vec<CachedSite> = cached_sites(&tx)?
        .into_iter()
        .filter(|s| owed.contains(&s.file))
        .collect();
    let mut del = tx.prepare("DELETE FROM edges WHERE site_id = ?1")?;
    for s in &sites {
        del.execute((s.id,))?;
    }
    drop(del);
    insert_edges(&tx, &sites, &mut resolve)?;
    // every visible ledger row is in `owed` and was just resolved (a
    // row whose file vanished resolves to nothing) — settled entire
    tx.execute("DELETE FROM resolve_pending", [])?;
    tx.commit()?;
    Ok(())
}

/// Whether any committed refresh still owes a re-resolve — peeked
/// without the write lock so the no-debt path never contends.
fn debt_standing(conn: &Connection) -> Result<bool> {
    Ok(
        conn.query_row("SELECT EXISTS(SELECT 1 FROM resolve_pending)", [], |r| {
            r.get(0)
        })?,
    )
}

/// The single edge-insert throat shared by the phase-2 sweep and the
/// phase-1.5 per-file refresh.
fn insert_edges(
    tx: &Transaction<'_>,
    sites: &[CachedSite],
    resolve: &mut impl FnMut(&CachedSite) -> Vec<EdgeRow>,
) -> Result<()> {
    let mut ins = tx.prepare(
        "INSERT INTO edges (site_id, dst_path, dst_unit, kind, rung, granularity, via_reexport)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
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
                e.via_reexport,
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
