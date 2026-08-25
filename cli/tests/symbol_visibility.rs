//! The symbol edge, end to end (plan v2.14): its two inputs and the
//! join that turns them into a reference. Both inputs were
//! reserved-or-absent until this round: `symbols.flags` was created
//! at 2g and left zero because no producer could fill it without
//! guessing, and no table held the names an import binds at all.
//!
//! Why integration legs and not more unit tests: the unit tests
//! measure the FUNCTIONS. These measure the CHAIN — extractor to the
//! INSERT to the read surface to the lookup — which is where a
//! forgotten binding, a column-order slip, or a fixture the resolver
//! cannot anchor would hide. The last one is not hypothetical: the
//! binding leg's first draft passed vacuously over an empty edge
//! table.

mod common;

use codeeraser::dedup::{Params, index::Index};
use codeeraser::fourclass::visibility::VIS_EXPORTED;
use codeeraser::graph::load::binding_edges;
use codeeraser::graph::symbols::symbol_rows;
use codeeraser::graph::symedges::symbol_edges;
use std::path::{Path, PathBuf};

/// Split a fixture document into its files: a line `--- <path>` opens
/// one, and everything until the next such line is its body.
///
/// One document per fixture rather than a table of (path, source)
/// pairs: the pair table is this repo's most-rhyming token shape —
/// its own clone gate matched a six-entry one against two unrelated
/// fixture tables — and a document also reads as the tree it makes.
fn files_of(doc: &str) -> Vec<(&str, String)> {
    let mut out: Vec<(&str, String)> = Vec::new();
    for line in doc.lines() {
        match line.strip_prefix("--- ") {
            Some(path) => out.push((path.trim(), String::new())),
            None => {
                if let Some((_, body)) = out.last_mut() {
                    body.push_str(line);
                    body.push('\n');
                }
            }
        }
    }
    out
}

/// Write a fixture document out and index it with the real `ce
/// dedup`. One helper, not a setup body per test: two copies of this
/// were a T2 clone by this repo's own gate, which said so on the
/// first draft.
fn indexed(name: &str, doc: &str) -> PathBuf {
    let dir = common::fixtures::tmp(name);
    for (rel, body) in files_of(doc) {
        if let Some(parent) = Path::new(rel)
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
        {
            std::fs::create_dir_all(dir.join(parent)).expect("mkdir fixture subdir");
        }
        std::fs::write(dir.join(rel), body).expect("write fixture file");
    }
    common::build_index(&dir);
    dir
}

fn open(dir: &Path) -> Index {
    Index::open(&dir.join(".ce/index.db"), Params::default()).expect("open index")
}

const VIS_FIXTURE: &str = "\
--- lib.rs
pub fn open_door() {
    let _ = 1;
}

fn shut_door() {
    let _ = 2;
}
--- mod.py
def public_call():
    return 1


def _private_call():
    return 2
";

#[test]
fn indexed_symbols_carry_their_declared_visibility() {
    let rows = symbol_rows(&open(&indexed("symbol-visibility", VIS_FIXTURE))).expect("symbol rows");
    let got: Vec<(String, String, bool)> = rows
        .into_iter()
        .map(|s| (s.path, s.key, s.vis & VIS_EXPORTED != 0))
        .collect();

    // one row per expectation, not one assertion block per file
    let want: &[(&str, &str, bool)] = &[
        ("lib.rs", "open_door/0", true),
        ("lib.rs", "shut_door/0", false),
        ("mod.py", "public_call/0", true),
        ("mod.py", "_private_call/0", false),
    ];
    for (file, key, exported) in want {
        assert!(
            got.contains(&((*file).to_string(), (*key).to_string(), *exported)),
            "{file}: {key} should read exported={exported}; got {got:?}"
        );
    }
}

/// The fixture needs BOTH the src/ layout and a manifest: the ladder
/// anchors on the crate-root conventions, and it learns that
/// src/main.rs IS a root from Cargo.toml (graph/cargo.rs). Without
/// either, both sites go unresolved and the edge table stays empty —
/// which is exactly what this leg's first draft measured.
const BIND_FIXTURE: &str = "\
--- Cargo.toml
[package]
name = \"fixture\"
version = \"0.1.0\"
edition = \"2021\"
--- src/helpers.rs
pub fn open_door() {
    let _ = 1;
}

