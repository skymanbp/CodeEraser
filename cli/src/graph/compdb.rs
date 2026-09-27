//! The C-family databases: translation-unit commands and directory-wide flags.
//! https://clang.llvm.org/docs/JSONCompilationDatabase.html, Format: `arguments`
//! is argv; `command` uses quoting. File and flag paths belong to `directory`.
//! Placement stays lexical against the root: no canonicalize, no stat, no host
//! working directory. Outside paths cannot hold an in-scope candidate.
//! https://github.com/llvm/llvm-project/blob/main/clang/lib/Tooling/CompilationDatabase.cpp,
//! expandResponseFiles and FixedCompilationDatabase::loadFromBuffer: JSON
//! commands expand response files, fixed databases trim nonempty flag lines.
//! Each trimmed flag line is one verbatim argument; separate operands occupy
//! separate lines, and response-looking words in that form remain literal.
//! https://github.com/llvm/llvm-project/blob/main/clang-tools-extra/clangd/GlobalCompilationDatabase.cpp,
//! DirectoryCache::load expands JSON responses too. Here their placed names,
//! including missing files, are resolve-key inputs like tsconfig bases (keys.rs).
//! Response cycles remain literal; sixteen open files bound a recursive branch.
//! Program spelling selects response syntax (cmdline.rs), never the host OS.

use super::{
    cmdline,
    compdb_flags::{self, Chain},
    roots,
};
use std::{collections::BTreeSet, path::Path};

/// One translation unit, its working directory, and its explicit include chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Translation unit placed relative to the repository root.
    pub unit: String,
    /// Placed working directory; None outside the tree, empty at its root.
    pub dir: Option<String>,
    /// Search paths and forced includes defined by the invocation.
    pub chain: Chain,
}

/// In-tree entries in file order, with every named in-tree response file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompDb {
    /// Entries whose translation unit belongs to the tree.
    pub entries: Vec<Entry>,
    /// Resolve-key inputs, whether readable, missing, cyclic, or depth-limited.
    pub responses: BTreeSet<String>,
}

/// One directory's compile_flags.txt, applied with GNU flag semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flags {
    /// The database's parent directory relative to the root.
    pub dir: String,
    /// The explicit search chain of its flags.
    pub chain: Chain,
}

/// Read one JSON compilation database; skip missing or outside translation units.
pub fn parse(root: &Path, rel: &str) -> Option<CompDb> {
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    let rows: Vec<serde_json::Value> = serde_json::from_str(&text).ok()?;
    let base = root_text(root);
    let mut db = CompDb::default();
    for row in &rows {
        let dir = row.get("directory").and_then(|v| v.as_str()).unwrap_or("");
        let Some(unit) = row
            .get("file")
            .and_then(|v| v.as_str())
            .and_then(|file| relativize(&base, dir, file))
        else {
            continue;
        };
        let argv = arguments(row);
        let windows = argv
            .first()
            .is_some_and(|program| cmdline::windows_shaped(program));
        let argv = expand(
            root,
            dir,
            &argv,
            windows,
            &mut db.responses,
            &mut Vec::new(),
        );
        db.entries.push(Entry {
            unit,
            dir: relativize(&base, dir, ""),
            chain: compdb_flags::chain(&argv, &|path| relativize(&base, dir, path)),
        });
    }
    Some(db)
}

/// Fixed flags are anchored to their file, with no response expansion or host defaults.
pub fn parse_flags(root: &Path, rel: &str) -> Option<Flags> {
    let (text, dir) = roots::beside(root, rel)?;
    let base = root_text(root);
    let argv = std::iter::once("clang".to_string())
        .chain(
            text.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string),
        )
        .collect::<Vec<_>>();
    let chain = compdb_flags::chain(&argv, &|path| relativize(&base, &dir, path));
    Some(Flags { dir, chain })
}

/// An arguments array is verbatim, even empty; otherwise tokenize the command.
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
        .map(cmdline::split_command)
        .unwrap_or_default()
}

/// Expand a branch against the entry's working directory, recording before reading.
fn expand(
    root: &Path,
    dir: &str,
    argv: &[String],
    windows: bool,
    responses: &mut BTreeSet<String>,
    stack: &mut Vec<String>,
) -> Vec<String> {
    let mut out = Vec::new();
    let base = root_text(root);
    for arg in argv {
        let Some(path) = arg
            .strip_prefix('@')
            .and_then(|path| relativize(&base, dir, path))
        else {
            out.push(arg.clone());
            continue;
        };
        responses.insert(path.clone());
        if stack.contains(&path) || stack.len() >= 16 {
            out.push(arg.clone());
            continue;
        }
        let Ok(bytes) = std::fs::read(root.join(&path)) else {
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
        let words = if windows {
            cmdline::split_windows(text, false)
        } else {
            cmdline::split_gnu(text)
        };
        stack.push(path);
        out.extend(expand(root, dir, &words, windows, responses, stack));
        stack.pop();
    }
    out
}

/// Root spelling shared with the ladder: forward slashes, no verbatim or final slash.
pub(crate) fn root_text(root: &Path) -> String {
    slashed(&root.to_string_lossy())
        .trim_start_matches("//?/")
        .trim_end_matches('/')
        .to_string()
}

/// Treat either database separator alike on every host.
pub(crate) fn slashed(path: &str) -> String {
    path.replace('\\', "/")
}

/// Place an absolute or working-directory-relative path lexically inside the tree.
pub(crate) fn relativize(root: &str, dir: &str, path: &str) -> Option<String> {
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
    roots::join_rel(&roots::join_rel("", &base)?, &path)
}

/// Absolute spellings include POSIX paths and drive-qualified Windows paths.
pub(crate) fn is_absolute(path: &str) -> bool {
    path.starts_with('/') || path.as_bytes().get(1) == Some(&b':')
}

/// Strip a whole root component, case-insensitively only for a drive-lettered root.
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

/// Mounted compilation-database unit tests.
#[cfg(test)]
#[path = "../../tests/unit/graph/compdb.rs"]
mod tests;
