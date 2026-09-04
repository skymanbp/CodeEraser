//! The changelog-role detector (user ruling 2026-09-04, spec §三 M5):
//! a document whose JOB is to narrate change — a changelog, release
//! notes, a migration guide, a decision record — is exempt as a
//! whole, and every exemption is counted where the reader can see it
//! (the docdup ledger discipline: never silent). Two witnesses on the
//! file, either one enough: the path convention the ecosystem already
//! speaks (Keep a Changelog, GNU NEWS, ADR directories), and the SHAPE
//! of a version-indexed ledger — headings that carry a version, a date
//! or `Unreleased`. A third witness on the SEGMENT (plan v2.27, user
//! ruling 2026-09-04): a quote run or a section body that is itself a
//! version ledger exempts only itself — the plan book's banner is a
//! ledger and its §4 is a norm, and a file-level answer would have to
//! get one of them wrong. Only Markdown can hold the role: a code file
//! narrates nothing by job. Deliberately no regex: every matcher is a
//! digit walk.

use crate::graph::ladder::md::slug::atx_heading;
use crate::graph::md::content_lines;
use crate::scan::lang::Lang;
use std::path::Path;

/// Which witness exempted a file or a segment; `name` is its feed
/// spelling, read off `NAMES` by discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Witness {
    /// The file's stem or a directory on its path says so.
    Path,
    /// At least half its level-2/3 headings are version-indexed.
    Ledger,
    /// The touched segment is a version ledger by itself: exempt as a
    /// segment, never as a file (plan v2.27).
    Segment,
}

const NAMES: [&str; 3] = ["path", "ledger", "segment"];

impl Witness {
    pub fn name(self) -> &'static str {
        NAMES[self as usize]
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

/// Distinct ledger tokens a segment must carry to be a ledger by
/// itself — the third witness's threshold. The two-corpus replay
/// (docs/FPR-TOMBSTONE.md round 7) left a window of [1, 33]: every
/// true positive's segment carried 0 and the three in-between sites'
/// segments (the plan book's banner) 33, 75 and 77. Three is where a
/// list becomes a ledger — the floor `ledger_shape` already applies
/// to headings — and sits thirty tokens under the nearest banner.
pub const SEGMENT_TOKENS: usize = 3;

type Row<'a> = (usize, &'a str, Vec<bool>);

/// The segment around `line` of a Markdown text and its distinct
/// ledger tokens, as (first line, tokens): the `>` quote run when the
/// line is quoted (a banner is one run), else the section body — the
/// nearest heading at or above the line down to the next heading of
/// any level (the preamble before any heading is a body too). A line
/// the walk skipped (inside a fence) is its own empty segment.
pub fn segment(after: &str, line: usize) -> (usize, usize) {
    let rows = content_lines(after);
    let Some(at) = rows.iter().position(|(no, _, _)| *no == line) else {
        return (line, 0);
    };
    let (lo, hi) = if quoted(&rows[at]) {
        quote_run(&rows, at)
    } else {
        section(&rows, at)
    };
    let text: Vec<&str> = rows[lo..hi].iter().map(|r| r.1).collect();
    (rows[lo].0, ledger_tokens(&text.join("\n")))
}

fn quoted(row: &Row) -> bool {
    row.1.trim_start().starts_with('>')
}

/// Heading level of a content row, or None (a masked line is markup).
fn level(row: &Row) -> Option<usize> {
    if row.2.first().copied().unwrap_or(false) {
        return None;
    }
    let t = row.1.trim_start();
    atx_heading(t).map(|_| t.chars().take_while(|c| *c == '#').count())
}

/// [lo, hi) of the quoted rows around `at` on consecutive lines.
fn quote_run(rows: &[Row], at: usize) -> (usize, usize) {
    // row i extends the run downward from i-1: quoted, and on the
    // very next line (a fence the walk skipped breaks the run too)
    let extends = |i: usize| quoted(&rows[i]) && rows[i - 1].0 + 1 == rows[i].0;
    let mut lo = at;
    while lo > 0 && extends(lo) && quoted(&rows[lo - 1]) {
        lo -= 1;
    }
    let mut hi = at + 1;
    while hi < rows.len() && extends(hi) {
        hi += 1;
    }
    (lo, hi)
}

/// [lo, hi) of the section body holding row `at`.
fn section(rows: &[Row], at: usize) -> (usize, usize) {
    let lo = (0..=at)
        .rev()
        .find(|&i| level(&rows[i]).is_some())
        .unwrap_or(0);
    let hi = (lo + 1..rows.len())
        .find(|&i| level(&rows[i]).is_some())
        .unwrap_or(rows.len());
    (lo, hi)
}

/// Distinct version-ledger tokens in a text: a semver (`1.6.0`, or a
/// `v`-prefixed `v1.6` — the prefix is dropped from the key, so one
/// version spelled both ways is one token), an ISO date, or a 7–40
/// hex commit carrying both a digit and a letter. `§4.2`, `0.57`, a
/// `file:102-103` span, a run number and a hex-looking word are none
/// of these. ASCII-word shaped: a CJK sentence contributes only what
/// it spells in ASCII.
pub fn ledger_tokens(text: &str) -> usize {
    let mut seen: Vec<String> = text
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '-'))
        .map(|word| word.trim_matches(['.', '-']))
        .filter_map(|w| match version(w) {
            Some(v) => Some(v),
            None if (w.len() == 10 && has_iso_date(w)) || commit(w) => Some(w),
            None => None,
        })
        .map(str::to_ascii_lowercase)
        .collect();
    seen.sort_unstable();
    seen.dedup();
    seen.len()
}

/// The version a word spells, without its `v`: three numeric parts,
/// or two behind the prefix.
fn version(w: &str) -> Option<&str> {
    let (prefixed, body) = match w.strip_prefix(['v', 'V']) {
        Some(b) => (true, b),
        None => (false, w),
    };
    let parts: Vec<&str> = body.split('.').collect();
    let numeric = parts
        .iter()
        .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    (numeric && (parts.len() >= 3 || (prefixed && parts.len() == 2))).then_some(body)
}

fn commit(w: &str) -> bool {
    (7..=40).contains(&w.len())
        && w.bytes().all(|b| b.is_ascii_hexdigit())
        && w.bytes().any(|b| b.is_ascii_digit())
        && w.bytes().any(|b| b.is_ascii_alphabetic())
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone/role.rs"]
mod tests;
