//! The superproject's declared submodules — ONE reader of the tracked
//! `.gitmodules`, shared by every surface that must know whether a
//! directory owning a `.git` entry is a foreign nested repository or
//! part of THIS tree: the mention universe's cut (mention/walk.rs),
//! root resolution (root.rs), the two whole-tree walks' seating
//! refusal (scan/walk.rs `collect`, mention/walk.rs `universe`), the
//! Stop audit's nested git legs (audit/changes.rs) and churn's named
//! shortfall. Read off the file's bytes with git's own config grammar
//! and never off git: a declaration is tree content, so the answer is
//! the commit's own on any machine, with or without a `.git` (the
//! mention walk's `require_git(false)` invariant, methodology 13 §1).
//!
//! git-config(1) grammar honoured: `[section "sub"]` headers (section
//! name case-insensitive, a variable may follow the `]` on the same
//! line), keys case-insensitive, `\"` `\\` `\n` `\t` `\b` escapes
//! inside AND outside quotes (git's parse_value treats them alike),
//! `#`/`;` comments outside quotes, whitespace kept verbatim (tabs
//! stay tabs; only trailing unquoted whitespace is shed), and a `\`
//! before the newline keeps the NEWLINE in the value — git 2.52
//! measured: `cli/\⏎tests` reads as `cli/⏎tests`, not as a joined
//! `cli/tests`, quoted or not. Only a
//! `path` under a `[submodule …]` header declares: a scanned
//! third-party tree is untrusted input, and a stray `path =` in some
//! other section must not exempt a foreign clone into this tree's U.
//! An escape git would reject (`\e`) is kept literally here — git
//! refuses the whole file, a walk cannot.

use std::collections::BTreeSet;
use std::path::Path;

/// The declared submodule paths under `root`, `/`-spelled without a
/// trailing slash — the spelling `scan::walk::rel_str` uses, so the
/// set joins directly against walked paths. No file = nothing declared.
pub fn declared(root: &Path) -> BTreeSet<String> {
    parse(&std::fs::read_to_string(root.join(".gitmodules")).unwrap_or_default())
}

/// Declared paths PRESENT but not checked out: the empty directory
/// `git clone` without `--recurse-submodules` and `git submodule
/// deinit` both leave, and every other presence that is no checkout
/// — a regular file at the path, a directory someone copied files
/// into, a `.git` gitfile whose pointer resolves to nothing (the
/// codex review of step #12 named all three; each was read as a
/// foreign reader instead of refused by name). A declared path
/// missing altogether is a stale stanza, not an unseated checkout,
/// and names nothing.
pub fn unseated(root: &Path) -> Vec<String> {
    declared_where(root, |dir| dir.exists() && !seated_at(dir))
}

/// Declared paths whose checkout carries a REAL `.git` anchor —
/// seated, so a nested git command can be asked of them.
pub fn seated(root: &Path) -> Vec<String> {
    declared_where(root, seated_at)
}

/// Seated declared submodules carrying a `ce.toml` of their own — the
/// nested projects WITH A GATE. The hooks delegate to them (plan v2.18
/// step #12): a write or a Stop under one of these is judged there,
/// under its own config, index and baseline; a submodule without one
/// is a reader here and measured by nobody.
pub fn gated(root: &Path) -> Vec<String> {
    declared_where(root, |dir| seated_at(dir) && dir.join("ce.toml").is_file())
}

/// A checkout sits at `dir`: its `.git` is a real anchor (root.rs — the
/// directory, or a gitfile whose pointer resolves). The ONE seating
/// test: the owner rule's Cut, trend's worktree seats and the three
/// questions above all read it, so a `.git` that anchors nothing
/// seats nothing anywhere (a `.exists()` here once seated a broken
/// gitfile that git itself refuses).
pub(crate) fn seated_at(dir: &Path) -> bool {
    crate::root::is_git_anchor(&dir.join(".git"))
}

/// The declared paths the owner rule can reach — a declaration under
/// an undeclared repository is cut with it and names nothing — whose
/// checkout directory satisfies `keep`: the one filter the three
/// seating questions above are asked through.
fn declared_where(root: &Path, keep: impl Fn(&Path) -> bool) -> Vec<String> {
    let declared = declared(root);
    declared
        .iter()
        .filter(|rel| owner(root, &declared, rel) != Owner::Cut && keep(&root.join(rel)))
        .cloned()
        .collect()
}

