//! The cabal layout walk (split from cabal.rs at the 300-line
//! dogfood gate as a `#[path]` child, the rs.rs → rs_use.rs shape):
//! stanza headers, fields with their continuation blocks, and the
//! routing of each collected field into the surface. cabal.rs owns
//! the surface TYPES and the reads; this file owns the parse
//! MECHANICS.

use super::{Cabal, HEADS, Stanza, roots};
use std::collections::{BTreeMap, BTreeSet};

/// Where the walk stands: in a component stanza (fields land on the
/// last stanza), in a named `common` stanza (fields land on its block,
/// pulled into components by `import:` — plan v2.17 L round step 8,
/// O58; before it a common stanza was a dead region whose roots
/// reached no component), or dead under an unknown or nameless
/// header. A column-0 COMMENT is not a header and must not move the
/// walk — the clearance review caught the first cut killing a live
/// stanza's fields on a top-level `-- note` inside it.
pub(super) enum Region {
    Live,
    Common(String),
    Dead,
}

/// A common stanza's importable fields: its roots and the two module
/// lists. build-depends is not held here — it is file-wide either way
/// (`consume`).
#[derive(Default, Clone)]
struct Common {
    roots: Vec<String>,
    exposed: BTreeSet<String>,
    other: BTreeSet<String>,
}

/// The walk's state: pre-2.0 top-level fields (before any header)
/// are live, and the common blocks accumulate in file order.
pub(super) struct Walk {
    region: Region,
    commons: BTreeMap<String, Common>,
}

impl Default for Walk {
    fn default() -> Self {
        Walk {
            region: Region::Live,
            commons: BTreeMap::new(),
        }
    }
}

/// One parser step — a stanza header, a field with its continuation
/// block, or a line to skip; returns the next line index.
pub(super) fn step(out: &mut Cabal, walk: &mut Walk, dir: &str, lines: &[&str], i: usize) -> usize {
    let trimmed = lines[i].trim();
    if !lines[i].starts_with([' ', '\t']) && !trimmed.is_empty() && !trimmed.starts_with("--") {
        walk.region = open(out, trimmed);
        return i + 1;
    }
    let Some((field, first)) = split_field(trimmed) else {
        return i + 1;
    };
    let mut values = vec![first.to_string()];
    let next = continuation(lines, i + 1, &mut values);
    consume(out, walk, dir, &field, &values);
    next
}

/// The region a header opens. The public library is the bare
/// `library` header; a named one (`library internal-utils`) is an
/// internal sublibrary, importable only inside its own package. A
/// `common` header without a name is a cabal error and opens nothing.
fn open(out: &mut Cabal, trimmed: &str) -> Region {
    let head = head_word(trimmed);
    let name = trimmed.split_whitespace().nth(1);
    out.has_library |= head == "library" && name.is_none();
    if head == "common" {
        return name.map_or(Region::Dead, |n| Region::Common(n.to_string()));
    }
    if !HEADS.contains(&head.as_str()) {
        return Region::Dead;
    }
    out.stanzas.push(Stanza {
        roots: Vec::new(),
        main_is: None,
    });
    Region::Live
}

/// The field's continuation block: indented lines without a field
/// colon (cabal layout — value blocks carry no `name:` shape).
/// Comment lines inside the block are skipped at ANY indentation —
/// cabal's lexer drops a `--` line wherever it stands, so a comment
/// between two dependency lines is neither a value nor an end, and a
/// column-0 one does not cut the block short either (the step-8
/// review's counterexample: the values after it were silently lost).
fn continuation(lines: &[&str], mut i: usize, values: &mut Vec<String>) -> usize {
    while i < lines.len() {
        let next = lines[i].trim();
        if next.starts_with("--") {
            i += 1;
            continue;
        }
        if !lines[i].starts_with([' ', '\t']) || next.is_empty() || split_field(next).is_some() {
            break;
        }
        values.push(next.to_string());
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

/// Route one collected field into the surface. build-depends is the
/// file-wide UNION feeding the R2 external gate — a dependency
/// declared in a `common` stanza IS declared in this file, so it
/// accumulates regardless of the region (clearance review: the first
/// cut starved hs.rs's gate for the common-deps + import layout).
/// The three importable fields are collected into one block and
/// landed where the walk stands (`merge`) — the same throat an
/// `import:` lands a common block through, so a field written in a
/// component and one pulled from a common cannot drift apart. Fields
/// outside any stanza (pre-2.0 top-level layout) fall to the LAST
/// stanza if one exists, else are dropped — the empty-stanza default
/// covers them.
fn consume(out: &mut Cabal, walk: &mut Walk, dir: &str, field: &str, values: &[String]) {
    let mut block = Common::default();
    match field {
        // "." is the package dir; an escape above the repo root is
        // out of tree and contributes nothing
        "hs-source-dirs" => {
            block.roots = words(values)
                .filter_map(|v| roots::join_rel(dir, v))
                .collect();
        }
        "exposed-modules" => block.exposed = words(values).map(str::to_string).collect(),
        "other-modules" => block.other = words(values).map(str::to_string).collect(),
        "import" => return import_commons(out, walk, values),
        "main-is" => return set_main(out, walk, values),
        "build-depends" => {
            for entry in values.iter().flat_map(|v| v.split(',')) {
                let name = dep_name(entry);
                if !name.is_empty() {
                    out.deps.push(name);
                }
            }
            return;
        }
        _ => return,
    }
    merge(out, walk, &block);
}

/// Land one block of importable fields where the walk stands: on the
/// last component stanza (roots per stanza, module lists file-wide),
/// on the open common stanza's block, or nowhere in a dead region.
fn merge(out: &mut Cabal, walk: &mut Walk, block: &Common) {
    match &walk.region {
        Region::Live => {
            if let Some(stanza) = out.stanzas.last_mut() {
                stanza.roots.extend(block.roots.iter().cloned());
            }
            out.exposed.extend(block.exposed.iter().cloned());
            out.other.extend(block.other.iter().cloned());
        }
        Region::Common(name) => {
            let c = walk.commons.entry(name.clone()).or_default();
            c.roots.extend(block.roots.iter().cloned());
            c.exposed.extend(block.exposed.iter().cloned());
            c.other.extend(block.other.iter().cloned());
        }
        Region::Dead => {}
    }
}

/// `import: a, b` pulls each named common block in — into the
/// component, or into another common stanza (cabal lets a common
/// import an earlier one, so the pulled block is already transitively
/// complete: definition-before-use is cabal's own rule). A name with
/// no block is a cabal error and pulls nothing.
fn import_commons(out: &mut Cabal, walk: &mut Walk, values: &[String]) {
    for name in words(values) {
        if let Some(block) = walk.commons.get(name).cloned() {
            merge(out, walk, &block);
        }
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
/// wins — cabal main-is is a single filename, a component field
/// only (a common stanza cannot carry one).
fn set_main(out: &mut Cabal, walk: &Walk, values: &[String]) {
    if !matches!(walk.region, Region::Live) {
        return;
    }
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
