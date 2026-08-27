//! The mounts table from the tree to its rows (plan v2.17 L round
//! piece (5), sealed criterion §4 — K30's producer half): every fact
//! the core will fold into `mountedPrivate` / `pkgPrivate` is read here
//! from a real index, so a wrong join (a `mod` unit's visibility
//! matched on the wrong line), a missed star export, or a bin root
//! misread as a lib root lands in this leg and not in an advisory.
//!
//! The K30 matrix, one cell per file: a bin root's private `mod` chain
//! (`tool.rs`, the main_cli.rs shape with `pub(crate)` inside — the
//! content's visibility is not the mount's), the lib root with zero
//! mounts, the lib's `pub mod` and a private one, a bin root's `pub
//! mod` (`shown.rs` — the §6 residual, still `[0,1,0]` here and code 0
//! in the core), a DOUBLE mount (`open.rs`: the lib's `pub mod` plus
//! main's private `#[path]` mount), and a re-export target
//! (`source.rs`: a private mount whose `Thing` the lib façade
//! re-exports and the bin root reaches through `use fixture::Thing` —
//! the package-name rung lands on the façade with `Thing` unconsumed,
//! the R5 hop re-walks the façade's `pub use crate::source::Thing`
//! and the edge carries `via_reexport`). The hop needs two things the
//! ladder has (rs_use::bound, rs_reexport::binds_to): a walk that
//! ended with segments left over at ONE terminal, and a façade entry
//! spelled `crate::`/`self::`/`super::`. What it does not have, and
//! the self index shows (0 `via_reexport` edges): a glob façade
//! (never followed), a uniform-path façade (`pub use source::Thing` —
//! the bare head is read as a crate name by R4 and stops; 26 such
//! `use` sites in cli/src have no edge), and an importer whose walk
//! ends AT the roots of a lib+bin package from a non-root module
//! (`use crate::Thing` / `use crate::{…}` — two covering roots, two
//! terminals, AmbiguousRoot before any hop; a `crate::facade::Thing`
//! path binds fine because both roots descend to the same file). A
//! recall limit of the RUNG, named in the CHANGELOG, not of this
//! table. Beside Rust: a lib-less Cargo package, Go's `package main`
//! and `internal/`, TS `export *` / `export * as`, the two cabal
//! facts, and a markdown file whose anchor link mints a SECTION node
//! sharing the file's path (the row map below must not collapse the
//! two). A phantom file node — an edge target nothing walked — is
//! not producible here: every rung resolves against the walked set
//! (ladder/md.rs:75), so that shape is index skew and the unit leg
//! witnesses its `[0,0,0]` row; `facts` keys itself by the walked set
//! it reads from the index, never by node paths, so an `internal/`
//! path on such a node could not earn it a bit either way.

use crate::common;
use codeeraser::dedup::{Params, index::Index};
use codeeraser::graph::deadcode;
use codeeraser::graph::mounts::{self, MOUNT_PKG_PRIVATE, MOUNT_REEXPORTED};
use codeeraser::graph::wire::GRAN_SECTION;
use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = "\
--- Cargo.toml
[package]
name = \"fixture\"
version = \"0.1.0\"
edition = \"2021\"
--- src/lib.rs
mod hidden;
pub mod open;
mod source;
pub use crate::source::Thing;
--- src/hidden.rs
pub fn h() {}
--- src/open.rs
pub fn o() {}
--- src/source.rs
pub struct Thing;
--- src/main.rs
mod tool;
pub mod shown;
#[path = \"open.rs\"]
mod twice;

use fixture::Thing;

fn main() {
    let _ = Thing;
    tool::t();
    shown::s();
    twice::o();
}
--- src/tool.rs
pub(crate) fn t() {}
--- src/shown.rs
pub fn s() {}
--- src/bin/extra.rs
fn main() {}
--- tools/Cargo.toml
[package]
name = \"tools\"
version = \"0.1.0\"
--- tools/src/main.rs
mod util;

fn main() {
    util::v();
}
--- tools/src/util.rs
pub fn v() {}
--- go/main.go
package main

func main() {}
--- go/internal/x/x.go
package x
--- go/lib/lib.go
package lib
--- ts/index.ts
export * from './all';
export * as ns from './space';
export { a } from './named';
import './use';
--- ts/all.ts
export const all = 1;
--- ts/space.ts
export const sp = 1;
--- ts/named.ts
export const a = 1;
--- ts/use.ts
export const used = 1;
--- hs/x.cabal
executable x
  hs-source-dirs: app
  main-is: Main.hs
  other-modules: Lib
