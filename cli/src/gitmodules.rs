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

/// Declared paths whose directory EXISTS but holds no entry — the shape
/// `git clone` without `--recurse-submodules` and `git submodule
/// deinit` both leave, and the one a filesystem walk cannot tell from
/// "no such files". A declared path missing altogether is a stale
/// stanza, not an unseated checkout, and names nothing.
pub fn unseated(root: &Path) -> Vec<String> {
    declared(root)
        .into_iter()
        .filter(|rel| {
            std::fs::read_dir(root.join(rel)).is_ok_and(|mut entries| entries.next().is_none())
        })
        .collect()
}

/// Declared paths whose checkout carries a `.git` entry — seated, so a
/// nested git command can be asked of them. Seated proves seating, not
/// health (a broken checkout leaves a `.git` file too).
pub fn seated(root: &Path) -> Vec<String> {
    declared(root)
        .into_iter()
        .filter(|rel| root.join(rel).join(".git").exists())
        .collect()
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
mod tests {
    use super::*;

    /// Every spelling git reads, and the two it must not: one document
    /// per case so the table is the grammar, not a loop over stanzas.
    /// The newline-in-value and tab cases are `git config -f` 2.52's
    /// own answers on this machine, byte for byte.
    #[test]
    fn the_reading_is_gits_own() {
        const CASES: [(&str, &[&str]); 12] = [
            ("[submodule \"s\"]\n\tpath = cli/tests\n", &["cli/tests"]),
            ("[submodule \"s\"]\n\tpath = \"cli/tests\"\n", &["cli/tests"]),
            ("[submodule \"s\"]\n\tpath = cli/tests # the suite\n", &["cli/tests"]),
            ("[submodule \"s\"]\n\tpath = cli/tests ; note\n", &["cli/tests"]),
            ("[SUBMODULE \"s\"]\n\tPath = cli/tests\n", &["cli/tests"]),
            ("[submodule \"s\"] path = cli/tests\n", &["cli/tests"]),
            ("[include]\n\tpath = ../elsewhere\n", &[]),
            ("[submodule \"a#b\"]\n\tpath = \"a#b\"\n", &["a#b"]),
            ("[submodule \"s\"]\n\tpath = \"a\\\\b\"\n", &["a/b"]),
            ("[submodule \"s\"]\n\tpath = cli/\\\n\ttests\n", &["cli/\n\ttests"]),
            ("[submodule \"s\"]\n\tpath = a\t\tb  \n", &["a\t\tb"]),
            ("[submodule \"s\"]\n\tpath = declared/\n\turl = x\n", &["declared"]),
        ];
        for (text, want) in CASES {
            let got: Vec<String> = parse(text).into_iter().collect();
            assert_eq!(got, want, "{text:?}");
        }
    }

    /// A full-line comment, a non-path key and a `]` inside the
    /// subsection name declare nothing; two stanzas declare two.
    #[test]
    fn only_a_submodule_stanzas_path_declares() {
        let text = "[submodule \"a]b\"]\n\t# path = ghost\n\tignore = all\n\turl = u\n\
                    \tpath = one\n[submodule \"two\"]\n\tpath=two\n[core]\n\tpath = no\n";
        let got: Vec<String> = parse(text).into_iter().collect();
        assert_eq!(got, ["one", "two"]);
    }
}
