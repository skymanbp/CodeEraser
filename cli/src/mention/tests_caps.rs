//! The cap and census legs (K40/K41) on the tests_index scaffold: the
//! judged∖U census by its four causes, the dist-mirror witness on a
//! warm run, the per-file cap as a standing store fact, table-cap
//! starvation retried until room exists, and the connection tuning
//! undone once the pass returns.

use super::tests_index::{mentioned, put, query, run, seeded};
use super::{Caps, Stats, TABLE_ROW_CAP, refresh_under};
use crate::dedup::{Params, index::Index};
use crate::scan::lang::Lang;
use std::path::Path;

fn capped(dir: &Path, idx: &Index, file: usize, table: usize) -> Stats {
    refresh_under(dir, idx, Caps { file, table }).expect("refresh")
}

/// K40/K41: judged files outside U by the four named causes, and the
/// header's skip counters for the same files.
#[test]
fn judged_files_outside_the_universe_are_counted_by_cause() {
    // one comment line past the cap: over 4 MiB by bytes, trivial to
    // parse — a 2M-line fixture cost the judged pipeline six minutes
    let mut big = b"# ".to_vec();
    big.extend(std::iter::repeat_n(b'x', 4 * 1024 * 1024 + 1));
    let (dir, mut idx) = seeded(
        "k40-outside",
        &[
            ("big.py", &big),
            ("nul.py", b"x = 1\0\n"),
            (".gitignore", b"ignored.py\n"),
            ("ignored.py", b"y = 2\n"),
            ("sub/.git", b"gitdir: elsewhere\n"),
            ("sub/nested.py", b"w = 4\n"),
            ("plain.py", b"z = 3\n"),
        ],
    );
    for rel in [
        "big.py",
        "nul.py",
        "ignored.py",
        "sub/nested.py",
        "plain.py",
    ] {
        let bytes = std::fs::read(dir.join(rel)).expect(rel);
        idx.refresh_file(rel, &bytes, Lang::Python, Params::default())
            .expect(rel);
    }
    let s = run(&dir, &idx);
    assert_eq!(
        (
            s.outside.oversize,
            s.outside.binary,
            s.outside.nested,
            s.outside.ignored
        ),
        (1, 1, 1, 1)
    );
    assert_eq!(
        (s.skipped.oversize, s.skipped.binary, s.universe),
        (1, 1, 2),
        "{s:?}"
    );
}

/// K41: a generated mirror is reported, not hidden — the source count
/// grows when `dist/app.js` appears, the bundler-suffix witness counts
/// the `name$N` runs the JS arm keeps whole, and it still counts them
/// on the next, unchanged run: a tree fact, not a refresh delta.
#[test]
fn a_dist_mirror_is_visible_in_the_header_on_every_run() {
    let (dir, idx) = seeded("k41-dist", &[("src/a.ts", b"export const foo = 1;\n")]);
    let before = run(&dir, &idx);
    put(
        &dir,
        "dist/app.js",
        b"var foo$1 = 1, bar$22 = 2, baz$x = 3;\n",
    );
    let after = run(&dir, &idx);
    assert_eq!(after.sources, before.sources + 1);
    assert_eq!(after.dist_js_dedup_runs, 2);
    assert!(
        !mentioned(&idx, "foo", "src/a.ts"),
        "the JS arm keeps foo$1 whole"
    );
    let warm = run(&dir, &idx);
    assert_eq!((warm.run.refreshed, warm.dist_js_dedup_runs), (0, 2));
}

/// The per-file cap is a function of the bytes: the clip is stored as
/// final under the file's hash, counted this run, and stands in the
/// header of every later run as a capped file.
#[test]
fn the_per_file_cap_clips_once_and_stays_visible() {
    let (dir, idx) = seeded("cap-file", &[("a.txt", b"a_one a_two a_three\n")]);
    let cold = capped(&dir, &idx, 2, TABLE_ROW_CAP);
    assert_eq!(
        (cold.rows, cold.run.clipped, cold.run.starved, cold.capped),
        (2, 1, 0, 1)
    );
    let warm = capped(&dir, &idx, 2, TABLE_ROW_CAP);
    assert_eq!(
        (warm.run.refreshed, warm.run.clipped, warm.capped),
        (0, 0, 1)
    );
}

/// The table cap is a function of the whole store: a starved file
/// keeps neither rows nor hash, is retried — and counted — every run,
/// and lands the moment a vanished file frees the room, in that same
/// run (the reap of vanished paths precedes the writes).
#[test]
fn a_starved_file_is_retried_until_the_table_has_room() {
    let (dir, idx) = seeded(
        "cap-table",
        &[("a.txt", b"a_one a_two\n"), ("b.txt", b"b_one b_two\n")],
    );
    let first = capped(&dir, &idx, 65_536, 3);
    assert_eq!(
        (first.rows, first.run.starved, first.run.clipped),
        (2, 1, 2)
    );
    assert!(mentioned(&idx, "a_one", "x") && !mentioned(&idx, "b_one", "x"));
    let again = capped(&dir, &idx, 65_536, 3);
    assert_eq!(
        (again.run.refreshed, again.run.starved),
        (0, 1),
        "starvation is re-counted, never gated"
    );
    std::fs::remove_file(dir.join("a.txt")).expect("rm");
    let freed = capped(&dir, &idx, 65_536, 3);
    assert_eq!(
        (
            freed.run.removed,
            freed.run.refreshed,
            freed.run.starved,
            freed.rows
        ),
        (1, 1, 0, 2)
    );
    assert!(mentioned(&idx, "b_one", "x"));
}

/// The tuning guard: the three pass-scoped pragmas are back to what
/// the connection had before, once the pass returns.
#[test]
fn the_connection_tuning_is_undone_after_the_pass() {
    let (dir, idx) = seeded("tuned", &[("a.txt", b"alpha_one\n")]);
    let cache: i64 = query(&idx, "PRAGMA cache_size", []);
    run(&dir, &idx);
    let after: (i64, i64, i64) = (
        query(&idx, "PRAGMA cache_size", []),
        query(&idx, "PRAGMA synchronous", []),
        query(&idx, "PRAGMA wal_autocheckpoint", []),
    );
    assert_eq!(
        after,
        (cache, 2, 1000),
        "prior cache, FULL, default checkpoint"
    );
}
