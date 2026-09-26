//! Entry-role measurement for file nodes (split from deadcode.rs at
//! the 300-line dogfood wall). Since proto 2.28.0 (batch-7 slice 3
//! main body) this side measures ROLE FACTS — named main, executable
//! dir, test convention, entry glob, doc entry, allow claim, declared
//! build target — and the entry DECISION is the core's role table
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

/// Files a runtime or a build starts by their own name, which nothing
/// imports: Rust's `main.rs` and `build.rs`, Go's `main.go`, Python's
/// `__main__.py`, cabal's executable main-is `Main.hs`, the C family's
/// `main` file, the class a Java launcher names, what LÖVE runs
/// (`main.lua`, and `conf.lua` before it) and what Shiny's `runApp`
/// reads from an app directory (`app.R`, or `ui.R` and `server.R`, and
/// `global.R`), and the pages a web server serves by name — a
/// directory's `index.html`, the not-found page `404.html` (plan v2.30
/// step 5). Neovim's `init.lua` is one only at the root, where a
/// config keeps it: anywhere else the name is a module's own file
/// (`require "a"` reads `a/init.lua`), which the graph reaches.
const ENTRY_NAMES: &str = "main.rs build.rs main.go __main__.py Main.hs main.c main.cc \
                           main.cpp Main.java main.lua conf.lua app.R ui.R server.R global.R \
                           index.html 404.html";

/// Directories whose files a runtime starts by where they sit, never
/// by an import — one row per language (`*` every judged one), each
/// under the tree root except R's, which sit under each package root
/// (a DESCRIPTION's directory, targets.rs): Cargo's `src/bin/`
/// `examples/` `benches/` and Go's
/// `cmd/`; the Neovim runtime directories a Lua file is sourced from by
/// path (`plugin/` at startup; `ftplugin/` `indent/` `syntax/`
/// `colors/` `compiler/` `ftdetect/` `lsp/` on demand; `after/`
/// holding the same — `autoload/` is Vim script's alone, `lua/` is
/// `require`'s); the directories of an R package whose scripts R and
/// its tools run by path (`inst/` installed as it is, `vignettes/`,
/// `data-raw/`, `exec/`, `demo/`).
const ENTRY_DIRS: &str = "\
* src/bin/ examples/ benches/ cmd/
lua plugin/ ftplugin/ indent/ syntax/ colors/ compiler/ ftdetect/ lsp/ after/
r inst/ vignettes/ data-raw/ exec/ demo/";

/// Role facts of one file node; the declared-target role covers the
/// manifests' OWN declarations beside the name and place conventions.
pub(super) fn roles_of(root: &Path, path: &str, entries: &Inclusions, declared: &Declared) -> i64 {
    let base = path.rsplit('/').next().unwrap_or(path);
    let mut r = 0i64;
    if listed(ENTRY_NAMES, base) || path == "init.lua" {
        r |= ROLE_ENTRY_NAMED;
    }
    if matches!(
        base.rsplit_once('.').map(|(_, ext)| ext),
        Some("c" | "cc" | "cpp" | "cxx")
    ) {
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
    if matches!(base, "README.md" | "CLAUDE.md")
        || (path.starts_with("docs/") && matches!(base, "index.md" | "README.md"))
    {
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

/// Whether a row of ENTRY_DIRS for the file's language, or for every
/// language, holds a prefix of the path — R's rows under each package
/// root, the others under the tree root.
fn entry_dir(path: &str, declared: &Declared) -> bool {
    let lang = Lang::judged_path(Path::new(path)).map_or("", Lang::name);
    ENTRY_DIRS.lines().any(|row| {
        let mut words = row.split_whitespace();
        let Some(scope) = words.next().filter(|s| *s == "*" || *s == lang) else {
            return false;
        };
        let bases: Vec<&str> = match scope {
            "r" => declared.packages().collect(),
            _ => vec![""],
        };
        words.any(|dir| {
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

/// Spec.hs is the cabal test-suite main-is convention (hspec/stack
/// templates) — the test root nothing imports, like _test.go; the
/// C-family `_test` files, Maven Surefire's test classes, busted's
/// `_spec.lua` and testthat's `test-*.R` come from the table the
/// convention word reads too (runner_test, plan v2.30). testthat's own
/// directory is `tests/testthat/`, which the `tests/` rule reads.
fn is_test(path: &str, base: &str) -> bool {
    base.ends_with("_test.go")
        || runner_test(base)
        || base.ends_with(".test.ts")
        || (base.starts_with("test_") && base.ends_with(".py"))
        || base == "Spec.hs"
        || path.starts_with("tests/")
        || path.contains("/tests/")
        || path.contains("/__tests__/")
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/deadcode/flags.rs"]
mod tests;
