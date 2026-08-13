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
    refresh_graph(
        &tx,
        1,
        "mod alpha;\nfn holder() {\n    use crate::x;\n}\n",
        Lang::Rust,
    )
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
