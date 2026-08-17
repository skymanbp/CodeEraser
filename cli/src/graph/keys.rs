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

/// TS-resolver filesystem facts that can never enter the walked set
/// — compiled-JS twins of in-scope TS files (esm_rewrite stats them)
/// and the node_modules entry names (bare_rung stats them). They are
/// resolve-key INPUTS exactly like md slug hashes (the M5-close
/// precedent): mutate one and the phase-2 sweep must re-fire —
/// before this, those edges were cached across the mutation forever
/// (clearance review MED).
pub fn ts_fs_facts(root: &Path, live: &BTreeSet<String>) -> Vec<(String, u64)> {
    let mut twins = Vec::new();
    for p in live {
        let Some(stem) = p.strip_suffix(".ts").or_else(|| p.strip_suffix(".tsx")) else {
            continue;
        };
        for ext in ["js", "mjs", "cjs"] {
            let twin = format!("{stem}.{ext}");
            if root.join(&twin).is_file() {
                twins.push(twin);
            }
        }
    }
    twins.sort();
    let mut nm = node_modules_names(root);
    nm.sort();
    let h = |v: &[String]| crate::dedup::tokens::fnv1a(v.join("\n").as_bytes());
    vec![
        ("ts:js_twins".into(), h(&twins)),
        ("ts:node_modules".into(), h(&nm)),
    ]
}

/// Top-level node_modules entries, @scope dirs expanded one level —
/// the exact shapes bare_rung's `is_dir` probes can name.
fn node_modules_names(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(root.join("node_modules")) else {
        return out;
    };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('@') {
            if let Ok(inner) = std::fs::read_dir(e.path()) {
                for s in inner.flatten() {
                    out.push(format!("{name}/{}", s.file_name().to_string_lossy()));
                }
            }
        } else {
            out.push(name);
        }
    }
    out
}
