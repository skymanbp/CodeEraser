//! The `mounts` table's producer (sealed criterion §4, plan v2.17 L
//! round piece (5)): for every node `[privateMounts, totalMounts,
//! bits]` — how many `mod` declarations mount a file and how many of
//! those are private, whether the file is a re-export target (bit 0)
//! and whether its own package keeps it private (bit 1). Three facts
//! from three clerical sources, none of them a graph walk: the graph's
//! own edges (a `mod_decl` edge joined to the declaring `mod` unit's
//! stored visibility, an edge that crossed a `pub use` — the
//! `via_reexport` mark — and a TS `export_star` site's target), a Go
//! file's package clause and path, and a manifest's target list (a
//! Cargo package's lib/bin targets, a cabal's library stanza and
//! other-modules). The folds — `mountedPrivate`, `pkgPrivate`, the
//! code order — are the core's (CE.Graph.Advisory, piece (6)); this
//! side measures.
//!
//! Coverage is the builder's contract, not the core's: `mount_rows`
//! maps EVERY node — package, section and phantom nodes to `[0,0,0]`
//! — in one full `enumerate().map().collect()`, because a missing row
//! reads as `[0,0,0]` on the other side and would turn a code 1/3 into
//! 0/2 with no validator able to see it (§4, W8-F4/W9-F1). The facts
//! themselves are keyed by the WALKED set, read here from the index in
//! the same snapshot as the edges, so a phantom node (an edge target
//! nothing walked) can never pick up a fact from its path alone. The
//! wire attachment (`GraphWire.mounts`) is piece (6); today the table
//! has this one producer and its tests.

use super::nodes::Node;
use super::{cabal, cargo, roots};
use crate::dedup::index::Index;
use crate::scan::lang::Lang;
use anyhow::{Result, ensure};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// bit 0: a re-export target — an edge crossed a terminal's `pub use`
/// to reach the file, or a TS `export *` names it.
pub const MOUNT_REEXPORTED: i64 = 1;
/// bit 1: the file's own package keeps it private — Go `package main`
/// or an `internal/` segment; a Cargo package without a lib target
/// (the whole package), else its bin roots; a cabal without a library
/// stanza (the whole package), else a module listed only under
/// other-modules; a Python module path with an underscore-led segment.
pub const MOUNT_PKG_PRIVATE: i64 = 1 << 1;

/// The facts per walked file, keyed by path — what the graph and the
/// manifests say; `mount_rows` projects them onto the node space.
#[derive(Default)]
pub struct MountFacts {
    /// (private, total) `mod` mounts of the mounted file.
    mounts: BTreeMap<String, (i64, i64)>,
    reexported: BTreeSet<String>,
    pkg_private: BTreeSet<String>,
}

/// One snapshot of the graph's own facts plus one manifest pass over
/// the walked set. A mount target outside the walked set is index
/// skew, named the way `symwire` names a symbol owner that is not a
/// node — the files and edges are read in ONE transaction (the
/// `graph_rows` discipline) so a convergent writer cannot manufacture
/// that skew between the two reads.
pub fn facts(root: &Path, idx: &Index) -> Result<MountFacts> {
    let txn = idx.raw().unchecked_transaction()?;
    let files: BTreeSet<String> = super::load::rows(&txn, "SELECT path FROM files", |r| r.get(0))?
        .into_iter()
        .collect();
    let edges = mount_edges(&txn)?;
    drop(txn);
    let mut out = MountFacts::default();
    for (dst, kind, via, pub_mod) in edges {
        ensure!(
            files.contains(&dst),
            "mount target {dst} not a walked file — index skew"
        );
        if kind == super::store::kind_code("mod_decl")? {
            let (private, total) = out.mounts.entry(dst.clone()).or_insert((0, 0));
            *total += 1;
            *private += i64::from(pub_mod == 0);
        }
        if via == 1 || kind == super::store::kind_code("export_star")? {
            out.reexported.insert(dst);
        }
    }
    let mut manifests = Manifests::default();
    for path in &files {
        if pkg_private(root, path, &files, &mut manifests) {
            out.pkg_private.insert(path.clone());
        }
    }
    Ok(out)
}

