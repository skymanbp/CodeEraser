//! The path steps the resolver-config readers share (design brief §1
//! roots.rs): the nearest file of a name walking up, a path's parent,
//! a directory's ancestors, the joins. The package.json and tsconfig
//! readers moved into the core in plan v2.33 W2-text stage E
//! (CE.Resolve.TsConfig); the TOML walk stays for the Cargo reader.

use std::path::Path;

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
/// nearest first — the order Node walks node_modules in (keys.rs; plan
/// v2.30 step 5b; the TS rungs read it as CE.Resolve.Str `ancestors`).
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

// The dd0eec61 package.json reader, frozen (tests subrepo
// unit/graph/oracle_cfg/ts_package.rs): the frozen TS rungs and the
// frozen tsconfig chain read it here, with the frozen JSONC cleaner
// (graph::jsonc, mounted for tests from oracle_cfg/jsonc.rs); the core
// reads the texts since plan v2.33 W2-text stage E. Only the mount and
// its re-export stand under cfg(test) here (it/unit_mounts.rs).
#[cfg(test)]
#[path = "../../tests/unit/graph/oracle_cfg/ts_package.rs"]
mod frozen_ts_package;
#[cfg(test)]
pub(crate) use frozen_ts_package::{Package, nearest_package, package, read_jsonc};

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
