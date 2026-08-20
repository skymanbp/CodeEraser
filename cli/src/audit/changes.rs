//! What changed, in CE's OWN path vocabulary.
//!
//! The Stop audit compares git's answer against `dedup::analyze`'s
//! block paths, which are `walk::rel_str` spellings: forward slashes,
//! relative to the ce root, one path per row, never quoted. Git
//! answers in none of those by default, and every mismatch is a
//! SILENT gate bypass — `changed.contains(&b.a_file)` simply returns
//! false and the Stop passes. This module is the one translation
//! layer; the three spellings it normalizes were three separate
//! confirmed bypasses (review 2026-08-19):
//!
//!   * base — `git -C sub diff` prints repo-root-relative paths, so a
//!     `ce.toml` anchored in a monorepo package compared `pkg/app/a.rs`
//!     against dedup's `a.rs`. `--relative` rebases AND scopes.
//!   * spelling — git C-quotes "unusual" paths. `core.quotePath=false`
//!     (churn::git) exempts the non-ASCII half ONLY; control chars,
//!     `"` and `\` stay quoted whatever that setting says, which the
//!     first fix here missed. `-z` is the whole answer: NUL-terminated
//!     records, never quoted — the idiom fourclass::session already
//!     used, now on the enforcement side too.
//!   * rows — a rename is ONE column, `old => new`; `--no-renames`
//!     splits it into rows whose paths are real.

use std::io::Read as _;
use std::path::Path;

/// Files above this count 0 lines (still entering the changed set —
/// the numstat `-` stance for binaries). The untracked leg READS to
/// count, so an unbounded read is an availability hole, not just
/// slowness: a stray 2 GB csv measured 1.96 GB RSS on a Stop.
const READ_CAP: u64 = 4 << 20;

/// Paths ce owns rather than judges: its own state directory, at ANY
/// depth (a vendored subpackage carries `vendor/pkg/.ce/`). Both legs
/// filter through here — the prefix test used to live on the
/// untracked leg alone, so a repo that never gitignored `.ce/` fed
/// the guard's own observe feed back in as user entropy.
fn ce_owned(path: &str) -> bool {
    path.split('/').any(|c| c == ".ce")
}

/// The `-z` record stream both legs read: NUL-separated, empties
/// dropped. One iterator because the two loops around it were
/// byte-shaped twins the moment they both stopped using `lines()`.
fn records(text: &str) -> impl Iterator<Item = &str> {
    text.split('\0').filter(|r| !r.is_empty())
}

/// The diff base: `HEAD`, or git's empty tree when HEAD is unborn.
/// A brand-new project — this product's headline scenario — has no
/// HEAD until its first commit, and `diff HEAD` then fails outright,
/// taking the whole audit down the fail-open path with no feed entry
/// at all. The empty-tree fallback is the fourclass side's own idiom
/// (`fourclass::session::commit_pairs`, attack review F12); the id is
/// asked of git rather than hard-coded because it differs between
/// sha1 and sha256 repositories. None = not a git repo.
pub fn base_rev(root: &Path) -> Option<String> {
    if crate::churn::git(root, &["rev-parse", "--verify", "-q", "HEAD"]).is_ok() {
        return Some("HEAD".into());
    }
    crate::churn::git(root, &["hash-object", "-t", "tree", "--stdin"])
        .ok()
        .map(|s| s.trim().to_string())
}

/// numstat over `tail` (the caller's base rev, or `--cached`) in ce's
/// path vocabulary. Git computes the arithmetic itself, so this leg
/// reads nothing and needs no scope filter beyond `.ce/`.
pub fn diff(root: &Path, tail: &[&str]) -> Option<(i64, Vec<String>)> {
    let mut args = vec!["diff", "--numstat", "--relative", "--no-renames", "-z"];
    args.extend_from_slice(tail);
    let text = crate::churn::git(root, &args).ok()?;
    let mut net = 0i64;
    let mut files = Vec::new();
    // `added\tdeleted\tpath`, the path RAW: splitn(3) keeps a tab
    // inside the name out of the separator's reach.
    for rec in records(&text) {
        let mut cols = rec.splitn(3, '\t');
        let (Some(a), Some(d), Some(path)) = (cols.next(), cols.next(), cols.next()) else {
            continue; // one malformed row must not void the whole audit
        };
        let path = path.replace('\\', "/");
        if ce_owned(&path) {
            continue;
        }
        // '-' marks binary files: count the file, skip the arithmetic
        if let (Ok(a), Ok(d)) = (a.parse::<i64>(), d.parse::<i64>()) {
            net += a - d;
        }
        files.push(path);
    }
    Some((net, files))
}

