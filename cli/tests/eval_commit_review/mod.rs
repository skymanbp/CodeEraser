//! Mechanical partition machinery for the commit-slice ground truth
//! (semantics: the retired commit-labels battery, git history). The
//! per-item review record is corpus-specific DATA and lives in
//! self.json / requests.json (data as data — as Rust consts the
//! parallel tables read as clone blocks to our own dedup ratchet);
//! every entry there was verified against the raw diff it describes.
//! The machinery here is corpus-neutral.

use serde_json::{Value, json};
use std::sync::OnceLock;

/// A named corpus's review record, embedded at compile time (all
/// three parse once). corrections: reviewed content-coincidence
/// entries {sha, file, added, lines, why}. relocated_units: reviewed
/// relocation targets {sha, to, units}. CI gates iterate every
/// frozen corpus, so resolution is BY NAME — resolving through the
/// active corpus would silently read the wrong record there.
fn tables_for(corpus: Option<&str>) -> &'static Value {
    static TABLES: OnceLock<[(Option<&'static str>, Value); 3]> = OnceLock::new();
    let parse = |raw: &str| serde_json::from_str(raw).expect("review record json");
    let all = TABLES.get_or_init(|| {
        [
            (None, parse(include_str!("self.json"))),
            (Some("requests"), parse(include_str!("requests.json"))),
            (Some("ripgrep"), parse(include_str!("ripgrep.json"))),
        ]
    });
    &all.iter()
        .find(|(n, _)| *n == corpus)
        .unwrap_or_else(|| panic!("no review record for corpus {corpus:?}"))
        .1
}

/// A record's `key` rows whose sha prefix matches `sha`.
fn rows_of<'a>(
    t: &'static Value,
    key: &str,
    sha: &'a str,
) -> impl Iterator<Item = &'static Value> + use<'a> {
    t[key]
        .as_array()
        .expect("review rows")
        .iter()
        .filter(move |r| sha.starts_with(r["sha"].as_str().expect("sha prefix")))
}

/// Project a record's review rows for one sha: carry `fields`
/// through, split the comma-joined units — the one shape both
/// registers share.
fn project(t: &'static Value, key: &str, sha: &str, fields: &[&str]) -> Vec<Value> {
    rows_of(t, key, sha)
        .map(|r| {
            let mut o = serde_json::Map::new();
            for f in fields {
                o.insert((*f).into(), r[*f].clone());
            }
            let units = r["units"].as_str().expect("units");
            let split: Vec<&str> = units.split(',').map(str::trim).collect();
            o.insert("units".into(), json!(split));
            Value::Object(o)
        })
        .collect()
}

/// The named corpus's unit register (CI-gate view — see tables_for).
pub fn units_in(corpus: Option<&str>, sha: &str) -> Vec<Value> {
    project(tables_for(corpus), "relocated_units", sha, &["to"])
}

/// The reviewed source->destination edge layer (M5-1c-iii): one row
/// per (from file, to file) edge with the units that rode it. Units
/// absent from every edge row are arrival-level GT only.
/// The named corpus's edge register (CI-gate view — see tables_for).
pub fn edges_in(corpus: Option<&str>, sha: &str) -> Vec<Value> {
    project(tables_for(corpus), "relocation_edges", sha, &["from", "to"])
}
