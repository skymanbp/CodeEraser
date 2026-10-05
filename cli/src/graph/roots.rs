//! Resolver-config surfaces (design brief §1 roots.rs): package.json
//! facts for the TS ladder (the tsconfig `extends` chain lives in the
//! sibling roots_ts.rs, which reads this file's helpers and is read by
//! nothing here — one way, no cycle); pyproject / go.mod / Cargo loaders
//! arrive with their language batches. tsconfig is JSONC in the wild,
//! so a minimal comment / trailing-comma stripper feeds serde_json (no
//! new deps, design §1). Config bytes are part of resolve_key
//! (store.rs), so reading them from disk here cannot serve stale
//! answers.

use serde_json::Value;
use std::path::Path;

/// One package.json surface, enough for the R4/R5 rungs. Clone: the
/// sweep memo hands out per-config parses once and callers keep
/// owned copies.
#[derive(Clone)]
pub struct Package {
    /// Repo-relative directory of the package ("" = repo root).
    pub dir: String,
    pub name: Option<String>,
    pub exports: Option<Value>,
    /// Union of dependencies/devDependencies/peerDependencies/
    /// optionalDependencies keys.
    pub deps: Vec<String>,
}

/// Parse one package.json at a repo-relative path.
pub fn package(root: &Path, rel: &str) -> Option<Package> {
    let doc = read_jsonc(root, rel)?;
    let deps = [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ]
    .iter()
    .filter_map(|k| doc.get(*k).and_then(Value::as_object))
    .flat_map(|m| m.keys().cloned())
    .collect();
    Some(Package {
        dir: parent_dir(rel),
        name: doc.get("name").and_then(Value::as_str).map(str::to_string),
        exports: doc.get("exports").cloned(),
        deps,
    })
}

/// Nearest package.json walking up from `from_dir`.
pub fn nearest_package(root: &Path, from_dir: &str) -> Option<Package> {
    package(root, &nearest_up(root, from_dir, "package.json")?)
}

pub(crate) fn nearest_up(root: &Path, from_dir: &str, name: &str) -> Option<String> {
    let mut dir = from_dir;
    loop {
        let rel = join_dir(dir, name);
        if root.join(&rel).is_file() {
            return Some(rel);
        }
        if dir.is_empty() {
            return None;
        }
        dir = dir.rfind('/').map_or("", |i| &dir[..i]);
    }
}

pub(crate) fn parent_dir(rel: &str) -> String {
    rel.rfind('/')
        .map_or(String::new(), |i| rel[..i].to_string())
}

/// A directory and every ancestor of it up to the repo root (""),
/// nearest first — the order Node walks node_modules in (ladder/ts.rs,
/// keys.rs; plan v2.30 step 5b).
pub(crate) fn ancestors(dir: &str) -> impl Iterator<Item = &str> {
    std::iter::successors(Some(dir), |d| {
        (!d.is_empty()).then(|| d.rfind('/').map_or("", |i| &d[..i]))
    })
}

pub(crate) fn join_dir(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    }
}

/// Join a relative spec onto a repo-relative dir, resolving ./ and
/// ../ segments; escaping above the repo root is None (out of tree).
pub fn join_rel(dir: &str, spec: &str) -> Option<String> {
    let mut parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();
    for seg in spec.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            s => parts.push(s),
        }
    }
    Some(parts.join("/"))
}

pub(crate) fn read_jsonc(root: &Path, rel: &str) -> Option<Value> {
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    serde_json::from_str(&super::jsonc::clean(&text)).ok()
}

// The 92e728b1 pyproject reader, frozen (tests subrepo
// unit/graph/oracle_cfg/pyproject.rs): the frozen Python ladder reads it
// here; the core reads the decoded document since plan v2.33 W2-text.
#[cfg(test)]
#[path = "../../tests/unit/graph/oracle_cfg/pyproject.rs"]
mod frozen_pyproject;
#[cfg(test)]
pub(crate) use frozen_pyproject::pyproject;

/// Walk one dotted key path into a parsed TOML document.
pub(crate) fn table_at<'a>(doc: &'a toml::Table, keys: &[&str]) -> Option<&'a toml::Value> {
    let mut cur = doc.get(keys[0])?;
    for key in &keys[1..] {
        cur = cur.get(key)?;
    }
    Some(cur)
}
