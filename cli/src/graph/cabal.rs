//! Cabal-file surface for the Haskell ladder (design §4 pattern:
//! gomod.rs / roots.rs — one minimal parser per resolver config,
//! bytes in resolve_key so answers cannot go stale). Only the facts
//! the rungs and the mounts table consume are modeled: per-stanza
//! `hs-source-dirs` (the R1 root set), the union of `build-depends`
//! package names (the R2 external gate), and since plan v2.17 piece
//! (5) the two package-privacy facts the sealed criterion §4 reads —
//! whether a `library` stanza exists at all, and which modules are
//! listed only under other-modules. Stated boundaries, degrading to
//! refusals never guesses: `common` stanza `import:` indirection is
//! not followed (its roots simply do not contribute), cabal.project
//! is not consulted (owner anchoring is directory-prefix), and
//! conditional blocks (`if os(..)`) contribute their fields
//! unconditionally — tag evaluation needs a build configuration we do
//! not have (the Go //go:build precedent). The layout walk itself is
//! the `#[path]` child cabal_parse.rs; this file owns the types and
//! the reads.

use super::roots;
use std::collections::BTreeSet;
use std::path::Path;

#[path = "cabal_parse.rs"]
mod layout;

/// One stanza's source roots, repo-relative ("" = repo root). A
/// stanza that declares no hs-source-dirs gets the package directory
/// itself — cabal's own default of ".".
#[derive(Clone)]
pub struct Stanza {
    pub roots: Vec<String>,
    /// The stanza's main-is file, verbatim (relative to each source
    /// root) — the declared-target role's cabal leg (2.28.0).
    pub main_is: Option<String>,
}

/// Clone: the sweep memo hands out per-config parses once and
/// callers keep owned copies.
#[derive(Clone)]
pub struct Cabal {
    /// Repo-relative directory of the .cabal file ("" = repo root).
    pub dir: String,
    /// Always at least one stanza (a pre-2.0 top-level-fields file
    /// parses as zero headers and degrades to one package-dir root).
    pub stanzas: Vec<Stanza>,
    /// build-depends package names, union across every stanza.
    pub deps: Vec<String>,
    /// Whether a `library` stanza exists — without one nothing in
    /// the package is importable from outside it.
    pub has_library: bool,
    /// Modules under other-modules and under no exposed-modules:
    /// declared, compiled, and private to their component.
    pub hidden_modules: BTreeSet<String>,
    exposed: BTreeSet<String>,
    other: BTreeSet<String>,
}

/// Stanza headers that open a component with source dirs. `common`
/// is deliberately absent: its fields reach components only through
/// `import:` indirection, which is not modeled (module header).
const HEADS: [&str; 4] = ["library", "executable", "test-suite", "benchmark"];

pub fn parse(root: &Path, rel: &str) -> Option<Cabal> {
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    let dir = roots::parent_dir(rel);
    let mut out = Cabal {
        dir: dir.clone(),
        stanzas: Vec::new(),
        deps: Vec::new(),
        has_library: false,
        hidden_modules: BTreeSet::new(),
        exposed: BTreeSet::new(),
        other: BTreeSet::new(),
    };
    let lines: Vec<&str> = text.lines().collect();
    // pre-2.0 top-level fields (before any header) are live
    let (mut i, mut live) = (0, true);
    while i < lines.len() {
        i = layout::step(&mut out, &mut live, &dir, &lines, i);
    }
    layout::finish(&mut out, &dir);
    Some(out)
}

/// Nearest *.cabal walking up from `from_dir` — the file name is
/// package-specific, so this is a per-directory scan, unlike
/// roots::nearest_up's fixed-name probe (`cargo::nearest` is the
/// fixed-name twin). Ties (several .cabal files in one directory)
/// resolve to the lexicographic first for determinism.
pub fn nearest(root: &Path, from_dir: &str) -> Option<String> {
    let mut dir = from_dir;
    loop {
        if let Some(name) = cabal_in(root, dir) {
            return Some(roots::join_dir(dir, &name));
        }
        if dir.is_empty() {
            return None;
        }
        dir = dir.rfind('/').map_or("", |i| &dir[..i]);
    }
}

fn cabal_in(root: &Path, dir: &str) -> Option<String> {
    let entries = std::fs::read_dir(root.join(dir)).ok()?;
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n.ends_with(".cabal").then_some(n)
        })
        .collect();
    names.sort();
    names.into_iter().next()
}

impl Cabal {
    /// Whether the package keeps `path` private (the mounts table's
    /// bit 1): no library stanza at all (then every file of the
    /// package, roots or not), or — with a library — the module the
    /// file spells under ANY stanza root is a hidden one. Every root
    /// is asked, not the first that prefixes the path: a stanza with
    /// no hs-source-dirs roots at the package directory, an ancestor
    /// of every other root, and stanza order is file order — so a
    /// first-match read mis-spelled the library's modules whenever the
    /// executable was written above it. Asking all roots is sound
    /// because `hidden_modules` is a file-wide set and a shallower
    /// root can only spell a name with a lowercase segment, never a
    /// module name. A file under no root spells no module and is not
    /// kept on this branch.
    pub fn keeps_private(&self, path: &str) -> bool {
        !self.has_library
            || self
                .stanzas
                .iter()
                .flat_map(|s| &s.roots)
                .filter_map(|r| module_under(r, path))
                .any(|m| self.hidden_modules.contains(&m))
    }
}

/// The module name `path` spells under one source root: the path
/// below the root, `/` → `.`, `.hs` dropped (ladder/hs.rs writes the
/// same convention the other way round); None when the file is not
/// below that root.
fn module_under(root: &str, path: &str) -> Option<String> {
    let below = if root.is_empty() {
        path
    } else {
        path.strip_prefix(&format!("{root}/"))?
    };
    Some(below.strip_suffix(".hs")?.replace('/', "."))
}

#[cfg(test)]
#[path = "cabal_tests.rs"]
mod tests;
