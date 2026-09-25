//! An R package's DESCRIPTION (Writing R Extensions §1.1.1), read for
//! the two fields the graph needs: `Package`, the name `library()` and
//! `pkg::` reach the package by (the R ladder's R2), and `Collate`, the
//! order R loads the package's `R/` files in — so the files the package
//! declares (the declared-target role, deadcode/targets.rs). The format
//! is Debian control: `Field: value`, the value continued on each
//! following line that starts with a space or a tab.

use crate::graph::roots;
use std::path::Path;

/// One DESCRIPTION: the directory holding it — the package root, `""`
/// at the tree root — the package's name, and its `Collate` list (file
/// names under `R/`; empty when the field is absent, and R then loads
/// every file of `R/`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description {
    pub dir: String,
    pub package: String,
    pub collate: Vec<String>,
}

/// One DESCRIPTION's text, the file held in `dir`. A package name is
/// one word; anything else names no package.
pub fn read(text: &str, dir: &str) -> Option<Description> {
    let package = field(text, "Package")?.trim().to_string();
    (!package.is_empty() && !package.contains(char::is_whitespace)).then(|| Description {
        dir: dir.to_string(),
        package,
        collate: field(text, "Collate").map_or_else(Vec::new, |v| names(&v)),
    })
}

/// The DESCRIPTION at `rel` under `root`; None when it cannot be read
/// or names no package. Read lossy: a DESCRIPTION may declare
/// `Encoding: latin1`, and the two fields read here are ASCII either
/// way.
pub fn parse(root: &Path, rel: &str) -> Option<Description> {
    let bytes = std::fs::read(root.join(rel)).ok()?;
    read(&String::from_utf8_lossy(&bytes), &roots::parent_dir(rel))
}

/// A field's value with its continuation lines, joined by newlines;
/// None when the file has no such field. `Packaged:` is not `Package:`.
fn field(text: &str, name: &str) -> Option<String> {
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let Some(value) = line.strip_prefix(name).and_then(|r| r.strip_prefix(':')) else {
            continue;
        };
        let mut out = value.to_string();
        for more in lines.by_ref().take_while(|l| l.starts_with([' ', '\t'])) {
            out.push('\n');
            out.push_str(more);
        }
        return Some(out);
    }
    None
}

/// The file names a `Collate` value lists: separated by white space,
/// each optionally quoted with `'` or `"` (a quoted name may hold a
/// space).
fn names(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = value.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        let name: String = if c == '\'' || c == '"' {
            chars.next();
            chars.by_ref().take_while(|&x| x != c).collect()
        } else {
            std::iter::from_fn(|| chars.next_if(|x| !x.is_whitespace())).collect()
        };
        if !name.is_empty() {
            out.push(name);
        }
    }
    out
}

#[cfg(test)]
#[path = "../../../../tests/unit/graph/ladder/r_description.rs"]
mod tests;
