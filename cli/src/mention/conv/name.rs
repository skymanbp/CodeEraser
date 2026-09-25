//! The convention-category word's NAME-TABLE half (sealed criterion
//! §3.2, plan v2.17 L round piece (6)): the bits a declaration earns
//! from its key, its name and its path — never from its syntax, which
//! is the stored AST half (mod.rs). Computed at wire time and stored
//! nowhere, so a table row here moves freely (no GRAPH_REV bump); the
//! renderer reads the same assembled word the core judged (K38).
//!
//! Four sources, each a bit family:
//!   - the PATH: `Test` by path component, package-root-qualified
//!     directory, equal basename or pattern (`*` non-empty); `Ambient`
//!     by the `.d.ts` family;
//!   - the NAME × language: `Main` (Python/Haskell/C/C++/Java `main`),
//!     `Protocol` (the framework names a loader spells for the author,
//!     one table per language in protocol.rs);
//!   - the KEY: a Go method's receiver decides `MemberDispatch` (an
//!     unexported receiver — only an interface or embedding reaches
//!     it) or `MemberApi` (an exported one);
//!   - the TEXT: a file-level `ce:allow(unmentioned) -- <why>` claim.
//!
//! Every bit is an exemption (silence), the safe direction of the veto.

use super::Conv;
use super::protocol::protocol;
use super::protocol::starred;
use crate::scan::lang::Lang;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Path components that make a file a test file wherever they sit.
const TEST_DIRS: [&str; 5] = ["test", "tests", "spec", "__tests__", "testdata"];
/// Basename suffixes behind a non-empty stem (`starred`): the Go and
/// Python `_test` files, hspec's `Spec.hs`.
const TEST_SUFFIXES: [&str; 3] = ["_test.go", "_test.py", "Spec.hs"];

/// The basenames a language's own test runner discovers, as `prefix *
/// suffix` with the `*` non-empty (`starred`): the C-family `_test`
/// convention (googletest's, Unity's) and Maven Surefire's default
/// includes (`Test*.java`, `*Test.java`, `*Tests.java`,
/// `*TestCase.java`; a bare `Test.java` earns nothing here, this
/// file's non-empty star). The one table both test readers share — the
/// dead-code role (graph/deadcode/flags.rs) and this category's path
/// half: the two lists judge different things elsewhere, and agree on
/// these by construction (plan v2.30, booklet §8). Lua: busted's
/// `_spec` and the `_test` of luaunit and its kin; R: testthat's
/// `test-` / `test_` files, which it finds under either extension case.
const RUNNER_TESTS: &str = "\
* _test.c
* _test.cc
* _test.cpp
Test * .java
* Test.java
* Tests.java
* TestCase.java
* _spec.lua
* _test.lua
test- * .R
test_ * .R
test- * .r
test_ * .r";

/// The path-derived bits of every file, memoized per file, with the
/// package-root stats memoized per DIRECTORY (one `Cargo.toml` stat
/// per directory per run — the `Declared::gather` discipline, L4-F13).
pub struct PathWords {
    root: PathBuf,
    pkg_roots: BTreeMap<String, bool>,
    words: BTreeMap<String, i64>,
}

impl PathWords {
    pub fn new(root: &Path) -> Self {
        PathWords {
            root: root.to_path_buf(),
            pkg_roots: BTreeMap::new(),
            words: BTreeMap::new(),
        }
    }

    /// `Test` and `Ambient` of one repo-relative path.
    pub fn bits(&mut self, rel: &str) -> i64 {
        if let Some(&word) = self.words.get(rel) {
            return word;
        }
        let mut word = 0;
        if self.test_file(rel) {
            word |= Conv::Test.bit();
        }
        if [".d.ts", ".d.mts", ".d.cts"]
            .iter()
            .any(|s| rel.ends_with(s))
        {
            word |= Conv::Ambient.bit();
        }
        self.words.insert(rel.to_string(), word);
        word
    }

    fn is_pkg_root(&mut self, dir: &str) -> bool {
        let root = &self.root;
        *self
            .pkg_roots
            .entry(dir.to_string())
            .or_insert_with(|| root.join(dir).join("Cargo.toml").is_file())
    }

