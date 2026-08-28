//! The export surface and the advisory tables from the tree to the
//! wire (plan v2.14 proto 4.1.0; plan v2.17 L round piece (6), proto
//! 6.2.0): a declaration's `pub` becomes a `symbols` row against the
//! graph's dense node identity, and — on the advisory road alone — an
//! unmentioned declaration becomes an `unmentioned` key beside every
//! node's `mounts` row.
//!
//! Three legs, three risks. K6 is the ADR-008 clause the whole symbol
//! slice hangs on — a symbol NAME is a text-shaped thing and
//! text-shaped things never cross the wire — asserted structurally
//! (no string leaf anywhere in the body), on both roads, with the
//! advisory road's two tables required non-empty so the assertion
//! cannot pass vacuously over an absent key. K16 is the legacy
//! contract: the `Advisory::No` body is the five-key request byte for
//! byte, and the advisory road only ADDS keys — including the empty
//! half, where a tree whose every declaration is mentioned sends
//! `unmentioned: []` beside a `mounts` table covering every node. The
//! table leg is the export surface itself — right node, right bits,
//! DEDUPED and MASKED to bit 0.

use crate::common;
use codeeraser::graph::deadcode::{Advisory, GraphWire, request_body};
use serde_json::Value;
use std::path::Path;

/// A crate whose files split every way the table can: one exporting
/// twice (the dedup case) plus a `pub(crate)` item (stored bit 2, a
/// wire row no different from `pub`) plus a private item, one wholly
/// private, one exporting a type only, and the bin root with a `pub
/// fn` inside a private `mod` (stored bit 0 without bit 1, a wire row
/// no different from a top-level `pub`). `shut_door` is spelled by no
/// other file, so the advisory road always has a candidate here.
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

/// Two files that spell each other's every declaration — the private
/// ones included, because the veto has no bit-0 prefilter — so the
/// advisory road has nothing to send and must say so with an empty
/// table, never an absent key (K16 (c1)).
const EVERYTHING_MENTIONED: &str = "\
--- Cargo.toml
[package]
name = \"fixture\"
version = \"0.1.0\"
edition = \"2021\"
--- src/main.rs
mod api;

fn main() {
    api::open_door();
}
// bolt
--- src/api.rs
//! main api
pub fn open_door() {}

fn bolt() {}
";

/// The fixture document indexed by the real walk — the same
/// `--- <path>` document shape symbol_visibility.rs uses, for the same
/// reason (a table of (path, source) pairs is this repo's most-rhyming
/// token shape and its own clone gate says so).
fn indexed(name: &str, doc: &str) -> std::path::PathBuf {
    common::indexed_doc(name, doc)
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

fn wire(dir: &Path, advisory: Advisory) -> GraphWire {
    common::graph_wire(dir, advisory).1
}

fn keys(body: &Value) -> Vec<String> {
    body.as_object()
        .expect("request object")
        .keys()
        .cloned()
        .collect()
}

/// K6: the request body carries integers and nothing else — on the
/// legacy road and on the advisory road, whose two tables must be
/// present and non-empty for the leaf assertion to mean anything.
#[test]
fn no_name_of_any_kind_reaches_the_graph_request() {
    let dir = indexed("export-surface-k6", FIXTURE);
    for advisory in [Advisory::No, Advisory::Yes] {
        let w = wire(&dir, advisory);
        let body = request_body(&w, &[]);
        let mut found = Vec::new();
        string_leaves(&body, &mut found);
        assert!(
            found.is_empty(),
            "text crossed the wire (ADR-008 §5.9.2) under {advisory:?}: {found:?}"
        );
        assert!(!w.symbols.is_empty(), "the fixture must exercise the table");
        if advisory == Advisory::Yes {
            assert!(!body["unmentioned"].as_array().expect("table").is_empty());
            assert!(!body["mounts"].as_array().expect("table").is_empty());
            let names = w.unmentioned.as_ref().expect("names ride beside the keys");
            assert!(
                names.values().flatten().any(|n| n.symbol == "shut_door"),
                "the candidate the fixture guarantees: {names:?}"
            );
        }
    }
}

/// K16: the legacy body is the five keys and nothing else; the
/// advisory road adds exactly two and leaves those five byte for byte
/// (serde_json's Map is ordered, so removing the two keys from the
/// advisory body must give the legacy body back).
#[test]
fn the_legacy_request_is_untouched_by_the_advisory_road() {
    let dir = indexed("export-surface-k16", FIXTURE);
    let legacy = request_body(&wire(&dir, Advisory::No), &[]);
    assert_eq!(keys(&legacy), ["edges", "nodes", "pos", "symbols", "unres"]);
    let mut advised = request_body(&wire(&dir, Advisory::Yes), &[]);
    assert_eq!(
        keys(&advised),
        [
            "edges",
            "mounts",
            "nodes",
            "pos",
            "symbols",
            "unmentioned",
            "unres"
        ]
    );
    let obj = advised.as_object_mut().expect("object");
    obj.remove("mounts");
    obj.remove("unmentioned");
    assert_eq!(
        advised.to_string(),
        legacy.to_string(),
        "five keys, same bytes"
    );
}

/// K16 (c): a tree with nothing unmentioned sends the empty table —
/// the key is present and `[]` — beside a mounts table with one row
/// per node, never fewer.
#[test]
fn an_all_mentioned_tree_sends_an_empty_unmentioned_table_and_full_mounts() {
    let dir = indexed("export-surface-empty", EVERYTHING_MENTIONED);
    let w = wire(&dir, Advisory::Yes);
    let body = request_body(&w, &[]);
    assert_eq!(body["unmentioned"], serde_json::json!([]), "{body}");
    assert_eq!(
        body["mounts"].as_array().expect("mounts").len(),
        body["nodes"].as_array().expect("nodes").len()
    );
}

/// The table itself: one row per (file, MASKED visibility) the tree
/// actually holds, addressed by node index.
#[test]
fn the_symbols_table_names_files_by_index_and_dedupes_them() {
    let dir = indexed("export-surface-table", FIXTURE);
    let w = wire(&dir, Advisory::No);
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
