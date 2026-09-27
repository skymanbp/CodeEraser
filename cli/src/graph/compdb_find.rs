//! Where a tree's compile databases are, found the way clangd finds
//! them (plan v2.30 step 5b, item 14): for each directory holding a
//! C-family file, that directory and every ancestor is probed for
//! `compile_commands.json`, `build/compile_commands.json` and
//! `compile_flags.txt` — clangd's `DirectoryCache` order
//! (clang-tools-extra/clangd/GlobalCompilationDatabase.cpp) — by a
//! stat, whatever the ignore rules say: a build tree is written beside
//! the project and is gitignored in practice, so the walk never enters
//! it, and a database nobody can find answers for nobody. The probed
//! files are resolve-key INPUTS like the tsconfig extends bases
//! (keys.rs): their bytes, and the bytes of every in-tree response
//! file a database names, join the key so that an edit re-fires the
//! sweep; the ladder (ladder/c_index.rs) and the deadcode request (the
//! forced-include arcs) read exactly this set through `found`.

use crate::dedup::tokens;
use crate::graph::compdb;
use crate::graph::roots;
use crate::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::Path;

/// clangd's probes per directory, in its order.
pub const PROBES: [&str; 3] = [
    "compile_commands.json",
    "build/compile_commands.json",
    "compile_flags.txt",
];

/// One database a probe found: the probed directory, the probe's index
/// in PROBES, and the file's repo-relative path.
pub struct Found {
    pub dir: String,
    pub probe: usize,
    pub rel: String,
}

/// Whether a walked path is a C-family file.
pub fn is_c(path: &str) -> bool {
    matches!(Lang::from_path(Path::new(path)), Some(Lang::C | Lang::Cpp))
}

/// Whether a path is a translation unit — a file the build compiles on
/// its own, `.c` / `.cc` / `.cpp` / `.cxx` (the compilation-unit role,
/// deadcode/flags.rs, reads the same set); a header is compiled only
/// through one.
pub fn is_unit(path: &str) -> bool {
    matches!(
        path.rsplit_once('.').map(|(_, ext)| ext),
        Some("c" | "cc" | "cpp" | "cxx")
    )
}

/// The directories clangd would probe for these files: each C-family
/// file's directory and its ancestors up to the root.
pub fn probe_dirs<'a>(files: impl Iterator<Item = &'a String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for file in files.filter(|f| is_c(f)) {
        let dir = roots::parent_dir(file);
        out.extend(roots::ancestors(&dir).map(str::to_string));
    }
    out
}

/// Every database that exists under `root` for those directories, in
/// probe-directory order then probe order (a file two probes reach —
/// `x/build/compile_commands.json` is probe 1 of `x/build` and probe 2
/// of `x` — is listed under each).
pub fn found<'a>(root: &Path, files: impl Iterator<Item = &'a String>) -> Vec<Found> {
    let mut out = Vec::new();
    for dir in probe_dirs(files) {
        for (probe, name) in PROBES.iter().enumerate() {
            let rel = roots::join_dir(&dir, name);
            if root.join(&rel).is_file() {
                out.push(Found {
                    dir: dir.clone(),
                    probe,
                    rel,
                });
            }
        }
    }
    out
}

/// The key inputs: each discovered database's (path, content hash),
/// and for a JSON database each response file it names, (path, hash)
/// — under their own labels, so a walked copy of the same file keys
/// beside them rather than over them. A response file is named with
/// `@`: a database holding no such byte names none, and is not parsed
/// on every walk.
pub fn facts<'a>(root: &Path, files: impl Iterator<Item = &'a String>) -> Vec<(String, u64)> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for f in found(root, files) {
        if !seen.insert(f.rel.clone()) {
            continue;
        }
        let bytes = std::fs::read(root.join(&f.rel)).unwrap_or_default();
        out.push((format!("c:db:{}", f.rel), tokens::fnv1a(&bytes)));
        if f.probe == 2 || !bytes.contains(&b'@') {
            continue;
        }
        let responses = compdb::parse(root, &f.rel).map(|db| db.responses);
        for rsp in responses.into_iter().flatten() {
            let bytes = std::fs::read(root.join(&rsp)).unwrap_or_default();
            out.push((format!("c:rsp:{rsp}"), tokens::fnv1a(&bytes)));
        }
    }
    out
}

#[cfg(test)]
#[path = "../../tests/unit/graph/compdb_find.rs"]
mod tests;
