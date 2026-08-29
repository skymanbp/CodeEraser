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
mod tests {
    use super::*;

    /// The slice-3 defect, red→green at the fact level: a declared
    /// [[bin]] path is a target, its undeclared sibling is not; a
    /// cabal main-is lands the same way through its stanza roots; a
    /// ce.toml-declared root is a target when walked and nothing
    /// when it names a missing file.
    #[test]
    fn declared_targets_come_from_the_manifests() {
        let root = crate::testutil::scratch("dc-targets");
        let sources = [
            "src/tools/gen.rs",
            "src/tools/other.rs",
            "hs/app/Runner.hs",
            "it/main.rs",
        ];
        let mut tree = vec![
            (
                "Cargo.toml",
                "[package]\nname='t'\n[[bin]]\nname='gen'\npath='src/tools/gen.rs'\n",
            ),
            (
                "hs/x.cabal",
                "executable x\n  hs-source-dirs: app\n  main-is: Runner.hs\n",
            ),
        ];
        tree.extend(sources.map(|f| (f, "")));
        crate::testutil::write_tree(&root, &tree);
        let files: BTreeSet<String> = sources.map(String::from).into();
        let d = Declared::gather(&root, &files, &BTreeSet::new());
        assert!(d.hit("src/tools/gen.rs"), "declared [[bin]] path");
        assert!(!d.hit("src/tools/other.rs"), "undeclared sibling");
        assert!(d.hit("hs/app/Runner.hs"), "cabal main-is through its root");
        assert!(
            !d.hit("it/main.rs"),
            "a manifest-less main is no target by itself"
        );
        let declared = ["it/main.rs", "it/gone.rs"].map(String::from).into();
        let d = Declared::gather(&root, &files, &declared);
        assert!(d.hit("it/main.rs"), "declared in ce.toml and walked");
        assert!(
            !d.hit("it/gone.rs"),
            "declared but missing declares nothing"
        );
        std::fs::remove_dir_all(&root).ok();
    }
}
