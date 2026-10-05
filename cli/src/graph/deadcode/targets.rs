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
//! elsewhere) is the same fact by declaration. An R package's
//! DESCRIPTION (plan v2.30 step 4) declares the code R loads with the
//! package, and its directory is the root the R entry directories sit
//! under (flags.rs); the core reads it and answers the code (resolve/1
//! `packages`, plan v2.33 W2-text stage B). A cabal's main-is targets
//! are the core's answer too (resolve/1 `mains`, stage D).

use crate::graph::{cabal_find, cargo, roots};
use crate::scan::lang::Lang;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn is_r(path: &str) -> bool {
    Lang::from_path(Path::new(path)) == Some(Lang::R)
}

/// The per-run declared-target set, and each R package's code by its
/// root — what a `library(pkg)` edge reaches (nodes::contain).
pub(super) struct Declared {
    targets: BTreeSet<String>,
    packages: BTreeMap<String, BTreeSet<String>>,
}

impl Declared {
    /// One pass over the walked set: nearest Cargo.toml and nearest
    /// .cabal per unique directory, nearest DESCRIPTION per directory
    /// holding an R file, each manifest's targets computed once. A
    /// manifest above the repo root is out of tree by construction
    /// (nearest_up never leaves it). `declared` are the ce.toml crate
    /// roots, kept where the file is walked. An R package's code and a
    /// cabal's mains are the core's answer; a core that cannot give them
    /// is a named refusal.
    pub(super) fn gather(
        root: &Path,
        files: &BTreeSet<String>,
        declared: &BTreeSet<String>,
    ) -> Result<Self, String> {
        let dirs: BTreeSet<String> = files.iter().map(|f| roots::parent_dir(f)).collect();
        let r_dirs: BTreeSet<String> = files
            .iter()
            .filter(|f| is_r(f))
            .map(|f| roots::parent_dir(f))
            .collect();
        let rust: BTreeSet<String> = dirs
            .iter()
            .filter_map(|d| roots::nearest_up(root, d, "Cargo.toml"))
            .collect();
        let haskell: BTreeSet<String> = dirs
            .iter()
            .filter_map(|d| cabal_find::nearest(root, d))
            .collect();
        let r: BTreeSet<String> = r_dirs
            .iter()
            .filter_map(|d| roots::nearest_up(root, d, "DESCRIPTION"))
            .collect();
        let core = crate::graph::resolve::declared(root, files, &r, &haskell)?;
        let mut out = Declared {
            targets: declared.intersection(files).cloned().collect(),
            packages: core.packages,
        };
        out.targets.extend(out.packages.values().flatten().cloned());
        out.targets.extend(core.mains);
        for m in &rust {
            if let Some(p) = cargo::package(root, m) {
                out.targets.extend(p.crate_roots(files));
            }
        }
        Ok(out)
    }

    pub(super) fn hit(&self, path: &str) -> bool {
        self.targets.contains(path)
    }

    /// The R package roots: each DESCRIPTION's directory (`""` = the
    /// tree root).
    pub(super) fn packages(&self) -> impl Iterator<Item = &str> {
        self.packages.keys().map(String::as_str)
    }

    /// Each R package's code by its root — the members its package
    /// node fans out to, in place of every file under the directory.
    pub(super) fn package_code_by_root(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.packages
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/deadcode/targets.rs"]
mod tests;
