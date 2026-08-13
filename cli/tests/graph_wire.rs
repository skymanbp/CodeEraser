//! Phase-2 wiring end to end: analyze() drives the walk, the key
//! gate, and the ladder callback, and edge rows land in .ce/index.db
//! with the frozen kind/granularity codes. The second run pins phase
//! 1.5: a content edit under a STABLE resolve_key cascade-drops the
//! file's edges, and only the per-file refresh restores them —
//! without it those edges stay missing until the next key shift,
//! not merely for one run (the re-walk sees the stored hash already
//! current and never refreshes again).

use codeeraser::dedup;
use std::path::Path;

mod common;

type Row = (String, String, String, i64, i64, i64);

/// (src, dst, unit, edge kind, rung, granularity) rows, owned.
fn rows(spec: &[(&str, &str, &str, i64, i64, i64)]) -> Vec<Row> {
    spec.iter()
        .map(|&(s, d, u, k, r, g)| (s.into(), d.into(), u.into(), k, r, g))
        .collect()
}

/// Every edge joined back to its source path, deterministically
/// ordered — the whole-table set, so a phantom or doubled row fails
/// the same assert as a missing one.
fn edges(db: &Path) -> Vec<Row> {
    let conn = rusqlite::Connection::open(db).expect("open db");
    let mut stmt = conn
        .prepare(
            "SELECT f.path, e.dst_path, e.dst_unit, e.kind, e.rung, e.granularity
             FROM edges e JOIN sites s ON s.id = e.site_id
             JOIN files f ON f.id = s.file_id
             ORDER BY f.path, e.dst_path, e.dst_unit",
        )
        .expect("prepare");
    stmt.query_map([], |r| {
        Ok((
            r.get(0)?,
            r.get(1)?,
            r.get(2)?,
            r.get(3)?,
            r.get(4)?,
            r.get(5)?,
        ))
    })
    .expect("query")
    .collect::<Result<Vec<Row>, _>>()
    .expect("rows")
}

#[test]
fn edges_land_and_survive_content_refresh() {
    let dir = common::tmp("wire-e2e");
    std::fs::write(dir.join("b.ts"), "export {};\n").expect("b.ts");
    std::fs::write(dir.join("a.ts"), "import { x } from './b';\n").expect("a.ts");
    // the autolink resolves External and must land NO row
    std::fs::write(
        dir.join("README.md"),
        "# Top\n[code](./a.ts)\n[sec](#top)\n<https://example.com>\n",
    )
    .expect("README.md");
    dedup::analyze(&dir, None, None, None).expect("first run");
    let want = rows(&[
        ("README.md", "README.md", "top", 1, 4, 2), // doc_link, R4, section
        ("README.md", "a.ts", "", 2, 1, 0),         // doc_ref, R1, file
        ("a.ts", "b.ts", "", 0, 1, 0),              // import, R1, file
    ]);
    let db = dir.join(".ce/index.db");
    assert_eq!(edges(&db), want, "first run: sweep populates the table");
    // same file set + configs → same resolve_key → NO sweep; the
    // content edit cascade-drops a.ts's edges — phase 1.5 or bust
    std::fs::write(dir.join("a.ts"), "// edited\nimport { x } from './b';\n").expect("edit");
    dedup::analyze(&dir, None, None, None).expect("second run");
    assert_eq!(edges(&db), want, "second run under a stable key");
}