/// `(dst_path, site kind, via_reexport, pub mod)` for every file-level
/// edge that is a mount, a re-export crossing or a star export. The
/// declaring `mod` unit shares the site's line and name — a `mod x;`
/// is its own one-line unit keyed by its name (sites.rs owner rule,
/// fourclass::kinds extra) — so the visibility join is exact; a mount
/// whose unit is not there reads as private. Bit 0 of the stored
/// word is the criterion's export axis (`pub(crate) mod` keeps it set
/// and carries the restriction on bit 2 — its own rung, §4).
fn mount_edges(conn: &rusqlite::Connection) -> Result<Vec<(String, i64, i64, i64)>> {
    let sql = format!(
        "SELECT e.dst_path, s.kind, e.via_reexport,
                COALESCE((SELECT MAX(y.flags & 1) FROM symbols y
                          WHERE y.file_id = s.file_id AND y.key = s.spec
                            AND y.start_line = s.line), 0)
         FROM edges e JOIN sites s ON s.id = e.site_id
         WHERE e.granularity = {} AND (s.kind IN ({}, {}) OR e.via_reexport = 1)
         ORDER BY e.dst_path, s.id",
        super::wire::GRAN_FILE,
        super::store::kind_code("mod_decl")?,
        super::store::kind_code("export_star")?,
    );
    super::load::rows(conn, &sql, super::load::t4)
}

/// Every node's row, in node order — the coverage contract above.
pub fn mount_rows(nodes: &[Node], facts: &MountFacts) -> BTreeMap<i64, [i64; 3]> {
    nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (i as i64, facts.row(n)))
        .collect()
}

impl MountFacts {
    /// Facts are file facts: a package, section or phantom node has
    /// none and reads `[0,0,0]` — the core's own absent-row reading.
    fn row(&self, n: &Node) -> [i64; 3] {
        if n.kind != super::wire::GRAN_FILE {
            return [0, 0, 0];
        }
        let (private, total) = self.mounts.get(&n.path).copied().unwrap_or((0, 0));
        let bit = |set: &BTreeSet<String>, flag: i64| i64::from(set.contains(&n.path)) * flag;
        let bits =
            bit(&self.reexported, MOUNT_REEXPORTED) | bit(&self.pkg_private, MOUNT_PKG_PRIVATE);
        [private, total, bits]
    }
}

/// The bin-root facts of one Cargo package, computed ONCE per manifest
/// (the `Declared::gather` discipline — a per-file recomputation
/// rescans the walked set for every Rust file, the shape the criterion
/// itself rules out where nothing bounds the rescanned set, W9-F6).
pub(crate) struct RustTargets {
    /// `[package]` present — a virtual workspace manifest is not a
    /// package and keeps nothing.
    is_package: bool,
    has_lib: bool,
    bins: BTreeSet<String>,
}

impl RustTargets {
    pub(crate) fn of(pkg: Option<cargo::Package>, files: &BTreeSet<String>) -> Self {
        match pkg {
            Some(p) => RustTargets {
                is_package: p.name.is_some(),
                has_lib: p.lib_root(files).is_some(),
                bins: p.bin_roots(files),
            },
            None => RustTargets {
                is_package: false,
                has_lib: false,
                bins: BTreeSet::new(),
            },
        }
    }

    /// The Rust arm, symmetric with the cabal one (§4, L3-F15): a
    /// package without a lib target keeps every file (nothing outside
    /// can `use` it); one with a lib target keeps its bin roots alone
    /// — tests, benches, examples and build.rs are test-side facts,
    /// and a file below a bin root is the mount table's business, not
    /// this bit's.
    pub(crate) fn keeps(&self, path: &str) -> bool {
        self.is_package && (!self.has_lib || self.bins.contains(path))
    }
}

/// Manifests resolved once per directory and parsed once per
/// manifest — every file of a directory shares one nearest
/// Cargo.toml and one nearest .cabal, and every directory of a
/// package shares one parse.
#[derive(Default)]
struct Manifests {
    cargo_of: BTreeMap<String, Option<String>>,
    cargo: BTreeMap<String, RustTargets>,
    cabal_of: BTreeMap<String, Option<String>>,
    cabal: BTreeMap<String, Option<cabal::Cabal>>,
}

