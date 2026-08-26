//! store.rs unit battery, split out per the E01 300-line file cap
//! (the md.rs / md_tests.rs precedent): two-phase lifecycle against
//! the real v4 schema, the frozen kind-code table, and resolve_key
//! sensitivity.

use super::*;
use crate::dedup::{Params, schema};

fn mem_db() -> Connection {
    let conn = Connection::open_in_memory().expect("mem db");
    conn.pragma_update(None, "foreign_keys", "ON").expect("fk");
    schema::ensure_cache_key(&conn, Params::default()).expect("schema");
    conn
}

/// mem_db with one file row and its phase-1 rows committed — the
/// arrange both lifecycle batteries share (the ratchet's own catch
/// when the idempotency battery landed).
fn seeded(text: &str) -> Connection {
    let mut conn = mem_db();
    conn.execute(
        "INSERT INTO files (path, content_hash, token_count, has_tokens) VALUES ('a.rs', 1, 0, 1)",
        [],
    )
    .expect("file row");
    let tx = conn.transaction().expect("tx");
    refresh_graph(&tx, 1, text, Lang::Rust).expect("phase 1");
    tx.commit().expect("commit");
    conn
}

/// Phase 1 + phase 2 against the real v4 schema: rows land, the
/// key gate skips on match and fires on change, and a phase-1
/// re-run cascades the stale edges away with the old sites.
#[test]
fn two_phase_lifecycle() {
    let mut conn = seeded("mod alpha;\nfn holder() {\n    use crate::x;\n}\n");
    let mut seen = 0;
    let fired = ensure_resolved(&mut conn, 7, |s| {
        seen += 1;
        vec![EdgeRow {
            dst_path: s.file.clone(),
            dst_unit: String::new(),
            kind: s.kind,
            rung: 1,
            granularity: 0,
            via_reexport: 0,
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
    recommit(&mut conn, "use crate::y;\n");
    let sites: i64 = conn
        .query_row("SELECT COUNT(*) FROM sites", [], |r| r.get(0))
        .expect("sites");
    assert_eq!(sites, 1, "old sites replaced, not stacked");
}

fn edge_count(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0))
        .expect("edge count")
}

fn pending_count(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM resolve_pending", [], |r| r.get(0))
        .expect("pending count")
}

/// One committed phase-1 re-refresh of file 1 — the shared arrange of
/// the debt battery and the lifecycle battery's replacement stanza.
fn recommit(conn: &mut Connection, text: &str) {
    let tx = conn.transaction().expect("tx");
    refresh_graph(&tx, 1, text, Lang::Rust).expect("refresh");
    tx.commit().expect("commit");
}

/// The empty-dirty, ledger-driven phase 1.5 both debt legs end on,
/// plus the retirement assertion they share.
fn settle_and_expect_clear(conn: &mut Connection) {
    resolve_refreshed(conn, &BTreeSet::new(), one_edge).expect("settle");
    assert_eq!(pending_count(conn), 0, "settled debt retires by evidence");
}

fn one_edge(s: &CachedSite) -> Vec<EdgeRow> {
    vec![EdgeRow {
        dst_path: s.file.clone(),
        dst_unit: String::new(),
        kind: s.kind,
        rung: 1,
        granularity: 0,
        via_reexport: 0,
    }]
}

/// The nightly-CI interleaving (run 31991997431), projected onto one
/// connection so it is deterministic: writer B refreshes a file,
/// writer A's phase-2 sweep lands edges on B's NEW site rows and
/// sets resolve_key (so B's own sweep skips), then B runs phase 1.5
/// over its dirty file. Plan v1.7 says every write path is
/// idempotent — the edge set must equal one serial pass, not stack.
#[test]
fn phase_15_is_idempotent_over_a_swept_file() {
    let mut conn = seeded("mod alpha;\n");
    assert!(
        ensure_resolved(&mut conn, 7, one_edge).expect("sweep"),
        "the racing sweep fires"
    );
    assert_eq!(edge_count(&conn), 1);
    let dirty: BTreeSet<String> = ["a.rs".to_string()].into();
    resolve_refreshed(&mut conn, &dirty, one_edge).expect("phase 1.5");
    assert_eq!(
        edge_count(&conn),
        1,
        "phase 1.5 over a swept file must converge, not stack"
    );
}

/// The v1.2.0 release-night loss (CI run 32964681934), projected onto
/// one connection so it is deterministic: a sweep lands edges and sets
/// resolve_key; a content refresh then cascade-drops that file's edges
/// (repair deferred to end-of-run) — and the process dies before phase
/// 1.5 (daemon shutdown amputated the cold-start build). The NEXT run
/// walks a clean tree (content hash matches → nothing dirty) and its
/// sweep skips on the standing key, so without a persisted debt the
/// hole is silent and permanent. The debt row must survive the death
/// and be settled by that next run's empty-dirty phase 1.5.
#[test]
fn a_refresh_debt_survives_process_death_and_the_next_run_settles_it() {
    let mut conn = seeded("mod alpha;\n");
    assert!(ensure_resolved(&mut conn, 7, one_edge).expect("sweep"));
    assert_eq!(edge_count(&conn), 1);
    recommit(&mut conn, "mod alpha;\n");
    assert_eq!(edge_count(&conn), 0, "the cascade dropped the edges");
    // …process death here; the next run finds nothing dirty itself
    assert!(!ensure_resolved(&mut conn, 7, one_edge).expect("skip"));
    settle_and_expect_clear(&mut conn);
    assert_eq!(edge_count(&conn), 1, "the persisted debt must be settled");
}

/// A firing sweep replays every file's edges, so it retires the whole
/// debt ledger — orphan rows included — in the same commit.
#[test]
fn a_firing_sweep_retires_the_whole_debt_ledger() {
    let mut conn = seeded("mod alpha;\n");
    recommit(&mut conn, "mod alpha;\n");
    conn.execute("INSERT INTO resolve_pending (path) VALUES ('gone.rs')", [])
        .expect("orphan debt");
    assert!(ensure_resolved(&mut conn, 9, one_edge).expect("sweep"));
    assert_eq!(
        pending_count(&conn),
        0,
        "sweep covered every file — ledger empty"
    );
}

/// The load-bearing half of phase 1.5's settle-entire claim: a debt
/// whose file has since been removed resolves to NOTHING and the row
/// is still cleared — the ledger can never accumulate rows no run
/// will ever settle (this drives resolve_refreshed itself, where the
/// claim lives; the sweep test above retires orphans by a blanket
/// delete and pins nothing about this branch).
#[test]
fn phase_15_settles_an_orphan_debt_to_nothing() {
    let mut conn = seeded("mod alpha;\n");
    // the seeding refresh booked its own (legitimate) debt for a.rs;
    // retire it so the orphan is the ONLY row this leg settles
    conn.execute("DELETE FROM resolve_pending", [])
        .expect("isolate the orphan");
    conn.execute("INSERT INTO resolve_pending (path) VALUES ('gone.rs')", [])
        .expect("orphan debt");
    settle_and_expect_clear(&mut conn);
    assert_eq!(edge_count(&conn), 0, "a vanished file resolves to nothing");
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
    // the md slug facts ride the same pair list (walkidx) — the
    // M5-close staleness repayment's key-level contract, pinned
    // where resolve_key lives and not only in the e2e
    assert_ne!(
        base,
        resolve_key(&one, &[("b.md".to_string(), 9)]),
        "md slug facts participate"
    );
    // the Rust pub-use surface rides the same list (§4 R5 amendment)
    assert_ne!(
        base,
        resolve_key(&one, &[("b.rs".to_string(), 9)]),
        "rs pub-use facts participate"
    );
}
