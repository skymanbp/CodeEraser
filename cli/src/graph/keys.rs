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

/// Source extension → the twins R2 can stat for it. The .mts/.cts
/// rows mirror esm_rewrite's [("js","ts"),("mjs","mts"),("cjs","cts")]
/// table (ladder/ts.rs): R2 tests `!root.join(js_twin).is_file()` on
/// exactly the .mjs/.cjs twin of an in-scope .mts/.cts source, so
/// while those rows were missing, creating or deleting foo.mjs beside
/// foo.mts flipped R2's answer under an unchanged resolve key and the
/// edge stayed stale forever — phase-1.5 cannot repair it either,
/// since dirty holds only content-refreshed judged files and .mjs is
/// scan-only, hence never judged. The .ts/.tsx row stays a superset
/// of R2's single "js" probe: a surplus stat fact costs one spurious
/// sweep, a missing one costs a permanently wrong edge.
const TWIN_EXTS: [(&str, &[&str]); 4] = [
    (".ts", &["js", "mjs", "cjs"]),
    (".tsx", &["js", "mjs", "cjs"]),
    (".mts", &["mjs"]),
    (".cts", &["cjs"]),
];

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
        let Some((stem, exts)) = TWIN_EXTS
            .iter()
            .find_map(|(src, exts)| Some((p.strip_suffix(src)?, *exts)))
        else {
            continue;
        };
        for ext in exts {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A scratch root holding exactly `files`. The in-scope set is
    /// minted apart from it (`live_of`): a probe's live names and its
    /// on-disk files deliberately differ — a .cts source with no file
    /// is still in scope, and the twin beside it is the fact tested.
    fn fixture(tag: &str, files: &[(&str, &str)]) -> PathBuf {
        let root = crate::testutil::scratch(tag);
        for (name, body) in files {
            std::fs::write(root.join(name), body).expect("fixture write");
        }
        root
    }

    fn live_of(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|n| (*n).to_string()).collect()
    }

    /// The phase-2 key as the sweep computes it — the twin and
    /// node_modules facts are INPUTS to it, never a separate answer.
    /// Spelling that join out at all four read sites was this batch's
    /// clone (dedup gate).
    fn key(root: &Path, live: &BTreeSet<String>) -> i64 {
        resolve_key(live, &ts_fs_facts(root, live))
    }

    /// R2 stats the .mjs twin of an in-scope .mts source, so its
    /// appearance must move the key. While ts_fs_facts only stripped
    /// .ts/.tsx, .mts/.cts sources contributed no twin fact at all and
    /// the rewrite verdict froze at whatever the first sweep saw.
    #[test]
    fn mjs_twin_beside_mts_moves_the_key() {
        let root = fixture("keys-mts-twin", &[("a.mts", "export {}")]);
        let live = live_of(&["a.mts"]);
        twin_refires(&root, &live, "a.mjs", "export {}");
    }

    /// Write `twin` and assert the key moves — the shared tail of both
    /// twin-table probes (the dedup gate refused the second copy).
    fn twin_refires(
        root: &Path,
        live: &std::collections::BTreeSet<String>,
        twin: &str,
        body: &str,
    ) {
        let before = key(root, live);
        std::fs::write(root.join(twin), body).expect("write twin");
        assert_ne!(
            before,
            key(root, live),
            "a {twin} twin must re-fire the sweep"
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    /// The .cts row of the same table; and the twin list is per-source
    /// exact — a stray .mjs next to a .cts is not one of R2's probes,
    /// so it must not be collected under the .cts source.
    #[test]
    fn cts_takes_cjs_twin_only() {
        let root = fixture("keys-cts-twin", &[("b.mjs", "export {}")]);
        let bare = fixture("keys-cts-bare", &[]);
        let live = live_of(&["b.cts"]);
        assert_eq!(
            key(&root, &live),
            key(&bare, &live),
            "a .mjs is not a .cts probe"
        );
        std::fs::remove_dir_all(&bare).expect("cleanup");
        twin_refires(&root, &live, "b.cjs", "module.exports={}");
    }
}
