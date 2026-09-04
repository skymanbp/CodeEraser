//! The changelog-role detector (user ruling 2026-09-04, spec §三 M5):
//! a document whose JOB is to narrate change — a changelog, release
//! notes, a migration guide, a decision record — is exempt as a
//! whole, and every exemption is counted where the reader can see it
//! (the docdup ledger discipline: never silent). Two witnesses, either
//! one enough: the path convention the ecosystem already speaks
//! (Keep a Changelog, GNU NEWS, ADR directories), and the SHAPE of a
//! version-indexed ledger — headings that carry a version, a date or
//! `Unreleased`. Only Markdown can hold the role: a code file narrates
//! nothing by job. Deliberately no regex: the two matchers are a
//! digit walk each.

use crate::graph::ladder::md::slug::atx_heading;
use crate::graph::md::content_lines;
use crate::scan::lang::Lang;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Witness {
    /// The file's stem or a directory on its path says so.
    Path,
    /// At least half its level-2/3 headings are version-indexed.
    Ledger,
}

impl Witness {
    pub fn name(self) -> &'static str {
        match self {
            Witness::Path => "path",
            Witness::Ledger => "ledger",
        }
    }
}

/// File stems (lower-cased, extension dropped) that name the role.
const STEMS: &[&str] = &[
    "changelog",
    "changes",
    "history",
    "news",
    "releases",
    "release-notes",
    "release_notes",
    "releasenotes",
    "migration",
    "migrations",
    "migrating",
    "upgrading",
    "upgrade",
    "breaking-changes",
    "breaking_changes",
];

/// Directory names (any depth) whose documents carry the role.
const DIRS: &[&str] = &[
    "adr",
    "adrs",
    "decisions",
    "changelogs",
    "releases",
    "release-notes",
    "release_notes",
];

/// Whether a document holds the changelog role, and by which witness.
pub fn changelog_role(rel: &str, after: &str, lang: Lang) -> Option<Witness> {
    if lang != Lang::Markdown {
        return None;
    }
    if by_path(rel) {
        return Some(Witness::Path);
    }
    ledger_shape(after).then_some(Witness::Ledger)
}

fn by_path(rel: &str) -> bool {
    let p = Path::new(rel);
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let named = STEMS.contains(&stem.as_str()) || stem.starts_with("adr-");
    let in_dir = p.parent().into_iter().flat_map(Path::components).any(|c| {
        c.as_os_str()
            .to_str()
            .is_some_and(|d| DIRS.contains(&d.to_ascii_lowercase().as_str()))
    });
    named || in_dir
}

/// At least three headings, and at least half of the level-2/3 ones
/// version-indexed (level 1 is the document's title, level 4+ the
/// entries under a version).
fn ledger_shape(after: &str) -> bool {
    let heads: Vec<(usize, &str)> = content_lines(after)
        .into_iter()
        .filter(|(_, _, mask)| !mask.first().copied().unwrap_or(false))
        .filter_map(|(_, line, _)| {
            let t = line.trim_start();
            let level = t.chars().take_while(|c| *c == '#').count();
            atx_heading(t).map(|h| (level, h))
        })
        .collect();
    if heads.len() < 3 {
        return false;
    }
    let sub: Vec<&str> = heads
        .iter()
        .filter(|(l, _)| (2..=3).contains(l))
        .map(|(_, h)| *h)
        .collect();
    let indexed = sub.iter().filter(|h| versioned(h)).count();
    !sub.is_empty() && indexed * 2 >= sub.len()
}

/// A heading that indexes a version: `1.6.0` / `v2.3`, an ISO date, or
/// the Keep-a-Changelog `Unreleased` (and its Chinese surface).
fn versioned(heading: &str) -> bool {
    let low = heading.to_lowercase();
    low.contains("unreleased")
        || heading.contains("未发布")
        || has_semver(heading)
        || has_iso_date(heading)
}

/// `digits . digits` anywhere.
fn has_semver(s: &str) -> bool {
    let b = s.as_bytes();
    (1..b.len().saturating_sub(1))
        .any(|i| b[i] == b'.' && b[i - 1].is_ascii_digit() && b[i + 1].is_ascii_digit())
}

/// `dddd-dd-dd` anywhere.
fn has_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    let digit = |i: usize| b.get(i).is_some_and(u8::is_ascii_digit);
    let dash = |i: usize| b.get(i) == Some(&b'-');
    (0..b.len()).any(|i| {
        (0..4).all(|k| digit(i + k))
            && dash(i + 4)
            && digit(i + 5)
            && digit(i + 6)
            && dash(i + 7)
            && digit(i + 8)
            && digit(i + 9)
    })
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone/role.rs"]
mod tests;
