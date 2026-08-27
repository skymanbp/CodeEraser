//! The export surface from the tree to the wire (plan v2.14, proto
//! 4.1.0): a declaration's `pub` becomes a `symbols` row against the
//! graph's dense node identity, and nothing else about it travels.
//!
//! Two legs, and they measure different risks. The first is K6, the
//! ADR-008 clause the whole symbol slice hangs on — a symbol NAME is
//! a text-shaped thing and text-shaped things never cross the wire.
//! Asserting "no path appears in the body" would only pin the paths
//! this fixture happens to hold, so the assertion is structural: no
//! leaf of the request is a string at all. The second is the table
//! itself — right node, right bits, DEDUPED (two public functions in
//! one file are not two facts about that file), and MASKED: the
//! stored word carries scope and restriction bits since plan v2.17
//! piece (2), and the wire shows bit 0 of it and nothing else.

use crate::common;
use codeeraser::dedup::{Params, index::Index};
use codeeraser::graph::deadcode;
use serde_json::{Value, json};
use std::path::Path;

/// A crate whose files split every way the table can: one exporting
/// twice (the dedup case) plus a `pub(crate)` item (stored bit 2, a
/// wire row no different from `pub`) plus a private item, one wholly
/// private, one exporting a type only, and the bin root with a `pub
/// fn` inside a private `mod` (stored bit 0 without bit 1, a wire row
/// no different from a top-level `pub`).
const FIXTURE: &str = "\
--- Cargo.toml
[package]
name = \"fixture\"
version = \"0.1.0\"
edition = \"2021\"
--- src/api.rs
pub fn open_door() {
    let _ = 1;
}

pub fn shut_door() {
    let _ = 2;
}

pub(crate) fn side_door() {
    let _ = 5;
}

fn bolt() {
    let _ = 3;
}
--- src/inner.rs
fn helper() {
    let _ = 4;
}
--- src/kind.rs
pub struct Gate {
    pub n: u8,
}
--- src/main.rs
mod api;
mod inner;
mod kind;

mod cellar {
    pub fn trapdoor() {
        let _ = 6;
    }
}

fn main() {
    api::open_door();
    let _ = kind::Gate { n: 0 };
}
";

/// Write the fixture document out and index it with the real walk —
/// the same `--- <path>` document shape symbol_visibility.rs uses,
/// for the same reason (a table of (path, source) pairs is this
/// repo's most-rhyming token shape and its own clone gate says so).
fn indexed(name: &str) -> std::path::PathBuf {
    let dir = common::fixtures::tmp(name);
    let mut current: Option<String> = None;
    let mut body = String::new();
    let flush = |rel: &Option<String>, body: &mut String| {
        if let Some(rel) = rel {
            let path = dir.join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("mkdir fixture subdir");
            }
            std::fs::write(&path, &body).expect("write fixture file");
        }
        body.clear();
    };
    for line in FIXTURE.lines() {
        match line.strip_prefix("--- ") {
            Some(next) => {
                flush(&current, &mut body);
                current = Some(next.trim().to_string());
            }
            None => {
                body.push_str(line);
                body.push('\n');
            }
        }
    }
    flush(&current, &mut body);
    common::build_index(&dir);
    dir
}

/// Every string leaf under `v`, keys excluded — an object's KEYS are
/// the schema and always text; its VALUES are the facts, and a fact
/// that is text is a name that escaped.
fn string_leaves(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(items) => items.iter().for_each(|i| string_leaves(i, out)),
        Value::Object(map) => map.values().for_each(|i| string_leaves(i, out)),
        _ => {}
    }
}

fn wire(dir: &Path) -> deadcode::GraphWire {
    let db = dir.join(".ce/index.db");
    let idx = Index::open(&db, Params::default()).expect("open index");
    deadcode::wire_of(dir, &idx, &db).expect("graph wire")
}

/// K6: the request body carries integers and nothing else. Built the
/// way `judge` builds it, so a key added there without thinking lands
/// in this assertion rather than in a release.
#[test]
fn no_name_of_any_kind_reaches_the_graph_request() {
    let w = wire(&indexed("export-surface-k6"));
    let body = json!({
        "nodes": w.rows,
        "edges": w.edges.iter().collect::<Vec<_>>(),
        "pos": Vec::<i64>::new(),
        "unres": w.unres,
        "symbols": w.symbols.iter().collect::<Vec<_>>(),
    });
    let mut found = Vec::new();
    string_leaves(&body, &mut found);
    assert!(
        found.is_empty(),
        "text crossed the wire (ADR-008 §5.9.2): {found:?}"
    );
    assert!(!w.symbols.is_empty(), "the fixture must exercise the table");
}

/// The table itself: one row per (file, MASKED visibility) the tree
/// actually holds, addressed by node index.
#[test]
fn the_symbols_table_names_files_by_index_and_dedupes_them() {
    let dir = indexed("export-surface-table");
    let w = wire(&dir);
    let at = |path: &str| {
        w.nodes
            .iter()
            .position(|n| n.path == path && n.unit.is_empty())
            .unwrap_or_else(|| panic!("{path} is not a node")) as i64
    };
    let got: Vec<[i64; 2]> = w.symbols.iter().copied().collect();

    // api.rs exports twice, once `pub(crate)`, and hides once: ONE
    // exported row (deduped across `pub` and `pub(crate)` alike —
    // stored words 3 and 7 both project to 1) beside its private row.
    // inner.rs is private only; kind.rs exports a type — a declaration
    // like any other; main.rs hides `main` and exports `trapdoor` from
    // a private mod (stored 1, bit 1 clear — the same wire row a
    // top-level `pub` would give).
    let mut want = vec![
        [at("src/api.rs"), 0],
        [at("src/api.rs"), 1],
        [at("src/inner.rs"), 0],
        [at("src/kind.rs"), 1],
        [at("src/main.rs"), 0],
        [at("src/main.rs"), 1],
    ];
    want.sort_unstable(); // node ids follow path order; the set does not
    assert_eq!(got, want, "export surface");
}