pub fn shut_door() {
    let _ = 2;
}
--- src/main.rs
mod helpers;
use crate::helpers::{open_door, shut_door as bolt};

fn main() {
    open_door();
    bolt();
}
";

/// The other half of the symbol-edge input, over the same chain:
/// detection reads the bound names, the store persists them under
/// their site, and the read surface joins them to the edge the ladder
/// resolved. Nothing here decides whether a name is a declaration —
/// that is the lookup's job — so the assertion is on the CANDIDATES.
#[test]
fn indexed_sites_carry_their_import_bindings() {
    let got: Vec<(String, String, String)> =
        binding_edges(&open(&indexed("import-bindings", BIND_FIXTURE)))
            .expect("binding edges")
            .into_iter()
            .filter(|b| b.src == "src/main.rs")
            .map(|b| (b.dst_path, b.local, b.target))
            .collect();

    let want: &[(&str, &str, &str)] = &[
        // a plain use names the same name on both sides
        ("src/helpers.rs", "open_door", "open_door"),
        // an alias splits local from target
        ("src/helpers.rs", "bolt", "shut_door"),
    ];
    for (dst, local, target) in want {
        assert!(
            got.contains(&(
                (*dst).to_string(),
                (*local).to_string(),
                (*target).to_string()
            )),
            "{local} -> {target} in {dst} missing; got {got:?}"
        );
    }
}

/// The corpus the join is measured on. Two files carry the refusals:
/// `other.rs` declares the same name at a DIFFERENT arity — so a
/// lookup that ignored which file the ladder resolved to shows up as
/// an extra row instead of deduplicating into silence — and
/// `refuser.rs` both binds the MODULE (which declares no such name)
/// and speaks `open_door` through a path, binding nothing.
const EDGE_FIXTURE: &str = "\
--- Cargo.toml
[package]
name = \"fixture\"
version = \"0.1.0\"
edition = \"2021\"
--- src/helpers.rs
pub struct Gate {
    pub n: u8,
}

pub fn open_door() {
    let _ = 1;
}

pub fn shut_door() {
    let _ = 2;
}
--- src/other.rs
pub fn open_door(x: u8) {
    let _ = x;
}
--- src/refuser.rs
use crate::helpers;

pub fn ring() {
    crate::helpers::open_door();
}
--- src/main.rs
mod helpers;
mod other;
mod refuser;
use crate::helpers::{Gate, open_door, shut_door as bolt};

fn main() {
    open_door();
    bolt();
    let _ = Gate { n: 0 };
}
";

/// The join, and the ways it must REFUSE. One fixture, because every
/// refusal is a statement about the same corpus: a name that is bound
/// resolves, a name that is merely spoken does not, and a name that
/// matches a declaration in some OTHER file does not either — that
/// last one is R6's grave, and it has to be a fixture, not a comment.
#[test]
fn a_symbol_edge_needs_a_binding_that_hits_a_declaration() {
    let idx = open(&indexed("symbol-edges", EDGE_FIXTURE));
    let got: Vec<(String, String, String)> = symbol_edges(&idx)
        .expect("symbol edges")
        .into_iter()
        .map(|e| (e.src, e.dst_path, e.key))
        .collect();

    // exactly these: the alias resolves to the TARGET-side key, and
    // the struct rides its bare key (units.rs keys extra kinds
    // without an arity)
    let want: Vec<(String, String, String)> = [
        ("src/main.rs", "src/helpers.rs", "Gate"),
        ("src/main.rs", "src/helpers.rs", "open_door/0"),
        ("src/main.rs", "src/helpers.rs", "shut_door/0"),
    ]
    .iter()
    .map(|(s, d, k)| ((*s).to_string(), (*d).to_string(), (*k).to_string()))
    .collect();
    assert_eq!(got, want, "symbol edges");

    // and the module binding was REFUSED BY THE LOOKUP, not dropped
    // by the extractor: the candidate is on the table, and its file
    // edge is what stands in the graph
    let candidates: Vec<(String, String)> = binding_edges(&idx)
        .expect("binding edges")
        .into_iter()
        .filter(|b| b.src == "src/refuser.rs")
        .map(|b| (b.dst_path, b.target))
        .collect();
    assert_eq!(
        candidates,
        vec![("src/helpers.rs".to_string(), "helpers".to_string())],
        "the module candidate must survive to the lookup"
    );
}
