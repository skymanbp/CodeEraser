//! Entry-role measurement for file nodes (split from deadcode.rs at
//! the 300-line dogfood wall). Since proto 2.28.0 (batch-7 slice 3
//! main body) this side measures ROLE FACTS — named main, executable
//! dir, test convention, entry glob, doc entry, allow claim, declared
//! build target, compilation unit, asset — and the entry DECISION is the core's role table
//! (CE.Graph.Cost.roleBits). The pre-2.28 legacy flags column this
//! module also produced retired at 5.0.0, once the symbols table
//! gave visibility a producer; nothing here measures bit 0, because
//! bit 0 is not a file fact — public-ness is a symbol fact (3l
//! re-review), and it now reaches the core through the export
//! surface (graph/symwire.rs) rather than through this column.

use super::targets::Declared;
use crate::graph::roots;
use crate::mention::conv::name::runner_test;
use crate::mention::conv::protocol::listed;
use crate::scan::globs::{self, Inclusions};
use crate::scan::lang::Lang;
use std::path::Path;

/// Frozen role-bit positions (wire node row column 4). Facts, never
/// verdicts: which entry bits each role lands on is the core's
/// roleBits table, where an ablation can perturb it.
pub(super) const ROLE_ENTRY_NAMED: i64 = 1;
const ROLE_ENTRY_DIR: i64 = 1 << 1;
const ROLE_TEST: i64 = 1 << 2;
const ROLE_GLOB: i64 = 1 << 3;
const ROLE_DOC: i64 = 1 << 4;
pub(super) const ROLE_ALLOW: i64 = 1 << 5;
const ROLE_DECLARED: i64 = 1 << 6;
/// A declared submodule's node (plan v2.18 step #12, wire 6.3.0): a
/// READER of this tree, never its candidate — the core lands it on
/// the entry bits, and this side measures none of its other roles
/// (they would only cost the reads).
pub(super) const ROLE_FOREIGN: i64 = 1 << 7;
/// A compilation unit (plan v2.30 step 2, register D18): a `.c` /
/// `.cc` / `.cpp` / `.cxx` file is never included by anything — the
/// build compiles it on its own — so without a role of its own every
/// one would be a dead candidate the moment its tree names no main.
/// The core lands it on the executable bit (roleBits row 8, 7.2.0).
const ROLE_UNIT: i64 = 1 << 8;
/// A walked asset (plan v2.30 step 5): a file the index holds no
/// parse of, named by a page — nodes.rs marks it, and this side
/// measures no other role on it (an image is nobody's main, and the
/// allow-claim read would only cost the bytes). The core lands it on
/// the dyn-referenced bit (roleBits row 9): the references the graph
/// cannot read — a stylesheet's `url()`, a script's fetch, a
/// manifest's icons — keep it alive, so it is never a candidate.
pub(super) const ROLE_ASSET: i64 = 1 << 9;
/// A tool convention's file (lane v235_licence, part 4A): a file a tool
/// finds by where it sits (convention.rs, the core's rows). The core
/// lands it on the entry bit (roleBits row 10, 9.1.0).
const ROLE_CONVENTION: i64 = 1 << 10;
/// Each role's name, by bit position (ROLE_ENTRY_NAMED bit 0 …
/// ROLE_CONVENTION bit 10) — the word the query family labels a node's
/// role with (query::legend), so every face names one reason.
pub(crate) const ROLE_NAMES: [&str; 11] = [
    "entry_named",
    "entry_dir",
    "test",
    "glob",
    "doc",
    "allow",
    "declared",
    "foreign",
    "unit",
    "asset",
    "convention",
];

/// Files a runtime or a build starts by their own name, which nothing
/// imports: Rust's `main.rs` and `build.rs`, Go's `main.go`, Python's
/// `__main__.py`, cabal's executable main-is `Main.hs`, the C family's
/// `main` file, the class a Java launcher names, what LÖVE runs
/// (`main.lua`, and `conf.lua` before it) and what Shiny's `runApp`
/// reads from an app directory (`app.R`, or `ui.R` and `server.R`, and
/// `global.R`), and the pages a web server serves by name — a
/// directory's `index.html`, the not-found page `404.html` (plan v2.30
/// step 5). Neovim's root `init.lua` is a tool convention since part 4A
/// (convention.rs). The names are the core's since plan v2.32 step 2 (CE.Lang.Common.Graph
/// `flags.entry_names`, read off `tables/1`).
fn entry_names() -> &'static [&'static str] {
    crate::tables::get().flags.entry_names
}

