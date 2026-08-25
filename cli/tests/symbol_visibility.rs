//! The `symbols.flags` column, end to end (plan v2.14). The column
//! was created at 2g and left reserved-and-zero because no producer
//! existed that could fill it without guessing; fourclass::visibility
//! is that producer, and this leg proves the bits survive the real
//! indexing path — not just the extractor's unit tests.
//!
//! Why an integration leg and not another unit test: the unit tests
//! measure the FUNCTION. This measures the CHAIN — extractor to
//! `Unit.vis` to the INSERT to the read surface — which is where a
//! forgotten binding or a column-order slip would hide.

mod common;

use codeeraser::dedup::{Params, index::Index};
use codeeraser::fourclass::visibility::VIS_EXPORTED;
use codeeraser::graph::symbols::symbol_rows;

/// Every symbol of `file`, paired with whether its exported bit is set.
fn exported_bits(dir: &std::path::Path, file: &str) -> Vec<(String, bool)> {
    let idx = Index::open(&dir.join(".ce/index.db"), Params::default()).expect("open index");
    symbol_rows(&idx)
        .expect("symbol rows")
        .into_iter()
        .filter(|s| s.path == file)
        .map(|s| (s.key.clone(), s.vis & VIS_EXPORTED != 0))
        .collect()
}

#[test]
fn indexed_symbols_carry_their_declared_visibility() {
    let dir = common::fixtures::tmp("symbol-visibility");
    std::fs::write(
        dir.join("lib.rs"),
        "pub fn open_door() {\n    let _ = 1;\n}\n\nfn shut_door() {\n    let _ = 2;\n}\n",
    )
    .expect("write rust");
    std::fs::write(
        dir.join("mod.py"),
        "def public_call():\n    return 1\n\n\ndef _private_call():\n    return 2\n",
    )
    .expect("write python");

    common::build_index(&dir);

    // one row per expectation, not one assertion block per file: a
    // per-file assertion body is a T2 clone chain by this repo's own
    // measure, and the guard hook said so on the first draft
    let want: &[(&str, &str, bool)] = &[
        ("lib.rs", "open_door/0", true),
        ("lib.rs", "shut_door/0", false),
        ("mod.py", "public_call/0", true),
        ("mod.py", "_private_call/0", false),
    ];
    for (file, key, exported) in want {
        let got = exported_bits(&dir, file);
        assert!(
            got.contains(&((*key).to_string(), *exported)),
            "{file}: {key} should read exported={exported}; got {got:?}"
        );
    }
}
