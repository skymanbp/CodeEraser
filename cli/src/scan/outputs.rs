//! Build-output directories — the one part of the built-in excludes
//! (plan §4.1, walk.rs) that is decided by what sits BESIDE a
//! directory rather than by its name alone (plan v2.30 step 5). A
//! build tool writes its default output directory next to its project
//! file, so `target/` beside a `Cargo.toml` is Cargo's and a source
//! directory of that name there would collide with the tool's own
//! output. Anywhere else the name is an ordinary directory: luarocks
//! keeps its build back ends in `src/luarocks/build/` (the module
//! `luarocks.build`), a KOReader plugin ships its modules in
//! `target/`, and a Java or Go package may well be called `build`. The
//! any-depth globs this replaces hid every one of them from every
//! measurement, the reference graph included.
//!
//! One row per directory name: the project files whose tool writes it
//! by default, per that tool's own documentation — Cargo, Maven, sbt
//! and Leiningen `target/`; Gradle, setuptools and Flutter `build/`;
//! webpack, Vite and Parcel (`package.json`), setuptools' sdist and
//! wheel, and Cabal's first build tree `dist/`; Cabal's current one
//! `dist-newstyle/`. A `*` row entry matches any file with that suffix
//! (a package's `<name>.cabal`). A tool whose output directory is a
//! convention rather than a default (CMake's `-B build`) is no row: a
//! build tree the table does not name is excluded the declarative way,
//! by the tree's `.gitignore` — where such trees live in practice — or
//! a ce.toml `exclude`.

use std::path::Path;

const OUTPUTS: &str = "\
target Cargo.toml pom.xml build.sbt project.clj
build build.gradle build.gradle.kts settings.gradle settings.gradle.kts setup.py setup.cfg pyproject.toml pubspec.yaml
dist package.json setup.py setup.cfg pyproject.toml *.cabal
dist-newstyle cabal.project *.cabal";

/// The project files that make a directory of this name a build
/// output, or None when no row names it.
fn manifests(name: &str) -> Option<impl Iterator<Item = &'static str>> {
    OUTPUTS.lines().find_map(|row| {
        let mut words = row.split_ascii_whitespace();
        (words.next() == Some(name)).then_some(words)
    })
}

/// Whether `dir` is a build tool's output: a row names it and the
/// directory holding it holds one of that row's project files. Only
/// the parent is read (a `stat` per exact name, one listing for a
/// suffix entry), so a directory that does not exist yet answers too.
pub fn is_output(dir: &Path) -> bool {
    let (Some(name), Some(parent)) = (dir.file_name().and_then(|n| n.to_str()), dir.parent())
    else {
        return false;
    };
    manifests(name).is_some_and(|mut files| {
        files.any(|file| match file.strip_prefix('*') {
            Some(suffix) => holds_suffix(parent, suffix),
            None => parent.join(file).is_file(),
        })
    })
}

/// Whether `dir` holds a file whose name ends in `suffix`.
fn holds_suffix(dir: &Path, suffix: &str) -> bool {
    std::fs::read_dir(dir).is_ok_and(|entries| {
        entries.flatten().any(|e| {
            e.file_type().is_ok_and(|t| t.is_file())
                && e.file_name().to_string_lossy().ends_with(suffix)
        })
    })
}

#[cfg(test)]
#[path = "../../tests/unit/scan/outputs.rs"]
mod tests;