/// Role facts of one file node; the declared-target role covers the
/// manifests' OWN declarations beside the name and place conventions.
pub(super) fn roles_of(root: &Path, path: &str, entries: &Inclusions, declared: &Declared) -> i64 {
    let base = path.rsplit('/').next().unwrap_or(path);
    let mut r = 0i64;
    if listed(entry_names(), base) {
        r |= ROLE_ENTRY_NAMED;
    }
    if super::convention::conventional(path) {
        r |= ROLE_CONVENTION;
    }
    // one table with the compile database's coverage (compdb_find.rs)
    if crate::graph::compdb_find::is_unit(base) {
        r |= ROLE_UNIT;
    }
    if entry_dir(path, declared) {
        r |= ROLE_ENTRY_DIR;
    }
    if is_test(path, base) {
        r |= ROLE_TEST;
    }
    if globs::selected(entries, path) {
        r |= ROLE_GLOB;
    }
    if doc_entry(path, base) {
        r |= ROLE_DOC;
    }
    if allow_claim(root, path) {
        r |= ROLE_ALLOW;
    }
    if declared.hit(path) {
        r |= ROLE_DECLARED;
    }
    r
}

/// Whether a row of the entry directories for the file's language, or
/// for every language (`*`), holds a prefix of the path — R's rows
/// under each package root, the others under the tree root. The rows
/// are the core's (CE.Lang.Common.Graph `flags.entry_dirs`): Cargo's
/// `src/bin/` `examples/` `benches/` and Go's `cmd/`, the Neovim runtime
/// directories a Lua file is sourced from by path, the R package
/// directories whose scripts R and its tools run by path.
fn entry_dir(path: &str, declared: &Declared) -> bool {
    let lang = Lang::judged_path(Path::new(path)).map_or("", Lang::name);
    crate::tables::get()
        .flags
        .entry_dirs
        .iter()
        .filter(|(scope, _)| *scope == "*" || *scope == lang)
        .any(|&(scope, dirs)| {
            let bases: Vec<&str> = match scope {
                "r" => declared.packages().collect(),
                _ => vec![""],
            };
            dirs.iter().any(|dir| {
                bases
                    .iter()
                    .any(|b| path.starts_with(&roots::join_dir(b, dir)))
            })
        })
}

/// `ce:allow(deadcode) -- <why>` anywhere in the file claims
/// liveness — the one claim grammar (crate::allow: a bare marker
/// claims NOTHING). An unreadable file makes no claim. Full content
/// scan per file node per run; the index already read every byte
/// upstream, so the pages are warm.
fn allow_claim(root: &Path, path: &str) -> bool {
    std::fs::read_to_string(root.join(path))
        .is_ok_and(|text| crate::allow::allow_claim(&text, "ce:allow(deadcode)"))
}

/// A documentation entry the liveness role keeps: a basename of the
/// package's `doc_names` anywhere, or of a `doc_dirs` row under its
/// directory (CE.Lang.Common.PathForms).
fn doc_entry(path: &str, base: &str) -> bool {
    let flags = &crate::tables::get().flags;
    listed(flags.doc_names, base)
        || flags
            .doc_dirs
            .iter()
            .any(|&(dir, names)| path.starts_with(dir) && listed(names, base))
}

/// A test file: a basename a test runner discovers — the package's
/// `runner_tests`, the one table the convention word reads too
/// (runner_test) — or a path component of `test_dirs_role`. The
/// directory row is this reader's own and narrower than the convention
/// word's: a directory name is a project's habit, not a runner's rule
/// (CE.Lang.Common.PathForms).
fn is_test(path: &str, base: &str) -> bool {
    let dirs = crate::tables::get().flags.test_dirs_role;
    runner_test(base) || path.split('/').rev().skip(1).any(|c| listed(dirs, c))
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/deadcode/flags.rs"]
mod tests;
