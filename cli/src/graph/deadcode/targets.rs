//! Declared-target discovery for the entry roles (proto 2.28.0,
//! batch-7 slice 3 main body): which files a manifest actually
//! declares as build targets — Cargo [lib]/[[bin]] paths plus the
//! conventional targets crate_roots models, and cabal main-is. The
//! defect this closes: a declared `[[bin]] path = "src/tools/x.rs"`
//! earned no entry standing while any stray main.rs did (inventory
//! slice 3, defects a/b). Discovery is nearest-manifest from each
//! walked directory, each manifest parsed once; the answer is a FACT
//! (this path is a declared target) — the entry decision stays with
//! the core's role table. A root ce.toml `[graph] crate_roots`
//! declares (plan v2.18 step #12: a tree whose manifest lives
//! elsewhere) is the same fact by declaration.

use crate::graph::{cabal, cargo, roots};
use std::collections::BTreeSet;
use std::path::Path;

/// Declared executable/test mains (2.28.0): each stanza's main-is
/// joined onto each of its source roots, kept only where the file is
/// in the walked set — a main-is naming a missing file declares
/// nothing.
fn main_targets(c: &cabal::Cabal, files: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for s in &c.stanzas {
        let Some(main) = &s.main_is else { continue };
        for r in &s.roots {
            if let Some(cand) = roots::join_rel(r, main)
                && files.contains(&cand)
            {
                out.insert(cand);
            }
        }
    }
    out
}

/// The per-run declared-target set.
pub(super) struct Declared(BTreeSet<String>);

impl Declared {
    /// One pass over the walked set: nearest Cargo.toml and nearest
    /// .cabal per unique directory, each manifest's targets computed
    /// once. A manifest above the repo root is out of tree by
    /// construction (nearest_up never leaves it). `declared` are the
    /// ce.toml crate roots, kept where the file is walked.
    pub(super) fn gather(
        root: &Path,
        files: &BTreeSet<String>,
        declared: &BTreeSet<String>,
    ) -> Self {
        let dirs: BTreeSet<String> = files.iter().map(|f| roots::parent_dir(f)).collect();
        let mut manifests = BTreeSet::new();
        for d in &dirs {
            if let Some(m) = roots::nearest_up(root, d, "Cargo.toml") {
                manifests.insert(m);
            }
            if let Some(m) = cabal::nearest(root, d) {
                manifests.insert(m);
            }
        }
        let mut out = BTreeSet::new();
        for m in &manifests {
            if m.ends_with("Cargo.toml") {
                if let Some(p) = cargo::package(root, m) {
                    out.extend(p.crate_roots(files));
                }
            } else if let Some(c) = cabal::parse(root, m) {
                out.extend(main_targets(&c, files));
            }
        }
        out.extend(declared.intersection(files).cloned());
        Declared(out)
    }

    pub(super) fn hit(&self, path: &str) -> bool {
        self.0.contains(path)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/deadcode/targets.rs"]
mod tests;