    /// §3.2 bit 1, the path half: a component in TEST_DIRS; a
    /// `benches`/`examples` component whose parent is a Cargo package
    /// root; `conftest.py` / `Spec.hs` anywhere and `build.rs` as a
    /// package root's direct child (the equal group); and the pattern
    /// group with `*` non-empty — `_test.go` itself is a file the Go
    /// toolchain ignores and earns nothing.
    fn test_file(&mut self, rel: &str) -> bool {
        let comps: Vec<&str> = rel.split('/').collect();
        let (base, dirs) = comps.split_last().expect("a path has a basename");
        for (i, c) in dirs.iter().enumerate() {
            if TEST_DIRS.contains(c) {
                return true;
            }
            if matches!(*c, "benches" | "examples") && self.is_pkg_root(&dirs[..i].join("/")) {
                return true;
            }
        }
        matches!(*base, "conftest.py" | "Spec.hs")
            || (*base == "build.rs" && self.is_pkg_root(&dirs.join("/")))
            || TEST_SUFFIXES.iter().any(|s| starred(base, "", s))
            || runner_test(base)
            || starred(base, "test_", ".py")
            || infixed(base, ".test.")
            || infixed(base, ".spec.")
    }
}

/// A basename the language's own test runner discovers (RUNNER_TESTS).
pub(crate) fn runner_test(base: &str) -> bool {
    RUNNER_TESTS.lines().any(|row| {
        let (prefix, suffix) = row.split_once('*').expect("a `prefix * suffix` row");
        starred(base, prefix.trim(), suffix.trim())
    })
}

/// `*<mid>*` with both stars non-empty — at ANY occurrence, so a
/// basename opening with the infix (`.test.helper.test.ts`) still
/// matches on its later one.
fn infixed(base: &str, mid: &str) -> bool {
    base.match_indices(mid)
        .any(|(at, _)| at > 0 && at + mid.len() < base.len())
}

/// The name × key bits of one declaration: `Main`, `Protocol`, and
/// the Go receiver pair. `name` is `mention_name` of `key`.
pub fn name_bits(lang: Lang, rel: &str, key: &str, name: &str) -> i64 {
    let mut word = 0;
    if matches!(
        lang,
        Lang::Python | Lang::Haskell | Lang::C | Lang::Cpp | Lang::Java
    ) && name == "main"
    {
        word |= Conv::Main.bit();
    }
    if protocol(lang, rel, name) {
        word |= Conv::Protocol.bit();
    }
    if lang == Lang::Go {
        word |= go_member(key);
    }
    word
}

/// A file-level `ce:allow(unmentioned) -- <why>` claim exempts every
/// declaration of the file (the one claim grammar, crate::allow).
pub fn text_bits(text: &str) -> i64 {
    if crate::allow::allow_claim(text, "ce:allow(unmentioned)") {
        Conv::Allow.bit()
    } else {
        0
    }
}

/// The Go receiver's exportedness, read off the key `(<recv>) m/n`:
/// strip the outer parens (the text before the FIRST `") "`), the
/// leading `*`s, the type parameters from the first `[`, and the
/// package path up to the last `.`; empty at any step — `() M`,
/// `(*) M`, `([K]) M`, `(Cache.) M`, and `(anonymous)` (no `") "`) —
/// is neither bit (L4-F15/L6-F9). Exported = first char uppercase
/// (Go's `unicode.IsUpper`); `_` and non-uppercase non-ASCII are
/// dispatch, so the two bits partition every reached key.
fn go_member(key: &str) -> i64 {
    let Some((recv, _)) = key.strip_prefix('(').and_then(|k| k.split_once(") ")) else {
        return 0;
    };
    let recv = recv.trim_start_matches('*');
    let recv = recv.split('[').next().unwrap_or("");
    let recv = recv.rsplit('.').next().unwrap_or("");
    match recv.chars().next() {
        None => 0,
        Some(c) if c.is_uppercase() => Conv::MemberApi.bit(),
        Some(_) => Conv::MemberDispatch.bit(),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/mention/conv/name_tests.rs"]
mod tests;
