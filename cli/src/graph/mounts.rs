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
//! code order — are the core's (CE.Graph.Advisory, piece (6)), and so
//! are the cabal's and the Cargo.toml's readings (resolve/1 `private`,
//! plan v2.33 W2-text stages D and F: this side finds each Haskell
//! file's nearest .cabal and each Rust file's nearest Cargo.toml and
//! sends them); this side measures.
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
use super::{cabal_find, roots};
use crate::dedup::index::Index;
use crate::scan::lang::Lang;
use anyhow::{Result, ensure};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[path = "mounts_go.rs"]
mod go;
pub(crate) use go::go_private;

/// bit 0: a re-export target — an edge crossed a terminal's `pub use`
/// to reach the file, or a TS `export *` names it.
pub const MOUNT_REEXPORTED: i64 = 1;
/// bit 1: the file's own package keeps it private — Go `package main`
/// or an `internal/` segment; a Cargo package without a lib target
/// (the whole package), else its bin roots; a cabal without a library
/// stanza (the whole package), else a module listed only under
/// other-modules (both the core's reading); a Python module path with
/// an underscore-led segment.
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
    out.pkg_private = package_private(root, &files)?;
    Ok(out)
}

/// bit 1 over the walked set: Go's and Python's read here, Haskell's and
/// Rust's asked of the core with each file's nearest manifest.
fn package_private(root: &Path, files: &BTreeSet<String>) -> Result<BTreeSet<String>> {
    let mut manifests = Manifests::default();
    let (mut hs, mut rs, mut out) = (BTreeMap::new(), BTreeMap::new(), BTreeSet::new());
    for path in files {
        let dir = roots::parent_dir(path);
        let (owners, found) = match Lang::judged_path(Path::new(path)) {
            Some(Lang::Haskell) => (&mut hs, manifests.cabal_of(root, &dir)),
            Some(Lang::Rust) => (&mut rs, manifests.cargo_of(root, &dir)),
            _ => {
                if pkg_private(root, path) {
                    out.insert(path.clone());
                }
                continue;
            }
        };
        if let Some(manifest) = found {
            owners.insert(path.clone(), manifest);
        }
    }
    out.extend(super::resolve::private(root, files, &hs, &rs).map_err(anyhow::Error::msg)?);
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

/// Manifests found once per directory — every file of a directory
/// shares one nearest Cargo.toml and one nearest .cabal; each is read
/// once, by the core (the Rust arm, symmetric with the cabal one, §4,
/// L3-F15: a package without a lib target keeps every file, one with a
/// lib target its bin roots alone; a manifest that does not read, or a
/// virtual workspace, keeps nothing — CE.Resolve.Cargo `keeps`).
#[derive(Default)]
struct Manifests {
    cargo_of: BTreeMap<String, Option<String>>,
    cabal_of: BTreeMap<String, Option<String>>,
}

impl Manifests {
    fn cargo_of(&mut self, root: &Path, dir: &str) -> Option<String> {
        self.cargo_of
            .entry(dir.to_string())
            .or_insert_with(|| roots::nearest_up(root, dir, "Cargo.toml"))
            .clone()
    }

    fn cabal_of(&mut self, root: &Path, dir: &str) -> Option<String> {
        self.cabal_of
            .entry(dir.to_string())
            .or_insert_with(|| cabal_find::nearest(root, dir))
            .clone()
    }
}

/// bit 1 by language (Haskell's and Rust's are the core's: `facts`);
/// TS and Markdown have no package privacy the criterion reads (0).
fn pkg_private(root: &Path, path: &str) -> bool {
    match Lang::judged_path(Path::new(path)) {
        Some(Lang::Go) => go_private(root, path),
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

#[cfg(test)]
#[path = "../../tests/unit/graph/mounts_tests.rs"]
mod tests;
