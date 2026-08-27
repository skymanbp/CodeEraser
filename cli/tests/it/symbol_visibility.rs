//! The symbols table's two stored words, end to end (plan v2.14 for
//! the visibility word, plan v2.17 L round piece (4) for the
//! convention word): `symbols.flags` was created at 2g and left zero
//! because no producer could fill it without guessing; the fourclass
//! visibility producer fills it now, `symbols.conv` arrived with a
//! producer (mention::conv) from the start, and this leg measures the
//! CHAIN — extractor to the INSERT to the read surface — which is
//! where a column-order slip or a constant-zero regression would hide,
//! and where a unit test on the producer alone would not look.
//!
//! The symbol-edge legs that lived beside this one (the names an
//! import binds, the binding→declaration join) retired with their
//! modules at index schema v14 (plan v2.17 L round piece (1), user
//! ruling: delete; the K10 precision audit stays on record in the
//! plan). This is the stored words' only end-to-end leg, so it stays
//! — and since piece (2) it reads the WHOLE visibility word: a `pub
//! fn` in a private `mod` stores bit 0 without bit 1 (K21), a
//! `pub(crate)` item stores bit 2 on top. What the wire shows of that
//! word is graph_export_surface.rs's question.

use crate::common;
use codeeraser::dedup::{Params, index::Index};
use codeeraser::fourclass::visibility::{VIS_EXPORTED, VIS_RESTRICTED, VIS_SCOPE_EXPORTED};
use codeeraser::graph::symbols::{SymbolRow, symbol_rows};
use codeeraser::mention::conv::Conv;
use std::path::{Path, PathBuf};

/// Write a fixture document out (`common::fixtures::write_doc`, the
/// `--- <path>` document shape) and index it with the real `ce
/// dedup`. One helper, not a setup body per test: two copies of this
/// were a T2 clone by this repo's own gate, which said so on the
/// first draft — and the splitter itself moved to fixtures.rs when a
/// third leg (graph_mounts.rs) needed it.
fn indexed(name: &str, doc: &str) -> PathBuf {
    let dir = common::fixtures::tmp(name);
    common::fixtures::write_doc(&dir, doc);
    common::build_index(&dir);
    dir
}

fn open(dir: &Path) -> Index {
    Index::open(&dir.join(".ce/index.db"), Params::default()).expect("open index")
}

/// Every expected (file, key, word) row is among the rows read back
/// through the symbols read surface, `project` choosing the word —
/// one row per expectation, not one assertion block per file.
fn expect_words(
    dir: &Path,
    project: fn(SymbolRow) -> (String, String, i64),
    want: &[(&str, &str, i64)],
    what: &str,
) {
    let got: Vec<(String, String, i64)> = symbol_rows(&open(dir))
        .expect("symbol rows")
        .into_iter()
        .map(project)
        .collect();
    for (file, key, word) in want {
        assert!(
            got.contains(&((*file).to_string(), (*key).to_string(), *word)),
            "{file}: {key} should store {what} {word}; got {got:?}"
        );
    }
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
    let open = VIS_EXPORTED | VIS_SCOPE_EXPORTED;
    expect_words(
        &indexed("symbol-visibility", VIS_FIXTURE),
        |s| (s.path, s.key, s.vis),
        &[
            ("lib.rs", "open_door/0", open),
            ("lib.rs", "shut_door/0", 0),
            ("lib.rs", "side_door/0", open | VIS_RESTRICTED),
            ("lib.rs", "trapdoor/0", VIS_EXPORTED),
            ("mod.py", "public_call/0", open),
            ("mod.py", "_private_call/0", 0),
        ],
        "visibility",
    );
}

/// One witness per stored AST-half category the two languages can
/// produce, so a column that stopped being written (or was written
/// from the wrong word) shows up as a wrong value, never as a
/// coincidental zero.
const CONV_FIXTURE: &str = "\
--- lib.rs
#[cfg(test)]
pub fn probe() {
    let _ = 1;
}

#[no_mangle]
pub extern \"C\" fn ffi_door() {
    let _ = 2;
}

#[allow(dead_code)]
fn spare() {
    let _ = 3;
}

pub fn plain() {
    let _ = 4;
}
--- app.py
@app.route(\"/\")
def index():
    return 1


class Box:
    def lid(self):
        return 2
";

#[test]
fn indexed_symbols_carry_their_convention_word() {
    expect_words(
        &indexed("symbol-conv", CONV_FIXTURE),
        |s| (s.path, s.key, s.conv),
        &[
            ("lib.rs", "probe/0", Conv::Test.bit()),
            ("lib.rs", "ffi_door/0", Conv::Ffi.bit()),
            ("lib.rs", "spare/0", Conv::Allow.bit()),
            ("lib.rs", "plain/0", 0),
            ("app.py", "index/0", Conv::Registration.bit()),
            ("app.py", "lid/1", Conv::Member.bit()),
            ("app.py", "Box", 0),
        ],
        "convention word",
    );
}
