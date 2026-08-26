//! ADR-003 as amended (plan v1.7): the index is a CONVERGENT
//! multi-writer cache. Convergence rests on two named legs (the
//! index.rs header, reworded by the clearance review): per-file
//! refresh converges by IDEMPOTENCE (delete-then-reinsert of
//! identical bytes, content-gated), while the order-sensitive paths
//! (phase-2 sweep, phase 1.5, remove_missing) hold IMMEDIATE locks
//! around the reads they act on. The audit demonstrated the old
//! "daemon is the sole writer" clause never held (nine subcommands
//! write in-process; the daemon's own cold start races its accept
//! loop) — this battery is the amendment's acceptance: real OS
//! processes write ONE database concurrently, and the row-level
//! digest must equal a serial run's.

use crate::common;
use std::path::Path;

/// Compact habitat exercising every table: fingerprint-bearing rust
/// (a T2 pair), sites/edges (mod + use + md links), symbols, docsegs.
fn habitat(dir: &Path) {
    let files: [(&str, String); 6] = [
        ("Cargo.toml", "[package]\nname='conc'".into()),
        ("src/lib.rs", "mod util;\n".into()),
        ("src/util.rs", common::rust_fn(1)),
        ("src/twin.rs", common::rust_fn(2)),
        ("README.md", "# Conc\n[code](./src/lib.rs)\n".into()),
        (
            "docs/guide.md",
            "# Guide\nsee [top](../README.md#conc)\n".into(),
        ),
    ];
    for (rel, content) in files {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(path, content).expect(rel);
    }
}

/// Row-level convergence digest: per-table counts, the resolve_key,
/// and a content digest over the edges and files tables — path-
/// relative, so two habitats materialized in different tmp dirs
/// compare byte-for-byte.
fn digest(db: &Path) -> String {
    let conn = rusqlite::Connection::open(db).expect("open");
    let count = |t: &str| -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |r| r.get(0))
            .expect(t)
    };
    let key: i64 = conn
        .query_row("SELECT v FROM meta WHERE k = 'resolve_key'", [], |r| {
            r.get(0)
        })
        .expect("resolve_key");
    let rows: Vec<String> = conn
        .prepare(
            "SELECT f.path || '|' || e.dst_path || '|' || e.dst_unit || '|' || e.kind || '|' || e.rung
             FROM edges e JOIN sites s ON s.id = e.site_id JOIN files f ON f.id = s.file_id
             ORDER BY 1",
        )
        .expect("prepare")
        .query_map([], |r| r.get::<_, String>(0))
        .expect("query")
        .collect::<Result<_, _>>()
        .expect("rows");
    format!(
        "files={} fps={} symbols={} sites={} edges={} unitsig={} docsegs={} key={} edges=[{}]",
        count("files"),
        count("fingerprints"),
        count("symbols"),
        count("sites"),
        count("edges"),
        count("unitsig"),
        count("docsegs"),
        key,
        rows.join(";")
    )
}

/// Shared arrange + assert: the serial-order reference digest for a
/// fresh habitat, and the convergence claim over a raced one (the
/// two legs' identical stanzas were the ratchet's own first catch in
/// this file).
fn serial_want(tag: &str) -> String {
    let serial = common::tmp(tag);
    habitat(&serial);
    assert!(
        common::run_ce(&serial, &["dedup", "."]).status.success(),
        "serial run"
    );
    digest(&serial.join(".ce/index.db"))
}

fn assert_converged(dir: &Path, want: &str, what: &str) {
    assert_eq!(
        digest(&dir.join(".ce/index.db")),
        want,
        "{what} diverged from the serial order"
    );
}

