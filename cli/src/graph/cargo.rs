//! Cargo surface for the Rust ladder (design §4 row 3): package
//! name, crate-root targets, and declared dependency names. Config
//! bytes sit in resolve_key (store.rs), so reading Cargo.toml from
//! disk here cannot serve stale answers — the package.json precedent.

use super::roots;
use std::collections::BTreeSet;
use std::path::Path;

/// One Cargo.toml surface, enough for the R1 root set and the R4
/// workspace / dependency rungs. Clone: the sweep memo hands out
/// per-config parses once and callers keep owned copies.
#[derive(Clone)]
pub struct Package {
    /// Repo-relative directory of the package ("" = repo root).
    pub dir: String,
    /// [package].name — hyphens allowed; the ladder normalizes.
    pub name: Option<String>,
    lib_path: Option<String>,
    bin_paths: Vec<String>,
    /// Union of dependencies/dev-dependencies/build-dependencies keys.
    pub deps: Vec<String>,
}

/// Parse one Cargo.toml at a repo-relative path.
pub fn package(root: &Path, rel: &str) -> Option<Package> {
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    // toml 1.x: Value::from_str parses a single VALUE; documents
    // parse as Table (roots.rs pyproject precedent)
    let doc: toml::Table = text.parse().ok()?;
    let deps = ["dependencies", "dev-dependencies", "build-dependencies"]
        .iter()
        .filter_map(|k| doc.get(*k).and_then(toml::Value::as_table))
        .flat_map(|m| m.keys().cloned())
        .collect();
    Some(Package {
        dir: roots::parent_dir(rel),
        name: roots::table_at(&doc, &["package", "name"]).and_then(as_string),
        lib_path: roots::table_at(&doc, &["lib", "path"]).and_then(as_string),
        bin_paths: doc
            .get("bin")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|b| b.get("path").and_then(as_string))
            .collect(),
        deps,
    })
}

fn as_string(v: &toml::Value) -> Option<String> {
    v.as_str().map(str::to_string)
}

/// Nearest Cargo.toml walking up from `from_dir`.
pub fn nearest(root: &Path, from_dir: &str) -> Option<Package> {
    package(root, &roots::nearest_up(root, from_dir, "Cargo.toml")?)
}

impl Package {
    /// Crate roots of this package that are scope files: declared
    /// [lib]/[[bin]] paths, the default targets, and the direct .rs
    /// children of the conventional auto-target dirs. The deeper
    /// auto-discovery form (src/bin/x/main.rs) is a documented recall
    /// limit — an unmodeled root's mod decls land Unresolved, never
    /// on a wrong file.
    pub fn crate_roots(&self, files: &BTreeSet<String>) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let declared = self.lib_path.iter().chain(&self.bin_paths);
        let defaults = ["src/lib.rs", "src/main.rs", "build.rs"];
        for target in declared.map(String::as_str).chain(defaults) {
            if let Some(cand) = roots::join_rel(&self.dir, target)
                && files.contains(&cand)
            {
                out.insert(cand);
            }
        }
        for sub in ["src/bin", "tests", "examples", "benches"] {
            let prefix = format!("{}/", roots::join_dir(&self.dir, sub));
            out.extend(
                files
                    .iter()
                    .filter(|f| {
                        f.strip_prefix(&prefix)
                            .is_some_and(|rest| rest.ends_with(".rs") && !rest.contains('/'))
                    })
                    .cloned(),
            );
        }
        out
    }

    /// The lib crate root — where `use <name>::…` from another
    /// package lands (design §4: workspace member → its root file).
    pub fn lib_root(&self, files: &BTreeSet<String>) -> Option<String> {
        let rel = self.lib_path.as_deref().unwrap_or("src/lib.rs");
        let cand = roots::join_rel(&self.dir, rel)?;
        files.contains(&cand).then_some(cand)
    }
}
