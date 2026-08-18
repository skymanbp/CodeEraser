//! M5-2c: the hash-ranked stratified audit sample over the five
//! frozen graph-site universes (design §5). Selection machinery
//! lives in eval_graph_sample_parts (shared by generator and gates);
//! this file owns the pool walk (generator-only, needs git and the
//! corpus clones) and the git-free CI gates.
//!
//! The 2c exit criterion: the frozen sample doc is COMMITTED before
//! any cli/src/graph/ladder/ file exists — "sampled before the
//! resolver" is provable from git log, and the 2d provenance gate
//! (G13) turns it into an ancestry assertion.
//!
//! Generate (needs the four external corpora cloned under
//! `<out_dir>/corpora/`, SOURCES.md commits; the self corpus is the
//! enclosing repo):
//!   cargo test --test eval_graph_sample -- --ignored --nocapture

mod eval_graph_sample_parts;
mod eval_support;

use eval_graph_sample_parts as select;
use eval_support::{eval_doc, load};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Count rows per derived key.
fn tally(rows: &[Value], key: impl Fn(&Value) -> String) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for r in rows {
        *out.entry(key(r)).or_insert(0) += 1;
    }
    out
}

fn cell_key(r: &Value) -> String {
    format!(
        "{}/{}",
        r["lang"].as_str().expect("lang"),
        r["kind"].as_str().expect("kind")
    )
}

fn lang_key(r: &Value) -> String {
    r["lang"].as_str().expect("lang").into()
}

/// The frozen sample unpacked once — the gates' one shared opening
/// (the repeated load-and-unpack prologue was caught by the dedup
/// ratchet on this file's first scan).
fn sample() -> (Vec<Value>, Vec<Value>, Value) {
    let doc = load(&eval_doc("graph-sample"));
    let rows = doc["rows"].as_array().expect("rows").clone();
    let backups = doc["backups"].as_array().expect("backups").clone();
    (rows, backups, doc)
}

/// Structural gate: verify() over every row (recomputed hashes,
/// duplicate-id refusal), the plan-literal sizes, and the frozen
/// audit order on both arrays.
#[test]
fn graph_sample_verifies() {
    let (rows, backups, doc) = sample();
    assert_eq!(doc["constants"], select::constants(), "constants drifted");
    assert_eq!(rows.len(), 100, "primary must be the plan-literal 100");
    let mut seen = BTreeSet::new();
    for row in rows.iter().chain(backups.iter()) {
        select::verify_row(row, &mut seen).expect("verify()");
    }
    for pair in rows.windows(2) {
        assert!(
            pair[0]["audit"].as_str() < pair[1]["audit"].as_str(),
            "primary audit order broken"
        );
    }
    for pair in backups.windows(2) {
        let key = |r: &Value| (lang_key(r), r["audit"].as_str().expect("audit").to_string());
        assert!(
            key(&pair[0]) < key(&pair[1]),
            "backup (lang, audit) order broken"
        );
    }
}

/// Strata gate: row counts match the allocation cell-for-cell (and
/// carry no cell outside it), every language holds its pre-registered
/// floor (the G5 shape at sample level), and the backup tail is
/// exactly BACKUP_PER_LANG per language.
#[test]
fn graph_sample_strata_hold() {
    let (rows, backups, doc) = sample();
    let cells = doc["allocation"].as_object().expect("allocation");
    let counted = tally(&rows, cell_key);
    for (cell, quota) in cells {
        assert_eq!(
            counted.get(cell).copied().unwrap_or(0),
            quota.as_u64().expect("quota"),
            "{cell}: rows disagree with the allocation"
        );
    }
    for cell in counted.keys() {
        assert!(
            cells.contains_key(cell),
            "{cell}: sampled outside the allocation"
        );
    }
    let langs = tally(&rows, lang_key);
    for (lang, n) in &langs {
        assert!(
            *n >= select::MIN_PER_LANG,
            "{lang}: {n} primaries below the pre-registered floor"
        );
    }
    let backup_langs = tally(&backups, lang_key);
    assert_eq!(
        backup_langs.keys().collect::<Vec<_>>(),
        langs.keys().collect::<Vec<_>>(),
        "backup languages differ from primary languages"
    );
    for (lang, n) in &backup_langs {
        assert_eq!(*n, select::BACKUP_PER_LANG, "{lang}: backup tail size");
    }
}

/// The frozen corpus spectrum, asserted (2c/2d review F14: a forged
/// row swapped across corpora passed every gate — per-cell strata do
/// not pin per-corpus counts).
#[test]
fn graph_sample_corpus_spectrum_holds() {
    let (rows, _, _) = sample();
    let spectrum: BTreeMap<String, u64> = [
        ("cobra", 12u64),
        ("requests", 15),
        ("ripgrep", 25),
        ("self", 18),
        ("zod", 30),
    ]
    .iter()
    .map(|(c, n)| (c.to_string(), *n))
    .collect();
    assert_eq!(
        tally(&rows, |r| r["corpus"].as_str().expect("corpus").into()),
        spectrum,
        "corpus spectrum drifted from the frozen draw"
    );
}

/// Counterfactual (the G9 discipline): a tampered payload and a
/// duplicated row must actually refuse — asserted, not assumed.
#[test]
fn graph_sample_refuses_tampering() {
    let (rows, _, _) = sample();
    let mut row = rows[0].clone();
    row["spec"] = json!(format!("{}x", row["spec"].as_str().expect("spec")));
    assert!(
        select::verify_row(&row, &mut BTreeSet::new()).is_err(),
        "tampered spec must refuse"
    );
    let original = rows[0].clone();
    let mut seen = BTreeSet::new();
    select::verify_row(&original, &mut seen).expect("pristine row verifies");
    assert!(
        select::verify_row(&original, &mut seen).is_err(),
        "duplicate id must refuse"
    );
}