/// Untracked-but-not-ignored files with their line counts — `git diff
/// HEAD` cannot see a brand-new file, exactly what a probe-bypassing
/// shell write leaves behind.
///
/// Scoped through the SAME exclusion model every other whole-tree
/// reader uses (the language gate then `walk::in_scope`), unlike the
/// diff leg above: here every listed file is READ to count its lines,
/// so an un-ignored `node_modules/` or a stray multi-GB csv turned
/// every Stop into a full-tree read. A file ce would never index also
/// can never match a dedup block, so the scope costs the gate nothing
/// — it only stops the audit paying for what it cannot use.
pub fn untracked(root: &Path, excludes: &[String]) -> Option<(i64, Vec<String>)> {
    let args = ["ls-files", "--others", "--exclude-standard", "-z"];
    let text = crate::churn::git(root, &args).ok()?;
    let mut net = 0i64;
    let mut files = Vec::new();
    // raw paths again: a newline inside one no longer ends the record
    // early, as `lines()` let it
    for rec in records(&text) {
        let path = rec.replace('\\', "/");
        let full = root.join(&path);
        if path.is_empty()
            || ce_owned(&path)
            || crate::scan::lang::Lang::from_path(&full).is_none()
            || !crate::scan::walk::in_scope(root, &full, excludes)
        {
            continue;
        }
        net += line_count(&full);
        files.push(path);
    }
    Some((net, files))
}

/// Lines of a file ce would index, 0 for anything unreadable, binary,
/// or past the read cap — the numstat `-` stance, applied to a leg
/// that has to open the file to know.
///
/// The cap is enforced by the READ, not by a metadata check beforehand:
/// a stat-then-read pair is a race a growing file wins (and a symlink
/// or /dev/zero has no useful size at all), so the reader itself is
/// bounded and an over-cap file counts 0 rather than being trusted.
fn line_count(path: &Path) -> i64 {
    let Ok(file) = std::fs::File::open(path) else {
        return 0;
    };
    let mut buf = Vec::new();
    // cap + 1: reading one byte past tells us it was truncated
    if file.take(READ_CAP + 1).read_to_end(&mut buf).is_err() || buf.len() as u64 > READ_CAP {
        return 0;
    }
    match String::from_utf8(buf) {
        Ok(text) => text.lines().count() as i64,
        Err(_) => 0, // binary: counted as a changed file, zero lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `.ce` is ce's own state at any depth — the prefix-only test let
    /// a vendored `vendor/pkg/.ce/index.db` through on both legs.
    #[test]
    fn ce_owned_matches_any_component_not_just_the_prefix() {
        assert!(ce_owned(".ce/observe.ndjson"));
        assert!(ce_owned("vendor/pkg/.ce/index.db"));
        assert!(!ce_owned("src/.certs/key.rs"), "prefix of a name is not it");
        assert!(!ce_owned("a.rs"));
    }

    /// The `-z` record grammar, pinned where a filesystem cannot go:
    /// Windows refuses to create a tab-bearing path, but git happily
    /// emits one from a tree object, and `core.quotePath=false` does
    /// NOT unquote it — only -z does. A `splitn(3)` keeps that tab
    /// inside the path instead of reading it as a third separator.
    #[test]
    fn a_numstat_record_keeps_tabs_inside_the_path() {
        let rec = "1\t0\tweird\tname.rs";
        let mut cols = rec.splitn(3, '\t');
        assert_eq!(cols.next(), Some("1"));
        assert_eq!(cols.next(), Some("0"));
        assert_eq!(
            cols.next(),
            Some("weird\tname.rs"),
            "the path is whatever follows the two counts, tabs and all"
        );
    }
}