/// Launch two real `ce dedup` processes on `dir` together and require
/// both to land (busy_timeout absorbs the contention) — the one race
/// throat both writer legs share (the ratchet's second catch here).
fn race_two(dir: &Path) {
    let spawn = || {
        std::process::Command::new(env!("CARGO_BIN_EXE_ce"))
            .args(["dedup", "."])
            .current_dir(dir)
            .spawn()
            .expect("spawn ce dedup")
    };
    let (mut a, mut b) = (spawn(), spawn());
    let (ra, rb) = (a.wait().expect("wait a"), b.wait().expect("wait b"));
    assert!(
        ra.success() && rb.success(),
        "concurrent writers must both land: {ra:?} {rb:?}"
    );
}

/// Two real `ce dedup` processes on ONE database, launched together;
/// the surviving state must equal a serial run's digest.
#[test]
fn two_concurrent_dedups_converge_to_the_serial_state() {
    let want = serial_want("conc-serial");
    let dir = common::tmp("conc-double");
    habitat(&dir);
    race_two(&dir);
    assert_converged(&dir, &want, "two concurrent dedups");
}

/// A DIRTY tree raced by two writers: after the first index lands,
/// one file mutates and another is deleted, then two `ce dedup`
/// processes race the update. This forces the content-gate refresh,
/// the FK cascade AND remove_missing's delete to interleave for real
/// (clearance review: the quiescent-habitat legs never exercised the
/// deletion path at all).
#[test]
fn racing_writers_over_a_mutated_tree_converge() {
    let serial = common::tmp("conc-mut-serial");
    habitat(&serial);
    assert!(common::run_ce(&serial, &["dedup", "."]).status.success());
    mutate(&serial);
    assert!(common::run_ce(&serial, &["dedup", "."]).status.success());
    let want = digest(&serial.join(".ce/index.db"));

    let dir = common::tmp("conc-mut-double");
    habitat(&dir);
    assert!(common::run_ce(&dir, &["dedup", "."]).status.success());
    mutate(&dir);
    race_two(&dir);
    assert_converged(&dir, &want, "raced writers over a mutated tree");
}

/// The same mutation on both trees: guide.md's heading changes (slug
/// fact + content refresh) and twin.rs vanishes (remove_missing).
fn mutate(dir: &Path) {
    std::fs::write(
        dir.join("docs/guide.md"),
        "# Guide2\nsee [top](../README.md)\n",
    )
    .expect("guide.md");
    std::fs::remove_file(dir.join("src/twin.rs")).expect("rm twin.rs");
}

/// The daemon's cold-start indexing OVERLAPPING an external writer on
/// the same database. Shutdown deliberately AMPUTATES a mid-flight
/// build — `bye` ⇒ prompt death is a contract eject's bounded
/// teardown and the stale-respawn chain both stand on (a join-on-exit
/// variant was tried and refuted by three A/B eject repros,
/// 2026-08-26) — so the honest claim is the convergent-cache one: an
/// amputated build's committed refresh leaves a resolve_pending debt
/// row (same transaction as the edge drop), and after ONE settling
/// run the raced database equals the serial one (the v1.2.0 release-
/// night loss, CI 32964681934, was this schedule with no ledger: the
/// hole was silent and permanent). The deterministic mechanisms are
/// pinned by store_tests::phase_15_is_idempotent_over_a_swept_file
/// and the debt-ledger legs beside it.
#[test]
fn daemon_cold_start_races_an_external_writer_and_converges() {
    let want = serial_want("conc-dserial");
    let root = common::tmp("conc-daemon");
    habitat(&root);
    // spawn_daemon_ready returns at first ping — the cold-start
    // indexing thread is still running; the external dedup lands on
    // top of it
    let child = common::spawn_daemon_ready(&root);
    let external = common::run_ce(&root, &["dedup", "."]);
    assert!(external.status.success(), "external writer under daemon");
    common::shutdown_and_wait(&root, child, "conc daemon");
    // the settling run: pays whatever debt an amputated build left
    assert!(
        common::run_ce(&root, &["dedup", "."]).status.success(),
        "settling run"
    );
    assert_converged(&root, &want, "daemon vs external writer");
}
