//! The symbols table's visibility bits, end to end (plan v2.14):
//! `symbols.flags` was created at 2g and left zero because no
//! producer could fill it without guessing; the fourclass visibility
//! producer fills it now, and this leg measures the CHAIN — extractor
//! to the INSERT to the read surface — which is where a column-order
//! slip or a constant-zero regression would hide, and where a unit
//! test on the producer alone would not look.
//!
//! The symbol-edge legs that lived beside this one (the names an
//! import binds, the binding→declaration join) retired with their
//! modules at index schema v14 (plan v2.17 L round piece (1), user
//! ruling: delete; the K10 precision audit stays on record in the
//! plan). This is the stored word's only end-to-end leg, so it stays
//! — and since piece (2) it reads the WHOLE word: a `pub fn` in a
//! private `mod` stores bit 0 without bit 1 (K21), a `pub(crate)`
//! item stores bit 2 on top. What the wire shows of that word is
//! graph_export_surface.rs's question.

use crate::common;
use codeeraser::dedup::{Params, index::Index};
use codeeraser::fourclass::visibility::{VIS_EXPORTED, VIS_RESTRICTED, VIS_SCOPE_EXPORTED};
use codeeraser::graph::symbols::symbol_rows;
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

pub(crate) fn side_door() {
    let _ = 3;
}

mod cellar {
    pub fn trapdoor() {
        let _ = 4;
    }
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
    let got: Vec<(String, String, i64)> =
        rows.into_iter().map(|s| (s.path, s.key, s.vis)).collect();

    // one row per expectation, not one assertion block per file
    let open = VIS_EXPORTED | VIS_SCOPE_EXPORTED;
    let want: &[(&str, &str, i64)] = &[
        ("lib.rs", "open_door/0", open),
        ("lib.rs", "shut_door/0", 0),
        ("lib.rs", "side_door/0", open | VIS_RESTRICTED),
        ("lib.rs", "trapdoor/0", VIS_EXPORTED),
        ("mod.py", "public_call/0", open),
        ("mod.py", "_private_call/0", 0),
    ];
    for (file, key, word) in want {
        assert!(
            got.contains(&((*file).to_string(), (*key).to_string(), *word)),
            "{file}: {key} should store visibility {word}; got {got:?}"
        );
    }
}