/// Who a root-relative path belongs to, read off the declarations and
/// the `.git` anchors on the way down — the ONE predicate both walks
/// and the guard's scope read (plan v2.18 step #12, user ruling
/// 2026-08-28: a declared submodule is a READER of this tree, never a
/// MEASURED part of it; an undeclared nested repository is nobody's
/// here). The walk order is the segment order: the first prefix that
/// is declared answers Foreign whether or not it is seated (an unseated
/// one is refused upstream by name), and the first prefix that owns a
/// real `.git` anchor without a declaration answers Cut — its files
/// are neither measured nor read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    /// This project's: measured, read, guarded.
    Own,
    /// A declared submodule's: read (mentions, graph edges), never
    /// measured (no size, score, clone or ratchet row), never a
    /// candidate of any verdict — the tree it belongs to gates it.
    Foreign,
    /// A nested repository nobody declared: cut whole.
    Cut,
}

pub fn owner(root: &Path, declared: &BTreeSet<String>, rel: &str) -> Owner {
    let mut dir = root.to_path_buf();
    let mut prefix = String::new();
    for seg in rel.split('/').filter(|s| !s.is_empty()) {
        dir.push(seg);
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(seg);
        if declared.contains(&prefix) {
            return Owner::Foreign;
        }
        // a REAL anchor only (root.rs: the directory, or a gitfile
        // whose pointer resolves) — a plain file named `.git` is what
        // one Write creates and a broken checkout leaves, and neither
        // makes the tree below it a repository of its own
        if crate::root::is_git_anchor(&dir.join(".git")) {
            return Owner::Cut;
        }
    }
    Owner::Own
}

/// The one refusal sentence for an unseated submodule (trend spoke it
/// first; every other refusing surface prefixes its own name).
pub fn refusal(rel: &str, root: &Path) -> String {
    format!(
        "submodule {rel} is not checked out under {} — `git submodule update --init` first",
        root.display()
    )
}

/// `.gitmodules` text → declared paths (see the module doc for the
/// grammar). Pure, so the K23 instruments can pin it on literal text.
pub fn parse(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut in_submodule = false;
    for line in logical_lines(text) {
        let mut rest = line.trim_start();
        if let Some(after) = rest.strip_prefix('[') {
            let Some((name, tail)) = header(after) else {
                continue;
            };
            in_submodule = name.eq_ignore_ascii_case("submodule");
            rest = tail.trim_start();
        }
        let Some((key, value)) = rest.split_once('=') else {
            continue; // a comment, a bare boolean, or nothing
        };
        if !in_submodule || !key.trim().eq_ignore_ascii_case("path") {
            continue;
        }
        let path = value_of(value).replace('\\', "/");
        let path = path.trim_end_matches('/');
        if !path.is_empty() {
            out.insert(path.to_string());
        }
    }
    out
}

/// A trailing unescaped `\` joins the next line into the same value,
/// the newline kept for `value_of` to decode as itself (git's reading,
/// in or out of quotes); CRs are shed so a CRLF file reads like an LF one.
fn logical_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut pending = String::new();
    for raw in text.lines() {
        let raw = raw.trim_end_matches('\r');
        pending.push_str(raw);
        let slashes = pending.chars().rev().take_while(|c| *c == '\\').count();
        if slashes % 2 == 1 {
            pending.push('\n');
            continue;
        }
        lines.push(std::mem::take(&mut pending));
    }
    if !pending.is_empty() {
        lines.push(pending);
    }
    lines
}

/// `section "sub"] tail` → (section, tail). A quoted subsection may
/// hold `]`, so the close bracket is looked for after the quote.
fn header(after: &str) -> Option<(&str, &str)> {
    let name_end = after.find(|c: char| c == ']' || c == '"' || c.is_whitespace())?;
    let name = &after[..name_end];
    let mut rest = after[name_end..].trim_start();
    if let Some(quoted) = rest.strip_prefix('"') {
        let mut escaped = false;
        let close = quoted.find(|c: char| {
            let done = c == '"' && !escaped;
            escaped = c == '\\' && !escaped;
            done
        })?;
        rest = quoted[close + 1..].trim_start();
    }
    let tail = rest.strip_prefix(']')?;
    Some((name, tail))
}

/// One variable's value: escapes decoded, a comment character outside
/// quotes ends it, trailing whitespace stripped unless quoted.
fn value_of(raw: &str) -> String {
    let mut out: Vec<(char, bool)> = Vec::new();
    let (mut quoted, mut escaped) = (false, false);
    for c in raw.trim_start().chars() {
        if escaped {
            let decoded = match c {
                'n' => '\n',
                't' => '\t',
                'b' => '\u{8}',
                other => other,
            };
            out.push((decoded, quoted));
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            quoted = !quoted;
        } else if !quoted && (c == '#' || c == ';') {
            break;
        } else {
            out.push((c, quoted));
        }
    }
    while out.last().is_some_and(|(c, q)| !q && c.is_whitespace()) {
        out.pop();
    }
    out.into_iter().map(|(c, _)| c).collect()
}

#[cfg(test)]
#[path = "../tests/unit/gitmodules/tests.rs"]
mod tests;
