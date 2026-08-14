//! `ce join` end to end (M5-3h): the three legs assemble over a real
//! fixture — a T2 clone pair both imported by main.rs, with a window
//! history that rewrites inside one clone body and appends outside
//! the other. Pins: the file tier carries all three legs (graph
//! position answered through the SAME wire deadcode judges), the
//! unit tier joins churn on the (path, key, nth) identity, and the
//! run is report-only (no verdicts before 3i).

mod common;

use codeeraser::join;
use std::path::{Path, PathBuf};

fn core_bin() -> String {
    std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    )
}

fn commit(dir: &Path, msg: &str) {
    common::git(dir, &["add", "."]);
    common::git(dir, &["commit", "-qm", msg]);
}

/// Two commits of a real Cargo layout: without Cargo.toml the rs
/// ladder has no crate root and `mod a;` correctly refuses. The
/// window edit is a literal tweak INSIDE work_1 (T2-invariant, so
/// the clone pair survives; churn sees a rewrite) plus a top-level
/// comment on b.rs (append; comments never enter the token stream).
fn fixture() -> PathBuf {
    let dir = common::tmp("join-e2e");
    common::git(&dir, &["init", "-q"]);
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"fx\"\n").expect("Cargo.toml");
    std::fs::create_dir_all(dir.join("src")).expect("src");
    std::fs::write(dir.join("src/main.rs"), "mod a;\nmod b;\nfn main() {}\n").expect("main.rs");
    std::fs::write(dir.join("src/a.rs"), common::rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("src/b.rs"), common::rust_fn(2)).expect("b.rs");
    commit(&dir, "seed");
    let tweaked = common::rust_fn(1).replace("+ 7;", "+ 9;");
    std::fs::write(dir.join("src/a.rs"), tweaked).expect("a.rs tweak");
    let touched = format!("{}// touched\n", common::rust_fn(2));
    std::fs::write(dir.join("src/b.rs"), touched).expect("b.rs touch");
    commit(&dir, "tweak + touch");
    dir
}

/// Tier F: the one similar pair carries all three legs.
fn assert_tier_f(r: &join::Report) {
    let [f] = r.files.as_slice() else {
        panic!("one file pair, got {:?}", r.files);
    };
    assert_eq!((f.a.as_str(), f.b.as_str()), ("src/a.rs", "src/b.rs"));
    assert!(f.blocks >= 1 && f.tokens >= 50, "similarity leg: {f:?}");
    let ga = f.graph_a.expect("a.rs graph leg answered");
    let gb = f.graph_b.expect("b.rs graph leg answered");
    assert!(
        ga[0] >= 1 && gb[0] >= 1,
        "main.rs imports both: {ga:?} {gb:?}"
    );
    assert!(f.churn_a.rewrote >= 1, "work_1 tweak is a rewrite: {f:?}");
    assert!(f.churn_b.appended >= 1, "b.rs comment is an append: {f:?}");
    assert_eq!(f.cochange, Some(2), "both commits touched the pair");
}

/// Tier U: the block's sides attribute to the two work units, and
/// churn joins on the SAME (path, key, nth) identity the caches
/// persist — the rewrite lands on work_1, never its twin.
fn assert_tier_u(r: &join::Report) {
    let u = r
        .units
        .iter()
        .find(|u| u.a.key == "work_1/2" || u.b.key == "work_1/2")
        .expect("unit row for the clone pair");
    let (one, two, c1, c2) = if u.a.key == "work_1/2" {
        (&u.a, &u.b, u.churn_a, u.churn_b)
    } else {
        (&u.b, &u.a, u.churn_b, u.churn_a)
    };
    assert_eq!((one.path.as_str(), one.nth), ("src/a.rs", 0));
    assert_eq!(
        (two.path.as_str(), two.key.as_str(), two.nth),
        ("src/b.rs", "work_2/2", 0)
    );
    assert!(c1.rewrote >= 1, "the tweak joined onto work_1: {u:?}");
    assert!(
        c2.appended >= 8 && c2.rewrote == 0,
        "work_2 only appended: {u:?}"
    );
}

#[test]
fn three_legs_assemble_on_both_tiers() {
    let dir = fixture();
    let r = join::run(&dir, None, &core_bin(), 30).expect("join");
    assert!(r.degraded.is_none(), "healthy graph reply");
    assert_eq!(r.commits, 2);
    assert_tier_f(&r);
    assert_tier_u(&r);
    // the JSON form declares itself and never fabricates a unit-tier
    // graph leg: null plus the R6 caveat on every unit row
    let doc = join::report_json(&r);
    assert_eq!(doc["schema"], "ce.join-report/0.1.0");
    for row in doc["units"].as_array().expect("units") {
        assert!(row["graph"].is_null(), "unit graph leg is null: {row}");
        assert_eq!(row["caveat"], join::churn_unit::GRAPH_CAVEAT);
    }
}
