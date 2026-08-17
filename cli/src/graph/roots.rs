//! Resolver-config surfaces (design brief §1 roots.rs): tsconfig
//! (+extends) chains and package.json facts for the TS ladder;
//! pyproject / go.mod / Cargo workspace loaders arrive with their
//! language batches. tsconfig is JSONC in the wild, so a minimal
//! comment / trailing-comma stripper feeds serde_json (no new deps,
//! design §1). Config bytes are part of resolve_key (store.rs), so
//! reading them from disk here cannot serve stale answers.

use serde_json::Value;
use std::path::Path;

/// Effective TS options after the extends walk. Each field comes
/// from the DEEPEST config that defines it (TS semantics: `paths`
/// replaces, never merges) and carries its anchor directory —
/// baseUrl resolves against the config that declared it, and paths
/// without baseUrl resolve against the config that declared them.
#[derive(Debug, Default)]
pub struct TsOptions {
    /// Directory the baseUrl points at, repo-relative ("" = root).
    pub base_dir: Option<String>,
    /// (pattern, targets, anchor_dir) — order preserved.
    pub paths: Vec<(String, Vec<String>)>,
    /// Anchor for `paths` targets: base_dir when baseUrl is set,
    /// else the directory of the config that declared paths.
    pub paths_anchor: String,
}

/// Outcome of the chain walk: no config at all, usable options, or a
/// chain the ladder must refuse (depth > 8 / cycle / non-string
/// extends) — Unresolved(config_depth), never a guess.
pub enum TsChain {
    None,
    Ok(TsOptions),
    Broken,
}

/// Nearest tsconfig walking up from `from_dir` (repo-relative, "" =
/// root), then its extends chain. A bare-package extends target
/// (node_modules) ends the walk but keeps fields already collected:
/// the invisible base could only supply values the visible configs
/// did not override, and guessing them would be inventing config.
pub fn ts_options(root: &Path, from_dir: &str) -> TsChain {
    let Some(start) = nearest_up(root, from_dir, "tsconfig.json") else {
        return TsChain::None;
    };
    let mut opts = TsOptions::default();
    let mut visited = Vec::new();
    let mut current = start;
    for _ in 0..8 {
        if visited.contains(&current) {
            return TsChain::Broken;
        }
        visited.push(current.clone());
        let Some(doc) = read_jsonc(root, &current) else {
            return TsChain::Broken;
        };
        let dir = parent_dir(&current);
        collect_ts(&doc, &dir, &mut opts);
        match doc.get("extends") {
            None => return TsChain::Ok(opts),
            Some(Value::String(target)) if target.starts_with('.') => {
                let base = format!("{}{}", target, json_ext(target));
                match join_rel(&dir, &base) {
                    Some(next) => current = next,
                    None => return TsChain::Broken,
                }
            }
            // bare package extends: unreachable in-scope, fields
            // collected so far stand (module doc)
            Some(Value::String(_)) => return TsChain::Ok(opts),
            Some(_) => return TsChain::Broken,
        }
    }
    TsChain::Broken
}

/// Deepest-wins collection of baseUrl and paths from one config.
fn collect_ts(doc: &Value, dir: &str, opts: &mut TsOptions) {
    let Some(co) = doc.get("compilerOptions") else {
        return;
    };
    if opts.base_dir.is_none()
        && let Some(base) = co.get("baseUrl").and_then(Value::as_str)
        && let Some(joined) = join_rel(dir, base)
    {
        opts.base_dir = Some(joined);
    }
    if opts.paths.is_empty()
        && let Some(map) = co.get("paths").and_then(Value::as_object)
    {
        for (pattern, targets) in map {
            let list: Vec<String> = targets
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|t| t.as_str().map(str::to_string))
                .collect();
            opts.paths.push((pattern.clone(), list));
        }
        opts.paths_anchor = dir.to_string();
    }
    if let Some(base) = &opts.base_dir
        && !opts.paths.is_empty()
    {
        opts.paths_anchor = base.clone();
    }
}

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

pub(crate) fn join_dir(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    }
}

/// tsconfig extends without an extension implies .json.
fn json_ext(target: &str) -> &'static str {
    if target.ends_with(".json") {
        ""
    } else {
        ".json"
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

fn read_jsonc(root: &Path, rel: &str) -> Option<Value> {
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    serde_json::from_str(&super::jsonc::clean(&text)).ok()
}

/// Python-side project surface: extra source roots and declared
/// dependency names, both read from the repo-root pyproject.toml
/// (its bytes sit in resolve_key, so answers cannot go stale).
pub struct PyProject {
    /// Directories that act as import roots besides the repo root
    /// and src/ ([tool.setuptools.package-dir] values and
    /// [tool.poetry.packages].from values).
    pub source_dirs: Vec<String>,
    /// [project] dependencies, reduced to bare package names.
    pub deps: Vec<String>,
}

pub fn pyproject(root: &Path) -> Option<PyProject> {
    let text = std::fs::read_to_string(root.join("pyproject.toml")).ok()?;
    // toml 1.x: Value::from_str parses a single VALUE; documents
    // parse as Table
    let doc: toml::Table = text.parse().ok()?;
    let setuptools = table_at(&doc, &["tool", "setuptools", "package-dir"])
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|map| map.values().filter_map(toml::Value::as_str));
    let poetry = table_at(&doc, &["tool", "poetry", "packages"])
        .and_then(toml::Value::as_array)
        .into_iter()
        .flat_map(|pkgs| {
            pkgs.iter()
                .filter_map(|p| p.get("from").and_then(toml::Value::as_str))
        });
    let source_dirs = setuptools.chain(poetry).map(str::to_string).collect();
    let deps = table_at(&doc, &["project", "dependencies"])
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|d| d.as_str())
        .map(dep_name)
        .collect();
    Some(PyProject { source_dirs, deps })
}

/// Walk one dotted key path into a parsed TOML document.
pub(crate) fn table_at<'a>(doc: &'a toml::Table, keys: &[&str]) -> Option<&'a toml::Value> {
    let mut cur = doc.get(keys[0])?;
    for key in &keys[1..] {
        cur = cur.get(key)?;
    }
    Some(cur)
}

/// "requests>=2.31 ; extra" → "requests" (PEP 508 name prefix).
fn dep_name(requirement: &str) -> String {
    requirement
        .find(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_' || c == '.'))
        .map_or(requirement, |i| &requirement[..i])
        .trim()
        .to_string()
}
