//! The cabal layout walk (split from cabal.rs at the 300-line
//! dogfood gate as a `#[path]` child, the rs.rs → rs_use.rs shape):
//! stanza headers, fields with their continuation blocks, and the
//! routing of each collected field into the surface. cabal.rs owns
//! the surface TYPES and the reads; this file owns the parse
//! MECHANICS.

use super::{Cabal, HEADS, Stanza, roots};

/// One parser step — a stanza header, a field with its continuation
/// block, or a line to skip; returns the next line index. A `common`
/// or unknown header opens a DEAD region for hs-source-dirs: those
/// reach components only through `import:` indirection (not
/// modeled), and routing them to the previous stanza mis-attributed
/// a common stanza's roots (M5-close review LOW). A column-0 COMMENT
/// is not a header and must not toggle the region — the clearance
/// review caught the first cut killing a live stanza's fields on a
/// top-level `-- note` inside it.
pub(super) fn step(out: &mut Cabal, live: &mut bool, dir: &str, lines: &[&str], i: usize) -> usize {
    let trimmed = lines[i].trim();
    if !lines[i].starts_with([' ', '\t']) && !trimmed.is_empty() && !trimmed.starts_with("--") {
        let head = head_word(trimmed);
        *live = HEADS.contains(&head.as_str());
        // the public library is the bare `library` header; a named
        // one (`library internal-utils`) is an internal sublibrary,
        // importable only inside its own package
        out.has_library |= head == "library" && trimmed.split_whitespace().nth(1).is_none();
        if *live {
            out.stanzas.push(Stanza {
                roots: Vec::new(),
                main_is: None,
            });
        }
        return i + 1;
    }
    let Some((field, first)) = split_field(trimmed) else {
        return i + 1;
    };
    let mut values = vec![first.to_string()];
    let next = continuation(lines, i + 1, &mut values);
    consume(out, dir, &field, &values, *live);
    next
}

/// The field's continuation block: indented lines without a field
/// colon (cabal layout — value blocks carry no `name:` shape).
/// Comment lines inside the block are skipped — not values and not
/// an end, so a comment between two dependency lines cannot eat the
/// rest of the block.
fn continuation(lines: &[&str], mut i: usize, values: &mut Vec<String>) -> usize {
    while i < lines.len() && lines[i].starts_with([' ', '\t']) {
        let next = lines[i].trim();
        if next.is_empty() || split_field(next).is_some() {
            break;
        }
        if !next.starts_with("--") {
            values.push(next.to_string());
        }
        i += 1;
    }
    i
}

/// Degrade defaults: a file with no stanza headers at all (pre-2.0
/// top-level layout) and a stanza with no hs-source-dirs both root
/// at the package directory — cabal's own default of ".". The hidden
/// set is other-modules minus every exposed-modules listing, so a
/// module the library exposes AND a test-suite compiles is not hidden.
pub(super) fn finish(out: &mut Cabal, dir: &str) {
    if out.stanzas.is_empty() {
        out.stanzas.push(Stanza {
            roots: vec![dir.to_string()],
            main_is: None,
        });
    }
    for s in &mut out.stanzas {
        if s.roots.is_empty() {
            s.roots.push(dir.to_string());
        }
    }
    out.deps.sort();
    out.deps.dedup();
    out.hidden_modules = out.other.difference(&out.exposed).cloned().collect();
}

/// Route one collected field into the surface. hs-source-dirs is
/// per-STANZA and only lands while live (a common stanza's roots
/// reach components only via import:, not modeled); build-depends is
/// the file-wide UNION feeding the R2 external gate — a dependency
/// declared in a `common` stanza IS declared in this file, so it
/// accumulates regardless of the region (clearance review: the first
/// cut starved hs.rs's gate for the common-deps + import layout).
/// The two module lists are file-wide unions too, live stanzas only
/// (a common stanza's modules reach a component the same unmodeled
/// way its roots do). Fields outside any stanza (pre-2.0 top-level
/// layout) fall to the LAST stanza if one exists, else are dropped —
/// the empty-stanza default covers them.
fn consume(out: &mut Cabal, dir: &str, field: &str, values: &[String], live: bool) {
    match field {
        "hs-source-dirs" if live => {
            let Some(stanza) = out.stanzas.last_mut() else {
                return;
            };
            for v in words(values) {
                // "." is the package dir; an escape above the repo
                // root is out of tree and contributes nothing
                if let Some(joined) = roots::join_rel(dir, v) {
                    stanza.roots.push(joined);
                }
            }
        }
        "main-is" if live => set_main(out, values),
        "exposed-modules" if live => out.exposed.extend(words(values).map(str::to_string)),
        "other-modules" if live => out.other.extend(words(values).map(str::to_string)),
        "build-depends" => {
            for entry in values.iter().flat_map(|v| v.split(',')) {
                let name = dep_name(entry);
                if !name.is_empty() {
                    out.deps.push(name);
                }
            }
        }
        _ => {}
    }
}

/// The list-valued fields' items: comma- or whitespace-separated.
fn words(values: &[String]) -> impl Iterator<Item = &str> {
    values
        .iter()
        .flat_map(|v| v.split([',', ' ', '\t']))
        .filter(|w| !w.is_empty())
}

/// The stanza main-is value (2.28.0): the first collected value
/// wins — cabal main-is is a single filename. Split from consume
/// so the match arms stay calls (the repo cognitive gate caught the
/// inlined form at CoC 16).
fn set_main(out: &mut Cabal, values: &[String]) {
    if let Some(stanza) = out.stanzas.last_mut() {
        stanza.main_is = values.first().cloned();
    }
}

/// First word, lowercased — cabal stanza/field names are
/// case-insensitive, values are not.
fn head_word(trimmed: &str) -> String {
    trimmed
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// `Field-Name: rest` → (lowercased name, rest). None for lines that
/// carry no field colon (continuation values, stanza names).
fn split_field(trimmed: &str) -> Option<(String, &str)> {
    let colon = trimmed.find(':')?;
    let name = &trimmed[..colon];
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return None;
    }
    Some((name.to_ascii_lowercase(), trimmed[colon + 1..].trim()))
}

/// "base >=4.19 && <5" → "base" (leading package-name prefix). A
/// token that does not open with an alphanumeric is not a package
/// name and yields nothing.
fn dep_name(entry: &str) -> String {
    let entry = entry.trim();
    if !entry.starts_with(|c: char| c.is_ascii_alphanumeric()) {
        return String::new();
    }
    entry
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .map_or(entry, |i| &entry[..i])
        .to_string()
}
