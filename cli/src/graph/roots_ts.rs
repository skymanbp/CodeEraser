//! The tsconfig chain: the nearest tsconfig from a directory and every
//! config its `extends` reaches, folded to the effective options the TS
//! ladder's third rung reads, and the chain's file list the resolve key
//! hashes (keys.rs). Split from roots.rs at the 300 line (plan v2.30
//! step 5b, the branch-stacked extends walk) as a SIBLING, not a
//! `#[path]` child: this file reads roots.rs's helpers and roots.rs
//! reads nothing here, so the split adds no cycle to the check's axis.

use super::roots::{join_rel, nearest_up, parent_dir, read_jsonc};
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
    /// (pattern, targets) — order preserved; the anchor lives in
    /// `paths_anchor` below.
    pub paths: Vec<(String, Vec<String>)>,
    /// Anchor for `paths` targets: base_dir when baseUrl is set,
    /// else the directory of the config that declared paths.
    pub paths_anchor: String,
}

/// Outcome of the chain walk: no config at all, usable options, or a
/// chain the ladder must refuse (a cycle, an unreadable or out-of-tree
/// target, a non-string entry) — Unresolved(config_depth), never a
/// guess.
pub enum TsChain {
    None,
    Ok(TsOptions),
    Broken,
}

/// Nearest tsconfig walking up from `from_dir` (repo-relative, "" =
/// root), then every config its extends chain reaches — deepest first,
/// an `extends` array read last entry first (TS 5.0: a later entry
/// overrides an earlier one, so the last is the nearest base, and each
/// entry's own chain is walked whole before the entry before it), no
/// depth cap (plan v2.30 step 5b: a chain is as long as the project
/// wrote it), a cycle refused. A bare-package extends target
/// (node_modules) ends its branch but keeps fields already collected:
/// the invisible base could only supply values the visible configs
/// did not override, and guessing them would be inventing config.
pub fn ts_options(root: &Path, from_dir: &str) -> TsChain {
    let Some(start) = nearest_up(root, from_dir, "tsconfig.json") else {
        return TsChain::None;
    };
    let mut opts = TsOptions::default();
    let walked = extends_walk(root, &start, &mut Vec::new(), &mut |doc, path| {
        collect_ts(doc, &parent_dir(path), &mut opts)
    });
    match walked {
        Ok(()) => TsChain::Ok(opts),
        Err(()) => TsChain::Broken,
    }
}

/// The extends walk from one config: `branch` is the path stack of the
/// current branch — a config on it again is a cycle, while a diamond
/// (one base reached by two branches) is not — and `visit` sees each
/// config's document with its path, deepest first.
fn extends_walk(
    root: &Path,
    current: &str,
    branch: &mut Vec<String>,
    visit: &mut dyn FnMut(&Value, &str),
) -> Result<(), ()> {
    if branch.iter().any(|c| c == current) {
        return Err(());
    }
    let doc = read_jsonc(root, current).ok_or(())?;
    visit(&doc, current);
    let dir = parent_dir(current);
    let targets: Vec<&Value> = match doc.get("extends") {
        None => Vec::new(),
        Some(Value::Array(items)) => items.iter().rev().collect(),
        Some(one) => vec![one],
    };
    branch.push(current.to_string());
    for target in targets {
        let Value::String(target) = target else {
            return Err(());
        };
        if !target.starts_with('.') {
            continue; // a bare package: the branch ends, fields stand
        }
        let next = join_rel(&dir, &format!("{}{}", target, json_ext(target))).ok_or(())?;
        extends_walk(root, &next, branch, visit)?;
    }
    branch.pop();
    Ok(())
}

/// Every config the extends chain from `start` reaches, `start` first
/// — the resolve key's input for targets the walk reads as no config
/// (keys.rs); a broken chain lists what was reached before the break.
pub fn ts_extends_files(root: &Path, start: &str) -> Vec<String> {
    let mut files = Vec::new();
    let _ = extends_walk(root, start, &mut Vec::new(), &mut |_, path| {
        files.push(path.to_string())
    });
    files
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

/// tsconfig extends without an extension implies .json.
fn json_ext(target: &str) -> &'static str {
    if target.ends_with(".json") {
        ""
    } else {
        ".json"
    }
}
