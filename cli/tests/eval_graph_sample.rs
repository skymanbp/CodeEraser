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

use codeeraser::graph::sites::detect;
use eval_graph_sample_parts as select;
use eval_support::{
    content_sha, eval_doc, generated_from, git_in, kind_counts, lang_of, load, out_dir, write_doc,
};
use select::binding::universes;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Where a corpus repository lives: the enclosing repo for self, the
/// machine-local clone for external corpora (the out_dir convention
/// every eval payload uses; SOURCES.md pins the commits).
fn repo_of(tag: &str) -> Option<String> {
    if tag == "self" {
        return None;
    }
    let dir = out_dir().join("corpora").join(tag);
    assert!(
        dir.is_dir(),
        "{}: clone the corpus first (contracts/eval/SOURCES.md)",
        dir.display()
    );
    Some(dir.to_string_lossy().into_owned())
}

/// Every site of one frozen universe, re-detected at the pinned tip.
/// Each file must reproduce BOTH its frozen content identity and its
/// frozen per-kind counts — the pool equals the frozen universe by
/// checked reconstruction, not by trust.
fn corpus_pool(tag: &str, doc: &Value) -> Vec<select::Site> {
    let repo = repo_of(tag);
    let tip = doc["corpus"]["tip"].as_str().expect("tip");
    let mut pool = Vec::new();
    for row in doc["files"].as_array().expect("files") {
        let path = row["path"].as_str().expect("path");
        let text = git_in(repo.as_deref(), &["show", &format!("{tip}:{path}")]);
        assert_eq!(
            content_sha(&text),
            row["sha256"].as_str().expect("sha256"),
            "{tag}/{path}: content identity drifted from the frozen universe"
        );
        let lang = row["lang"].as_str().expect("lang");
        let sites = detect(&text, lang_of(lang));
        assert_eq!(
            json!(kind_counts(&sites)),
            row["sites"],
            "{tag}/{path}: detector disagrees with the frozen universe"
        );
        for s in sites {
            pool.push(select::Site {
                corpus: tag.into(),
                commit: tip.into(),
                path: path.into(),
                line: s.line as u64,
                nth: s.nth as u64,
                kind: s.kind.into(),
                lang: lang.into(),
                spec: s.spec,
            });
        }
    }
    pool
}

#[test]
#[ignore] // needs every corpus repository at its pinned tip
fn generate_graph_sample() {
    let mut pool = Vec::new();
    let mut sources = Vec::new();
    for (tag, doc) in universes() {
        pool.extend(corpus_pool(&tag, &doc));
        sources.push(json!({"corpus": tag, "tip": doc["corpus"]["tip"],
                            "total_sites": doc["summary"]["total_sites"]}));
    }
    let draw = select::draw(&pool);
    let doc = json!({
        "schema": "ce.eval-graph-sample/1.0.0",
        "constants": select::constants(),
        "generated_from": generated_from(),
        "method": "hash-ranked stratified draw over the union of the five \
                   frozen site universes, reconstructed file-by-file against \
                   their frozen sha256 and per-kind counts. rank id = \
                   sha256(domain|corpus|commit|path|line|nth|kind|spec) — the \
                   design-§5 payload plus nth, which entered site identity at \
                   2b-iii (same-line sites would otherwise collide); spec is \
                   the last field so the encoding is injective. Primaries: \
                   per-language floor then largest-remainder extras, spread \
                   over kinds by largest remainder, top of the site-domain \
                   rank per cell; rows are stored in audit-domain order. \
                   Backups: per language, the audit-domain rank over the \
                   unselected remainder — audit walks them until 100 answered \
                   rows. The rung domain is pre-registered here and \
                   materializes at 2f when measured rungs exist (not a gate).",
        "sources": sources,
        "allocation": draw.cells,
        "rows": draw.primary.iter().map(|s| select::row_json(s)).collect::<Vec<_>>(),
        "backups": draw.backups.iter().map(|s| select::row_json(s)).collect::<Vec<_>>(),
    });
    let path = eval_doc("graph-sample");
    write_doc(&path, &doc, &format!("{path} written"));
}

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
