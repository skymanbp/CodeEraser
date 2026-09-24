//! compile_commands.json surface for the C-family ladder (plan v2.30
//! step 2; design booklet §8 row C / C++, rung 3): per translation
//! unit, the include directories its compiler invocation names. The
//! file is a resolver config (graph/keys.rs), so its bytes sit in
//! resolve_key and a disk read here cannot go stale — the go.mod
//! precedent.
//!
//! Read per the Clang JSON Compilation Database format: an array of
//! objects with `directory`, `file`, and `arguments` (an argv array)
//! or `command` (one shell string). The format spells `directory`
//! absolute and lets `file` and every `-I` directory be absolute or
//! relative to it; this side only ever compares them to the walked
//! repo-relative paths, so each is relativized against the repo root
//! LEXICALLY — no canonicalize, no stat — and a path outside the root
//! is dropped: it can hold no in-scope candidate. A `command` string
//! is split on whitespace; the format leaves shell quoting to the
//! producer, and a directory with a space in it is a boundary this
//! reading states rather than half-handles.

use super::roots;
use std::path::Path;

/// Clone: the sweep memo hands out per-config parses once and
/// callers keep owned copies.
#[derive(Clone)]
pub struct CompDb {
    /// (repo-relative translation unit, its include directories —
    /// repo-relative, in invocation order, the first hit winning).
    pub entries: Vec<(String, Vec<String>)>,
}

/// Parse one compile_commands.json at a repo-relative path.
pub fn parse(root: &Path, rel: &str) -> Option<CompDb> {
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    let rows: Vec<serde_json::Value> = serde_json::from_str(&text).ok()?;
    let root = root_text(root);
    let mut entries = Vec::new();
    for row in &rows {
        let dir = row.get("directory").and_then(|v| v.as_str()).unwrap_or("");
        let Some(file) = row.get("file").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(unit) = relativize(&root, dir, file) else {
            continue;
        };
        let argv = arguments(row);
        let dirs = include_dirs(&argv)
            .filter_map(|d| relativize(&root, dir, d))
            .collect();
        entries.push((unit, dirs));
    }
    Some(CompDb { entries })
}

/// `arguments` verbatim, else `command` split on whitespace.
fn arguments(row: &serde_json::Value) -> Vec<String> {
    if let Some(args) = row.get("arguments").and_then(|v| v.as_array()) {
        return args
            .iter()
            .filter_map(|a| a.as_str())
            .map(str::to_string)
            .collect();
    }
    row.get("command")
        .and_then(|v| v.as_str())
        .map(|c| c.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default()
}

/// `-I<dir>` / `-I <dir>`, and the same two spellings of `-iquote`
/// and `-isystem`, in invocation order.
fn include_dirs(argv: &[String]) -> impl Iterator<Item = &str> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i].as_str();
        let flag = ["-I", "-iquote", "-isystem"]
            .into_iter()
            .find(|f| arg.starts_with(f));
        match flag {
            Some(f) if arg.len() > f.len() => out.push(&arg[f.len()..]),
            Some(_) => {
                if let Some(next) = argv.get(i + 1) {
                    out.push(next.as_str());
                    i += 1;
                }
            }
            None => {}
        }
        i += 1;
    }
    out.into_iter()
}

/// The repo root as the database would spell it: forward slashes, no
/// verbatim prefix, no trailing separator.
fn root_text(root: &Path) -> String {
    slashed(&root.to_string_lossy())
        .trim_start_matches("//?/")
        .trim_end_matches('/')
        .to_string()
}

fn slashed(path: &str) -> String {
    path.replace('\\', "/")
}

/// A database path as the walk spells it: absolute, or relative to
/// the entry's `directory` (itself absolute by the format, or — a
/// producer's liberty — relative to the root), then stripped of the
/// root and normalized; None outside the root.
fn relativize(root: &str, dir: &str, path: &str) -> Option<String> {
    let path = slashed(path);
    if is_absolute(&path) {
        return roots::join_rel("", strip_root(root, &path)?);
    }
    let dir = slashed(dir);
    let base = if is_absolute(&dir) {
        strip_root(root, &dir)?.to_string()
    } else {
        dir
    };
    roots::join_rel(&base, &path)
}

fn is_absolute(path: &str) -> bool {
    path.starts_with('/') || path.as_bytes().get(1) == Some(&b':')
}

/// The path under `root`, or None. A drive-lettered root compares
/// without case (Windows spells the letter either way); a POSIX root
/// compares exactly.
fn strip_root<'a>(root: &str, abs: &'a str) -> Option<&'a str> {
    let head = abs.get(..root.len())?;
    let drive = root.as_bytes().get(1) == Some(&b':');
    let same = if drive {
        head.eq_ignore_ascii_case(root)
    } else {
        head == root
    };
    if !same {
        return None;
    }
    let rest = &abs[root.len()..];
    if rest.is_empty() {
        Some("")
    } else {
        rest.strip_prefix('/')
    }
}

#[cfg(test)]
#[path = "../../tests/unit/graph/compdb.rs"]
mod tests;
