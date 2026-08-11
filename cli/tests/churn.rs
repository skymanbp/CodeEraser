//! `ce churn` end-to-end over a synthetic history: append vs rewrite
//! classification, co-change counting, and window survival, on
//! commits authored NOW (so any window covers them — no clock
//! injection needed for determinism).

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
    assert!(r.added_in_window > 0, "root additions counted");
    assert_eq!(r.added_in_window, r.surviving, "everything survives");
}

fn assert_report(r: &churn::Report) {
    assert_eq!(r.commits, 3);
    assert!(
        r.rewrite_lines >= 2,
        "work_1 edits are rewrite: {}",
        r.rewrite_lines
    );
    assert!(
        r.append_lines > r.rewrite_lines,
        "seed + b.rs dominate as append: {} vs {}",
        r.append_lines,
        r.rewrite_lines
    );
    assert!(
        r.cochange.contains(&("a.rs".into(), "b.rs".into(), 2)),
        "co-change pair counted twice: {:?}",
        r.cochange
    );
    assert!(
        r.surviving < r.added_in_window,
        "b.rs's emptied function must count as churned: {} of {}",
        r.surviving,
        r.added_in_window
    );
    let json = churn::report_json(r);
    assert_eq!(json["schema"], "ce.churn-report/0.1.0");
    assert_eq!(
        json["added_in_window"].as_u64().unwrap(),
        json["surviving"].as_u64().unwrap() + json["churned"].as_u64().unwrap(),
        "survival ledger must balance"
    );
}
