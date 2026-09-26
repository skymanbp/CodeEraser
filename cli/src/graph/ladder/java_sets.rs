//! Java source sets (plan v2.30 step 5): the Maven and Gradle standard
//! layout keeps a module's code in `<module>/src/<set>/java/`, and the
//! build compiles each set against what it may see — `main` against
//! main code alone (no test output is ever on its classpath), any other
//! set (`test`, `integrationTest`, a `testFixtures`) against itself and
//! main code. A package split between `main` and `test` is therefore
//! half a package to a main file and a whole one to a test file, and a
//! package answer names one directory: the part in the importing file's
//! own source root when that root holds one — the nearest part, as a
//! simple name's own directory comes first (java.rs R3). What else a
//! non-main set sees (another module's test fixtures) is build
//! configuration this ladder does not read, so only the certain rule
//! filters: a main file sees no other set. A file outside the layout
//! sees every file, as before.

/// The standard-layout source root holding `path` —
/// `<module>/src/<set>/java`, the module prefix empty at the tree root
/// — and its set, or None outside the layout.
pub fn source_root(path: &str) -> Option<(&str, &str)> {
    let segs: Vec<&str> = path.split('/').collect();
    let at =
        (0..segs.len().saturating_sub(3)).find(|&i| segs[i] == "src" && segs[i + 2] == "java")?;
    let len = segs[..at + 3].iter().map(|s| s.len() + 1).sum::<usize>() - 1;
    Some((&path[..len], segs[at + 1]))
}

/// Whether a Java file at `from` can see the file `candidate`: a main
/// set sees main sets only; everything else sees everything.
pub fn visible(from: &str, candidate: &str) -> bool {
    match (source_root(from), source_root(candidate)) {
        (Some((_, "main")), Some((_, set))) => set == "main",
        _ => true,
    }
}

/// Of a split package's directories, the one inside the importing
/// file's own source root — when exactly one is.
pub fn own_root_dir<'a>(from: &str, dirs: impl Iterator<Item = &'a String>) -> Option<String> {
    let (root, _) = source_root(from)?;
    let prefix = format!("{root}/");
    let mut mine = dirs.filter(|d| d.starts_with(&prefix));
    match (mine.next(), mine.next()) {
        (Some(one), None) => Some(one.clone()),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/java_sets.rs"]
mod tests;
