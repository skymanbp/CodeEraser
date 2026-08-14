//! `ce churn` end-to-end over a synthetic history: append vs rewrite
//! classification, per-unit ledger attribution, co-change counting,
//! and window survival, on commits authored NOW (so any window covers
//! them — no clock injection needed for determinism).

mod common;

use codeeraser::churn;
use std::path::Path;

fn commit(dir: &Path, msg: &str) {
    common::git(dir, &["add", "."]);
    common::git(dir, &["commit", "-qm", msg]);
}

/// Fresh repo whose ROOT commit adds a.rs — the shared opening of
/// both cases (and the F12 fixture shape: a parentless commit).
fn seeded(name: &str) -> std::path::PathBuf {
    let root = common::tmp(name);
    common::git(&root, &["init", "-q"]);
    std::fs::write(root.join("a.rs"), common::rust_fn(1)).expect("a.rs");
    commit(&root, "seed");
    root
}

#[test]
fn churn_classifies_append_rewrite_cochange_and_survival() {
    // commit 1 (root): one new function — pure append
    let root = seeded("churn-e2e");

    // commit 2: edit inside work_1 (rewrite) AND add a new file
    // (append); a.rs + b.rs change together
    let edited = common::rust_fn(1).replace("+ 7;", "+ 8;\n            total_1 += 1;");
    std::fs::write(root.join("a.rs"), &edited).expect("a.rs edit");
    std::fs::write(root.join("b.rs"), common::rust_fn(2)).expect("b.rs");
    commit(&root, "edit + new");

    // commit 3: touch both again — co-change count reaches 2 — and
    // delete b.rs's function body (those added lines stop surviving)
    std::fs::write(root.join("a.rs"), format!("{edited}\n// tail\n")).expect("a.rs tail");
    std::fs::write(root.join("b.rs"), "// emptied\n").expect("b.rs emptied");
    commit(&root, "entangle + churn");

    assert_report(&churn::run(&root, 30).expect("churn"));
}

/// Attack review F12 (minimal surviving-root case): a root commit
/// inside the window must count its additions via the empty-tree
/// base. Before the fix `sha^..sha` failed, added_in_window stayed 0,
/// and blame still counted every surviving root line.
#[test]
fn root_commit_additions_are_counted() {
    let root = seeded("churn-root");
    let r = churn::run(&root, 30).expect("churn");
    assert_eq!(r.commits, 1);
    assert!(r.added_in_window() > 0, "root additions counted");
    assert_eq!(r.added_in_window(), r.surviving, "everything survives");
    // the whole seed lands in ONE ledger row: a.rs's work_1 unit
    assert_eq!(r.units.len(), 1, "rows: {:?}", r.units);
    let row = &r.units[0];
    assert_eq!(
        (row.path.as_str(), row.key.as_str(), row.nth),
        ("a.rs", "work_1/2", 0)
    );
    assert_eq!(row.appended, r.added_in_window());
    assert_eq!(row.rewrote, 0, "an empty before side cannot rewrite");
}

fn assert_report(r: &churn::Report) {
    assert_eq!(r.commits, 3);
    assert!(
        r.rewrite_lines() >= 2,
        "work_1 edits are rewrite: {}",
        r.rewrite_lines()
    );
    assert!(
        r.append_lines() > r.rewrite_lines(),
        "seed + b.rs dominate as append: {} vs {}",
        r.append_lines(),
        r.rewrite_lines()
    );
    assert_ledger(r);
    assert!(
        r.cochange.contains(&("a.rs".into(), "b.rs".into(), 2)),
        "co-change pair counted twice: {:?}",
        r.cochange
    );
    assert!(
        r.surviving < r.added_in_window(),
        "b.rs's emptied function must count as churned: {} of {}",
        r.surviving,
        r.added_in_window()
    );
    let json = churn::report_json(r);
    assert_eq!(json["schema"], "ce.churn-report/0.1.0");
    assert_eq!(
        json["added_in_window"].as_u64().unwrap(),
        json["surviving"].as_u64().unwrap() + json["churned"].as_u64().unwrap(),
        "survival ledger must balance"
    );
}

/// Per-unit attribution (M5-3h): every fixture edit is pinned to its
/// owning (path, key, nth) row, and the top level ("" key) is a real
/// destination, not lost lines.
fn assert_ledger(r: &churn::Report) {
    let row = |p: &str, k: &str| {
        r.units
            .iter()
            .find(|u| u.path == p && u.key == k && u.nth == 0)
    };
    let work1 = row("a.rs", "work_1/2").expect("a.rs work_1 row");
    assert!(work1.rewrote >= 2, "commit-2 edits: {work1:?}");
    assert!(work1.appended >= 8, "the seed body: {work1:?}");
    let work2 = row("b.rs", "work_2/2").expect("b.rs work_2 row");
    assert!(work2.appended >= 8, "new file is append: {work2:?}");
    assert_eq!(work2.rewrote, 0, "no before side in b.rs: {work2:?}");
    let tail = row("a.rs", "").expect("a.rs top-level row");
    assert!(tail.appended >= 1, "// tail lands at top level: {tail:?}");
    let emptied = row("b.rs", "").expect("b.rs top-level row");
    assert!(
        emptied.appended >= 1,
        "// emptied is top-level: {emptied:?}"
    );
}
