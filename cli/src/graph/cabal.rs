//! Cabal-file surface for the Haskell ladder (design §4 pattern:
//! gomod.rs / roots.rs — one minimal parser per resolver config,
//! bytes in resolve_key so answers cannot go stale). Only the facts
//! the rungs consume are modeled: per-stanza `hs-source-dirs` (the R1
//! root set) and the union of `build-depends` package names (the R2
//! external gate). Stated boundaries, degrading to refusals never
//! guesses: `common` stanza `import:` indirection is not followed
//! (its roots simply do not contribute), cabal.project is not
//! consulted (owner anchoring is directory-prefix), and conditional
//! blocks (`if os(..)`) contribute their fields unconditionally —
//! tag evaluation needs a build configuration we do not have (the Go
//! //go:build precedent).

use super::roots;
use std::path::Path;

/// One stanza's source roots, repo-relative ("" = repo root). A
/// stanza that declares no hs-source-dirs gets the package directory
/// itself — cabal's own default of ".".
#[derive(Clone)]
pub struct Stanza {
    pub roots: Vec<String>,
}

/// Clone: the sweep memo hands out per-config parses once and
/// callers keep owned copies.
#[derive(Clone)]
pub struct Cabal {
    /// Repo-relative directory of the .cabal file ("" = repo root).
    pub dir: String,
    /// Always at least one stanza (a pre-2.0 top-level-fields file
    /// parses as zero headers and degrades to one package-dir root).
    pub stanzas: Vec<Stanza>,
    /// build-depends package names, union across every stanza.
    pub deps: Vec<String>,
}

/// Stanza headers that open a component with source dirs. `common`
/// is deliberately absent: its fields reach components only through
/// `import:` indirection, which is not modeled (module header).
const HEADS: [&str; 4] = ["library", "executable", "test-suite", "benchmark"];

pub fn parse(root: &Path, rel: &str) -> Option<Cabal> {
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    let dir = roots::parent_dir(rel);
    let mut out = Cabal {
        dir: dir.clone(),
        stanzas: Vec::new(),
        deps: Vec::new(),
    };
    let lines: Vec<&str> = text.lines().collect();
    // pre-2.0 top-level fields (before any header) are live
    let (mut i, mut live) = (0, true);
    while i < lines.len() {
        i = step(&mut out, &mut live, &dir, &lines, i);
    }
    finish(&mut out, &dir);
    Some(out)
}

/// One parser step — a stanza header, a field with its continuation
/// block, or a line to skip; returns the next line index. A `common`
/// or unknown header opens a DEAD region for hs-source-dirs: those
/// reach components only through `import:` indirection (not
/// modeled), and routing them to the previous stanza mis-attributed
/// a common stanza's roots (M5-close review LOW). A column-0 COMMENT
/// is not a header and must not toggle the region — the clearance
/// review caught the first cut killing a live stanza's fields on a
/// top-level `-- note` inside it.
fn step(out: &mut Cabal, live: &mut bool, dir: &str, lines: &[&str], i: usize) -> usize {
    let trimmed = lines[i].trim();
    if !lines[i].starts_with([' ', '\t']) && !trimmed.is_empty() && !trimmed.starts_with("--") {
        *live = HEADS.contains(&head_word(trimmed).as_str());
        if *live {
            out.stanzas.push(Stanza { roots: Vec::new() });
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
/// at the package directory — cabal's own default of ".".
fn finish(out: &mut Cabal, dir: &str) {
    if out.stanzas.is_empty() {
        out.stanzas.push(Stanza {
            roots: vec![dir.to_string()],
        });
    }
    for s in &mut out.stanzas {
        if s.roots.is_empty() {
            s.roots.push(dir.to_string());
        }
    }
    out.deps.sort();
    out.deps.dedup();
}

/// Route one collected field into the surface. hs-source-dirs is
/// per-STANZA and only lands while live (a common stanza's roots
/// reach components only via import:, not modeled); build-depends is
/// the file-wide UNION feeding the R2 external gate — a dependency
/// declared in a `common` stanza IS declared in this file, so it
/// accumulates regardless of the region (clearance review: the first
/// cut starved hs.rs's gate for the common-deps + import layout).
/// Fields outside any stanza (pre-2.0 top-level layout) fall to the
/// LAST stanza if one exists, else are dropped — the empty-stanza
/// default covers them.
fn consume(out: &mut Cabal, dir: &str, field: &str, values: &[String], live: bool) {
    match field {
        "hs-source-dirs" if live => {
            let Some(stanza) = out.stanzas.last_mut() else {
                return;
            };
            for v in values.iter().flat_map(|v| v.split([',', ' ', '\t'])) {
                if v.is_empty() {
                    continue;
                }
                // "." is the package dir; an escape above the repo
                // root is out of tree and contributes nothing
                if let Some(joined) = roots::join_rel(dir, v) {
                    stanza.roots.push(joined);
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::parse;

    /// The REAL core cabal is the self-corpus resolution anchor: two
    /// stanzas (executable app; test-suite app+test — the multi-root
    /// comma line), five deps through a multi-line build-depends
    /// block. The 3l reconciliation stands on exactly this parse:
    /// 176 hs sites = 78 in-corpus R1 edges + 80 declared-external
    /// (base/containers/bytestring/array) + 18 aeson out-of-db.
    #[test]
    fn the_self_corpus_cabal_parses_to_its_known_facts() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cli/ has a parent");
        let c = parse(root, "core/ce-core.cabal").expect("parse");
        assert_eq!(c.dir, "core");
        let roots: Vec<&[String]> = c.stanzas.iter().map(|s| s.roots.as_slice()).collect();
        assert_eq!(
            roots,
            [
                ["core/app".to_string()].as_slice(),
                ["core/app".to_string(), "core/test".to_string()].as_slice()
            ]
        );
        assert_eq!(
            c.deps,
            ["aeson", "array", "base", "bytestring", "containers"]
        );
    }

    fn parse_str(text: &str) -> super::Cabal {
        let dir = std::env::temp_dir().join(format!("ce-cabal-probe-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("pkg")).expect("mkdir");
        std::fs::write(dir.join("pkg/x.cabal"), text).expect("write");
        parse(&dir, "pkg/x.cabal").expect("parse")
    }

    /// The two dead-region boundary cases the clearance review drew
    /// (both counterfactual against the first `live`-bit cut):
    /// a column-0 comment INSIDE a live stanza must not open a dead
    /// region, and a `common` stanza drops its roots but its
    /// build-depends still join the file-wide union feeding R2.
    #[test]
    fn comments_keep_stanzas_live_and_common_deps_still_count() {
        let c =
            parse_str("library\n-- top-level note\n  hs-source-dirs: src\n  build-depends: base\n");
        assert_eq!(c.stanzas[0].roots, ["pkg/src".to_string()]);
        assert_eq!(c.deps, ["base"]);
        let c = parse_str(
            "common deps\n  hs-source-dirs: gen\n  build-depends: text\nlibrary\n  hs-source-dirs: src\n",
        );
        assert_eq!(c.stanzas.len(), 1, "common opens no stanza");
        assert_eq!(
            c.stanzas[0].roots,
            ["pkg/src".to_string()],
            "common roots dropped"
        );
        assert_eq!(
            c.deps,
            ["text"],
            "common build-depends still declared in this file"
        );
    }
}
