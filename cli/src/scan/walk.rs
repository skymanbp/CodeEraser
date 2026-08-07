//! File discovery with the exclusion model (plan §4.1):
//! .gitignore (via `ignore` crate) + .ceignore + built-in category
//! defaults + ce.toml globs. Declarative only.

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use std::path::{Path, PathBuf};

/// Built-in excludes: lockfiles, minified/generated, vendored,
/// snapshots, migrations (plan §4.1 category list).
const BUILTIN_EXCLUDES: &[&str] = &[
    "!package-lock.json",
    "!yarn.lock",
    "!pnpm-lock.yaml",
    "!Cargo.lock",
    "!*.min.js",
    "!*.min.css",
    "!*.pb.go",
    "!*_pb2.py",
    "!*.generated.*",
    "!vendor/",
    "!node_modules/",
    "!__snapshots__/",
    "!*.snap",
    "!migrations/",
    "!dist/",
    "!build/",
    "!target/",
    "!dist-newstyle/",
];

/// Collect candidate files under `root`, honoring the exclusion model.
pub fn collect(root: &Path, extra_excludes: &[String]) -> Result<Vec<PathBuf>, String> {
    let overrides = build_overrides(root, extra_excludes)?;
    let mut files = Vec::new();
    let walker = WalkBuilder::new(root)
        .add_custom_ignore_filename(".ceignore")
        .overrides(overrides)
        .hidden(true)
        .build();
    for entry in walker {
        let entry = entry.map_err(|e| format!("walk: {e}"))?;
        if entry.file_type().is_some_and(|t| t.is_file()) {
            files.push(entry.into_path());
        }
    }
    files.sort();
    Ok(files)
}

fn build_overrides(root: &Path, extra: &[String]) -> Result<ignore::overrides::Override, String> {
    let mut builder = OverrideBuilder::new(root);
    for glob in BUILTIN_EXCLUDES {
        builder
            .add(glob)
            .map_err(|e| format!("builtin glob {glob}: {e}"))?;
    }
    for glob in extra {
        // ce.toml lists positive globs to exclude; Override semantics
        // need the leading '!' for exclusion.
        let g = format!("!{glob}");
        builder
            .add(&g)
            .map_err(|e| format!("ce.toml exclude {glob}: {e}"))?;
    }
    builder.build().map_err(|e| format!("excludes: {e}"))
}