impl Manifests {
    fn rust(&mut self, root: &Path, dir: &str, files: &BTreeSet<String>) -> Option<&RustTargets> {
        let manifest = self
            .cargo_of
            .entry(dir.to_string())
            .or_insert_with(|| roots::nearest_up(root, dir, "Cargo.toml"))
            .clone()?;
        Some(
            self.cargo
                .entry(manifest.clone())
                .or_insert_with(|| RustTargets::of(cargo::package(root, &manifest), files)),
        )
    }

    fn haskell(&mut self, root: &Path, dir: &str) -> Option<&cabal::Cabal> {
        let manifest = self
            .cabal_of
            .entry(dir.to_string())
            .or_insert_with(|| cabal::nearest(root, dir))
            .clone()?;
        self.cabal
            .entry(manifest.clone())
            .or_insert_with(|| cabal::parse(root, &manifest))
            .as_ref()
    }
}

/// bit 1 by language; TS and Markdown have no package privacy the
/// criterion reads (0).
fn pkg_private(root: &Path, path: &str, files: &BTreeSet<String>, m: &mut Manifests) -> bool {
    let dir = roots::parent_dir(path);
    match Lang::judged_path(Path::new(path)) {
        Some(Lang::Go) => go_private(root, path),
        Some(Lang::Rust) => m.rust(root, &dir, files).is_some_and(|t| t.keeps(path)),
        Some(Lang::Haskell) => m.haskell(root, &dir).is_some_and(|c| c.keeps_private(path)),
        Some(Lang::Python) => py_private(path),
        _ => false,
    }
}

/// The Python arm (plan v2.17 L round step 8, user ruling 2026-08-28):
/// a module whose import path carries an underscore-led segment —
/// `pkg/_types.py`, `pkg/_internal/x.py` — is private to its package
/// by the language's own convention (PEP 8: an internal interface),
/// the same clerical fact Go's `internal/` is; a dunder module
/// (`__init__.py`, `__main__.py`) is protocol. The declaration's own
/// name is the visibility word's business, never this bit's (§4).
pub(crate) fn py_private(path: &str) -> bool {
    path.trim_end_matches(".py")
        .split('/')
        .any(|seg| seg.starts_with('_') && !(seg.starts_with("__") && seg.ends_with("__")))
}

/// The Go arm: `package main` is never importable, and an `internal/`
/// directory is importable only from its parent tree; `_test.go` is a
/// test file and out of this fact (its word carries TEST).
pub(crate) fn go_private(root: &Path, path: &str) -> bool {
    if path.ends_with("_test.go") {
        return false;
    }
    let internal = path.split('/').rev().skip(1).any(|seg| seg == "internal");
    internal || go_package(root, path).as_deref() == Some("main")
}

/// The package clause: the first line OUTSIDE comments that opens with
/// `package `, its next word. Block comments carry state across lines
/// because both misreadings are wrong answers, not safe ones — a
/// gofmt-indented `package main` example inside a doc comment must not
/// win (bit 1 raises the row to code 1), and a comment naming another
/// package must not hide a real `package main`. A clause sharing its
/// line with the end of a block comment is not read: one bounded
/// refusal, a shape gofmt never writes.
fn go_package(root: &Path, path: &str) -> Option<String> {
    let text = std::fs::read_to_string(root.join(path)).ok()?;
    let mut in_block = false;
    for raw in text.lines() {
        let Some(line) = outside_block(raw.trim(), &mut in_block) else {
            continue;
        };
        if let Some(rest) = line.strip_prefix("package ") {
            return Some(
                rest.split([' ', '\t', '/'])
                    .next()
                    .unwrap_or("")
                    .to_string(),
            );
        }
    }
    None
}

/// The code prefix of one line under the block-comment state: None
/// inside a `/* … */` region, the text before a `/*` that opens one,
/// the whole line otherwise. Go comments do not nest, so one flag is
/// the whole state.
fn outside_block<'a>(line: &'a str, in_block: &mut bool) -> Option<&'a str> {
    if *in_block {
        *in_block = !line.contains("*/");
        return None;
    }
    match line.find("/*") {
        Some(open) => {
            *in_block = !line[open + 2..].contains("*/");
            Some(line[..open].trim_end())
        }
        None => Some(line),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/graph/mounts_tests.rs"]
mod tests;
