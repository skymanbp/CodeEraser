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
//!   - the NAME × language: `Main` (Python/Haskell `main`), `Protocol`
//!     (the framework names a loader spells for the author — Python
//!     unittest/xunit/pluggy/Django/reflection prefixes, TS filename ×
//!     export-name conventions, Haskell autogen modules and hspec);
//!   - the KEY: a Go method's receiver decides `MemberDispatch` (an
//!     unexported receiver — only an interface or embedding reaches
//!     it) or `MemberApi` (an exported one);
//!   - the TEXT: a file-level `ce:allow(unmentioned) -- <why>` claim.
//!
//! Every bit is an exemption (silence), the safe direction of the veto.

use super::Conv;
use crate::scan::lang::Lang;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Path components that make a file a test file wherever they sit.
const TEST_DIRS: [&str; 5] = ["test", "tests", "spec", "__tests__", "testdata"];

/// Python names a loader spells: unittest's discovered hooks, pytest's
/// xunit-style hooks, Django's loader targets. Prefixes follow.
const PY_NAMES: &str = "setUp tearDown setUpClass tearDownClass setUpModule tearDownModule \
                        asyncSetUp asyncTearDown load_tests runTest \
                        setup teardown setup_module teardown_module setup_function \
                        teardown_function setup_class teardown_class setup_method teardown_method \
                        Command Migration";
/// pluggy hooks and the fixed-prefix reflection Django/DRF perform
/// (`clean_<field>`, `validate_<field>`, `perform_<action>`).
const PY_PREFIXES: [&str; 4] = ["pytest_", "clean_", "validate_", "perform_"];

/// TS/TSX file form × export names, one line per form group: the
/// stem (basename minus extension) on the left, the function- or
/// class-capable exports a framework loads by name on the right —
/// Next App Router route handlers and segment hooks, SvelteKit
/// endpoints, page/layout loads and hooks, Next middleware and
/// instrumentation. Constant-form exports (`metadata`, `prerender`,
/// `actions`, `config`) are out of the domain by §3.1 and not rows.
const TS_BY_STEM: &str = "\
route : GET POST PUT PATCH DELETE HEAD OPTIONS
+server : GET POST PUT PATCH DELETE HEAD OPTIONS fallback
page layout template default loading error not-found global-error : generateStaticParams generateMetadata generateViewport
opengraph-image twitter-image icon apple-icon sitemap : generateImageMetadata generateSitemaps
middleware : middleware
instrumentation : register onRequestError
+page +page.server +layout +layout.server : load entries
hooks hooks.server hooks.client : handle handleError handleFetch init reroute";
/// Directory-scoped forms: Next/Astro `pages/**` (data fetchers and
/// endpoint verbs), Remix `routes/**` and its `root` module.
const TS_PAGES: &str =
    "getStaticProps getServerSideProps getStaticPaths GET POST PUT PATCH DELETE HEAD OPTIONS ALL";
const TS_ROUTES: &str = "loader action meta links headers ErrorBoundary HydrateFallback \
                         shouldRevalidate clientLoader clientAction";

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
            || starred(base, "", "_test.go")
            || starred(base, "", "_test.py")
            || starred(base, "test_", ".py")
            || starred(base, "", "Spec.hs")
            || infixed(base, ".test.")
            || infixed(base, ".spec.")
    }
}

/// `<prefix>*<suffix>` with `*` non-empty.
fn starred(base: &str, prefix: &str, suffix: &str) -> bool {
    base.len() > prefix.len() + suffix.len() && base.starts_with(prefix) && base.ends_with(suffix)
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
    if matches!(lang, Lang::Python | Lang::Haskell) && name == "main" {
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

fn protocol(lang: Lang, rel: &str, name: &str) -> bool {
    let base = rel.rsplit('/').next().unwrap_or(rel);
    match lang {
        Lang::Python => listed(PY_NAMES, name) || PY_PREFIXES.iter().any(|p| starred(name, p, "")),
        Lang::TypeScript | Lang::Tsx => ts_protocol(rel, base, name),
        Lang::Haskell => {
            base.starts_with("Paths_")
                || base.starts_with("PackageInfo_")
                || (name == "spec" && starred(base, "", "Spec.hs"))
        }
        _ => false,
    }
}

fn listed(table: &str, name: &str) -> bool {
    table.split_whitespace().any(|n| n == name)
}

/// Filename form × export name: the stem rows, then the directory
/// rows (`pages/**`; Remix `routes/**` and `app/root.*`).
fn ts_protocol(rel: &str, base: &str, name: &str) -> bool {
    let stem = base.rsplit_once('.').map_or(base, |(stem, _)| stem);
    let by_stem = TS_BY_STEM.lines().any(|row| {
        let (stems, names) = row.split_once(" : ").expect("a `stems : names` row");
        listed(stems, stem) && listed(names, name)
    });
    let in_dir = |d: &str| rel.split('/').rev().skip(1).any(|c| c == d);
    by_stem
        || (in_dir("pages") && listed(TS_PAGES, name))
        || ((in_dir("routes") || (stem == "root" && in_dir("app"))) && listed(TS_ROUTES, name))
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
#[path = "name_tests.rs"]
mod tests;
