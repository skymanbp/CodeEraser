//! The nearest .cabal of a directory (the directory scan the Haskell
//! manifests need; the file's reading moved into the core with the
//! Haskell rungs, CE.Resolve.Cabal — plan v2.33 W2-text stage D). The
//! declared-target pass (deadcode/targets.rs) and the mounts table
//! (mounts.rs) find a file's cabal here and send its text.

use super::roots;
use std::path::Path;

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
