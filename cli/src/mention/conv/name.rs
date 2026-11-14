//! The convention-category word's NAME-TABLE half (sealed criterion
//! §3.2, plan v2.17 L round piece (6)): the bits a declaration earns
//! from its key, its name and its path — never from its syntax, which
//! is the stored AST half (mod.rs). Computed at wire time and stored
//! nowhere, so a table row here moves freely (no GRAPH_REV bump); the
//! renderer reads the same assembled word the core judged (K38).
//!
//! Four sources, each a bit family:
//!   - the PATH: `Test` by path component, package-root-qualified
//!     directory or file, or a runner's basename form; `Ambient` by the
//!     `.d.ts` family (every row the package's, CE.Lang.Common.PathForms);
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
use super::protocol::{listed, protocol, starred};
use crate::scan::lang::Lang;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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
        let ambient = crate::tables::get().flags.ambient_suffixes;
        if ambient.iter().any(|s| rel.ends_with(s)) {
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

    /// §3.2 bit 1, the path half, from the package's rows
    /// (CE.Lang.Common.PathForms): a component in `test_dirs_word`; a
    /// `cargo_dirs` component whose parent is a Cargo package root; a
    /// `cargo_files` name as a package root's direct child; and a
    /// basename a test runner discovers (`runner_tests`, the table the
    /// liveness role reads too). The directory rows are this reader's
    /// own: the liveness role keeps a narrower `test_dirs_role`, because
    /// a directory name is a project's habit, not a runner's rule.
    fn test_file(&mut self, rel: &str) -> bool {
        let flags = &crate::tables::get().flags;
        let comps: Vec<&str> = rel.split('/').collect();
        let (base, dirs) = comps.split_last().expect("a path has a basename");
        for (i, c) in dirs.iter().enumerate() {
            if listed(flags.test_dirs_word, c) {
                return true;
            }
            if listed(flags.cargo_dirs, c) && self.is_pkg_root(&dirs[..i].join("/")) {
                return true;
            }
        }
        (listed(flags.cargo_files, base) && self.is_pkg_root(&dirs.join("/"))) || runner_test(base)
    }
}

/// A basename a language's own test runner discovers: a row of the
/// package's `runner_tests` — `prefix * suffix` with the star non-empty
/// (`starred`), or the whole basename.
pub(crate) fn runner_test(base: &str) -> bool {
    crate::tables::get()
        .flags
        .runner_tests
        .iter()
        .any(|row| match row.split_once('*') {
            Some((prefix, suffix)) => starred(base, prefix.trim(), suffix.trim()),
            None => base == *row,
        })
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
