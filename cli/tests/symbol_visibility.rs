//! The two symbol-edge inputs, end to end (plan v2.14). Both were
//! reserved-or-absent until this round: `symbols.flags` was created
//! at 2g and left zero because no producer could fill it without
//! guessing, and no table held the names an import binds at all.
//!
//! Why integration legs and not more unit tests: the unit tests
//! measure the FUNCTIONS. These measure the CHAIN — extractor to the
//! INSERT to the read surface — which is where a forgotten binding,
//! a column-order slip, or a fixture the resolver cannot anchor would
//! hide. The last one is not hypothetical: the binding leg's first
//! draft passed vacuously over an empty edge table.

mod common;

use codeeraser::dedup::{Params, index::Index};
use codeeraser::fourclass::visibility::VIS_EXPORTED;
use codeeraser::graph::load::binding_edges;
use codeeraser::graph::symbols::symbol_rows;
use std::path::{Path, PathBuf};

/// Write a fixture tree and index it with the real `ce dedup`. One
/// helper, not a setup body per test: two copies of this were a T2
/// clone by this repo's own gate, which said so on the first draft.
fn indexed(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = common::fixtures::tmp(name);
    for (rel, body) in files {
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

#[test]
fn indexed_symbols_carry_their_declared_visibility() {
    let dir = indexed(
        "symbol-visibility",
        &[
            (
                "lib.rs",
                "pub fn open_door() {\n    let _ = 1;\n}\n\nfn shut_door() {\n    let _ = 2;\n}\n",
            ),
            (
                "mod.py",
                "def public_call():\n    return 1\n\n\ndef _private_call():\n    return 2\n",
            ),
        ],
    );
    let rows = symbol_rows(&open(&dir)).expect("symbol rows");
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

/// The other half of the symbol-edge input, over the same chain:
/// detection reads the bound names, the store persists them under
/// their site, and the read surface joins them to the edge the ladder
/// resolved. Nothing here decides whether a name is a declaration —
/// that is the lookup's job — so the assertion is on the CANDIDATES.
#[test]
fn indexed_sites_carry_their_import_bindings() {
    // The fixture needs BOTH the src/ layout and a manifest: the
    // ladder anchors on the crate-root conventions, and it learns
    // that src/main.rs IS a root from Cargo.toml (graph/cargo.rs).
    // Without either, both sites go unresolved and the edge table
    // stays empty — which is exactly what the first draft measured.
    let dir = indexed(
        "import-bindings",
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (
                "src/helpers.rs",
                "pub fn open_door() {\n    let _ = 1;\n}\n\npub fn shut_door() {\n    let _ = 2;\n}\n",
            ),
            (
                "src/main.rs",
                "mod helpers;\nuse crate::helpers::{open_door, shut_door as bolt};\n\
                 fn main() {\n    open_door();\n    bolt();\n}\n",
            ),
        ],
    );
    let got: Vec<(String, String, String)> = binding_edges(&open(&dir))
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