--- hs/app/Main.hs
module Main where

import Lib

main :: IO ()
main = run
--- hs/app/Lib.hs
module Lib (run) where

run :: IO ()
run = pure ()
--- hs2/y.cabal
library
  hs-source-dirs: src
  exposed-modules: A
  other-modules: B
--- hs2/src/A.hs
module A where

import B
--- hs2/src/B.hs
module B where
--- docs/note.md
# Intro

See [intro](./note.md#intro).
";

/// Expected `path privateMounts totalMounts bits` per file, one line
/// per K30 cell, bits spelled as a set of letters — `R` (re-export
/// target), `P` (package private), `-` (none). A text table rather
/// than a tuple table: a run of `(path, [i64; 3])` tuples is this
/// repo's most-rhyming token shape, and its own clone gate matched
/// the first draft against itself.
const WANT: &str = "\
src/lib.rs          0 0 -
src/hidden.rs       1 1 -
src/open.rs         1 2 -
src/source.rs       1 1 R
src/main.rs         0 0 P
src/tool.rs         1 1 -
src/shown.rs        0 1 -
src/bin/extra.rs    0 0 P
tools/src/main.rs   0 0 P
tools/src/util.rs   1 1 P
go/main.go          0 0 P
go/internal/x/x.go  0 0 P
go/lib/lib.go       0 0 -
ts/index.ts         0 0 -
ts/all.ts           0 0 R
ts/space.ts         0 0 R
ts/named.ts         0 0 -
ts/use.ts           0 0 -
hs/app/Main.hs      0 0 P
hs/app/Lib.hs       0 0 P
hs2/src/A.hs        0 0 -
hs2/src/B.hs        0 0 P
docs/note.md        0 0 -
";

/// One WANT line → (path, row).
fn want_row(line: &str) -> (&str, [i64; 3]) {
    let mut w = line.split_whitespace();
    let path = w.next().expect("path");
    let num = |w: &mut std::str::SplitWhitespace| w.next().expect("field").parse().expect("i64");
    let (private, total) = (num(&mut w), num(&mut w));
    let bits = w
        .next()
        .expect("bits")
        .chars()
        .map(|c| match c {
            'R' => MOUNT_REEXPORTED,
            'P' => MOUNT_PKG_PRIVATE,
            _ => 0,
        })
        .fold(0, |acc, bit| acc | bit);
    (path, [private, total, bits])
}

#[test]
fn every_k30_cell_reads_its_row_from_a_real_index() {
    let dir = common::fixtures::tmp("graph-mounts-k30");
    common::fixtures::write_doc(&dir, FIXTURE);
    common::build_index(&dir);
    let db = dir.join(".ce/index.db");
    let idx = Index::open(&db, Params::default()).expect("open index");
    let w = deadcode::wire_of(&dir, &idx, &db).expect("graph wire");
    let facts = mounts::facts(&dir, &idx).expect("mount facts");
    let rows = mounts::mount_rows(&w.nodes, &facts);
    assert_eq!(rows.len(), w.nodes.len(), "one row per node");
    let kinds: BTreeSet<i64> = w.nodes.iter().map(|n| n.kind).collect();
    assert!(
        kinds.contains(&GRAN_SECTION),
        "the markdown anchor must mint a section node beside its file node"
    );
    // file nodes only — a section node shares its file's path and
    // would otherwise overwrite the file's row in a path-keyed map
    let by_path: BTreeMap<&str, [i64; 3]> = deadcode::file_nodes(&w)
        .into_iter()
        .map(|(id, path)| (path, rows[&id]))
        .collect();
    let want: BTreeMap<&str, [i64; 3]> = WANT.lines().map(want_row).collect();
    for (path, row) in &want {
        assert_eq!(by_path.get(path), Some(row), "{path}");
    }
    let unexpected: Vec<&str> = by_path
        .iter()
        .filter(|(p, row)| **row != [0, 0, 0] && !want.contains_key(*p))
        .map(|(p, _)| *p)
        .collect();
    assert!(
        unexpected.is_empty(),
        "facts on unlisted files: {unexpected:?}"
    );
}
