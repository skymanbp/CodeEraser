//! go.mod surface for the Go ladder (design §4 row 4): the module
//! path plus replace directives (single-line and block form; `//`
//! comments stripped; versions dropped). go.mod bytes sit in
//! resolve_key (store.rs), so disk reads here cannot go stale — the
//! package.json / Cargo.toml precedent.

use super::roots;
use std::path::Path;

pub struct GoMod {
    /// Repo-relative directory of the go.mod ("" = repo root).
    pub dir: String,
    pub module: Option<String>,
    /// (old, new): new is a filesystem path (./ or ../ per the go.mod
    /// spec) or a module path.
    pub replaces: Vec<(String, String)>,
}

/// Parse one go.mod at a repo-relative path.
pub fn parse(root: &Path, rel: &str) -> Option<GoMod> {
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    let mut module = None;
    let mut replaces = Vec::new();
    let mut in_block = false;
    for raw in text.lines() {
        let line = raw.split("//").next().unwrap_or("").trim();
        if in_block {
            if line == ")" {
                in_block = false;
            } else {
                push_replace(line, &mut replaces);
            }
        } else if let Some(rest) = line.strip_prefix("module ") {
            module = Some(rest.trim().trim_matches('"').to_string());
        } else if line == "replace (" {
            in_block = true;
        } else if let Some(rest) = line.strip_prefix("replace ") {
            push_replace(rest, &mut replaces);
        }
    }
    Some(GoMod {
        dir: roots::parent_dir(rel),
        module,
        replaces,
    })
}

/// "old [version] => new [version]" — versions dropped.
fn push_replace(entry: &str, out: &mut Vec<(String, String)>) {
    if let Some((lhs, rhs)) = entry.split_once("=>")
        && let (Some(old), Some(new)) =
            (lhs.split_whitespace().next(), rhs.split_whitespace().next())
    {
        out.push((old.to_string(), new.to_string()));
    }
}
