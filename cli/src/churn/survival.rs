//! The window's survival leg: of the lines HEAD's blame still gives to
//! a window commit, how many are left (churn = 1 - survival). Split
//! out of churn/mod.rs so the fix's clock-basis pin fits under the
//! 300-line dogfood ceiling without a ratchet-breaking growth step.
//!
//! The parent rule and the commit set it shares with the ledger are
//! recorded in the churn module header — one statement, not two.
//!
//! Counts are joined per (window commit, path at that commit), because
//! two valid diffs of one pair agree on HOW MANY lines a commit added,
//! not on WHICH (v2.35 V1 review). The ledger splits text into lines
//! and does not see a CR before the LF or a missing final newline;
//! blame compares bytes and gives such a line to the commit that only
//! turned its end. So a pair's survivors are min(its blame lines, the
//! ledger's added lines for it), and the rest is counted as
//! `beyond_ledger`: lines git gives to a window commit that the ledger
//! read as unchanged — a line end, a final newline, another diff
//! alignment, or a pair the ledger never read. Survivors can then
//! never exceed the added lines on any history, and the core's check
//! of that stays the refusal for a measurement breaking its own rule.
//! Known limit: when one commit both rewrites line ends and adds lines
//! that are deleted later in the window, `surviving` can count the
//! rewritten lines in their place, by at most that commit's share of
//! `beyond_ledger`.

use super::gitio::git;
use anyhow::Result;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

/// (window commit, root-relative path at that commit) → the lines the
/// ledger added there. A pair the ledger read with nothing added is
/// present with 0: its file is still blamed, and its blame lines are
/// all `beyond_ledger`.
pub(super) type Added = HashMap<(String, String), usize>;

/// The two counts of the survivor pass.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Survival {
    pub surviving: usize,
    pub beyond_ledger: usize,
}

/// Lines at HEAD whose first-parent blame names a window commit, over
/// the files the ledger attributed, joined per (commit, path) against
/// the ledger's added counts.
pub(super) fn survival(root: &Path, window: &[String], added: &Added) -> Result<Survival> {
    let window: HashSet<&str> = window.iter().map(String::as_str).collect();
    // blame names a path as the repository spells it; the ledger as
    // the root does (proc.rs `diff.relative`), so the root's place in
    // the repository comes off — "" at the toplevel
    let prefix = git(root, &["rev-parse", "--show-prefix"])?;
    let prefix = prefix.trim_end_matches(['\r', '\n']);
    let files: BTreeSet<&str> = added.keys().map(|(_, p)| p.as_str()).collect();
    let mut blamed = HashMap::new();
    for (n, file) in files.iter().enumerate() {
        // the blame storm is the expensive half of the window (155 s
        // of the first M5-3h measurement); it ticks per file so the
        // second half of a churn run is not a second silence
        crate::progress::at(crate::progress::Phase::Survival, n, files.len());
        let blame = [
            "blame",
            "--first-parent",
            "--line-porcelain",
            "HEAD",
            "--",
            file,
        ];
        let Ok(out) = git(root, &blame) else {
            // deleted since: zero survivors by definition (a path that
            // became a gitlink would land here too, but never reaches
            // the ledger — a gitlink pair has no judged extension)
            continue;
        };
        blamed_lines(&out, &window, prefix, &mut blamed);
    }
    Ok(settle(&blamed, added))
}

/// Each (commit, path) pair's survivors capped at the ledger's added
/// lines for it; what blame gives beyond that is counted, not dropped.
fn settle(blamed: &HashMap<(String, String), usize>, added: &Added) -> Survival {
    blamed
        .iter()
        .fold(Survival::default(), |s, (pair, &lines)| {
            let kept = lines.min(added.get(pair).copied().unwrap_or(0));
            Survival {
                surviving: s.surviving + kept,
                beyond_ledger: s.beyond_ledger + lines - kept,
            }
        })
}

/// Per (window commit, path at that commit), the porcelain records
/// blame gives it. `--line-porcelain` repeats the header block per
/// line, so this is a per-line count, and every record names its
/// `filename` — the path in that commit, which follows a rename. The
/// blamed content itself arrives TAB-prefixed, which is why a source
/// line spelling a header cannot forge a record. A path outside the
/// root is keyed "", a path no ledger row carries.
fn blamed_lines(
    porcelain: &str,
    window: &HashSet<&str>,
    prefix: &str,
    into: &mut HashMap<(String, String), usize>,
) {
    let mut open: Option<&str> = None;
    for line in porcelain.lines() {
        if let Some(sha) = record_sha(line) {
            open = window.get(sha).copied();
        } else if let Some(name) = line.strip_prefix("filename ") {
            let Some(sha) = open.take() else { continue };
            let path = porcelain_path(name);
            let path = path.strip_prefix(prefix).unwrap_or_default();
            *into.entry((sha.to_string(), path.to_string())).or_default() += 1;
        }
    }
}

/// The commit a record opens with: `<sha> <orig line> <final line>`
/// with an optional group size — a full sha1 or sha256 hex id followed
/// by numbers only, which no other header line (`author …`,
/// `previous <sha> <file>`, `boundary`) is.
fn record_sha(line: &str) -> Option<&str> {
    let (sha, rest) = line.split_once(' ')?;
    let hex = matches!(sha.len(), 40 | 64) && sha.bytes().all(|b| b.is_ascii_hexdigit());
    let numbers = rest.split(' ').all(|t| t.parse::<u64>().is_ok());
    (hex && numbers).then_some(sha)
}

/// A porcelain `filename` as git spells it: verbatim, or C-quoted when
/// it holds a `"`, a `\` or a control byte — `core.quotePath=false`
/// (proc.rs) exempts only the non-ASCII half (audit/changes.rs).
fn porcelain_path(name: &str) -> String {
    let Some(inner) = name.strip_prefix('"').and_then(|n| n.strip_suffix('"')) else {
        return name.to_string();
    };
    let (bytes, mut out, mut i) = (inner.as_bytes(), Vec::new(), 0);
    while i < bytes.len() {
        if bytes[i] != b'\\' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        let (byte, used) = unescape(&bytes[i + 1..]);
        out.push(byte);
        i += 1 + used;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// One C escape after its backslash: git's letter escapes, a three-
/// digit octal byte, or the character itself (`\"`, `\\`).
fn unescape(rest: &[u8]) -> (u8, usize) {
    let letter = |c: u8| {
        b"a\x07b\x08t\tn\nv\x0bf\x0cr\r"
            .chunks(2)
            .find(|p| p[0] == c)
    };
    match rest {
        [d @ b'0'..=b'3', e @ b'0'..=b'7', f @ b'0'..=b'7', ..] => {
            ((d - b'0') * 64 + (e - b'0') * 8 + (f - b'0'), 3)
        }
        [c, ..] => (letter(*c).map_or(*c, |p| p[1]), 1),
        [] => (b'\\', 0),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/churn/survival_tests.rs"]
mod tests;
