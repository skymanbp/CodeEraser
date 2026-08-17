//! Phase-2 invalidation keys (design §3/§4): WHAT invalidates the
//! edge cache — the in-scope file set and the resolver config bytes.
//! Split from store.rs at the E01 300-line cap when the convergence
//! hotfix landed; store.rs re-exports both names, so callers keep
//! the `store::` spelling.

use std::collections::BTreeSet;
use std::path::Path;

/// Resolver-relevant config files (design §4 ladder inputs). The
/// tsconfig arm is a basename pattern because `extends` targets
/// conventionally read tsconfig.<flavor>.json and participate in
/// resolution — leaving them out of the key would serve stale edges
/// (2f refinement); an extends target under an arbitrary name stays
/// a documented boundary.
const CONFIG_NAMES: &[&str] = &["Cargo.toml", "go.mod", "package.json", "pyproject.toml"];

/// `.cabal` is a basename SUFFIX (the file carries the package name:
/// ce-core.cabal); cabal.project stays out — the hs ladder anchors
/// by directory prefix and never reads a workspace list (hs.rs).
pub fn is_resolver_config(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
        CONFIG_NAMES.contains(&n)
            || (n.starts_with("tsconfig") && n.ends_with(".json"))
            || n.ends_with(".cabal")
    })
}

/// Phase-2 cache key: fnv1a over the sorted in-scope paths plus each
/// resolver config's (path, content hash) — design §3. Adding or
/// removing a file, or touching a config, shifts the key; content
/// edits to ordinary files do not.
pub fn resolve_key(live: &BTreeSet<String>, configs: &[(String, u64)]) -> i64 {
    let mut buf = Vec::new();
    for path in live {
        buf.extend_from_slice(path.as_bytes());
        buf.push(b'\n');
    }
    for (path, hash) in configs {
        buf.extend_from_slice(path.as_bytes());
        buf.push(0);
        buf.extend_from_slice(&hash.to_le_bytes());
    }
    crate::dedup::tokens::fnv1a(&buf) as i64
}
