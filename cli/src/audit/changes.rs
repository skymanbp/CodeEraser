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
//!   * spelling — `core.quotePath` (git's default) C-quotes any
//!     non-ASCII path; disabled at the ONE runner in churn::git.
//!   * rows — a rename is ONE column, `old => new`; `--no-renames`
//!     splits it into rows whose paths are real.

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
    let mut args = vec!["diff", "--numstat", "--relative", "--no-renames"];
    args.extend_from_slice(tail);
    let text = crate::churn::git(root, &args).ok()?;
    let mut net = 0i64;
    let mut files = Vec::new();
    for line in text.lines() {
        let mut cols = line.split('\t');
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
    let text = crate::churn::git(root, &["ls-files", "--others", "--exclude-standard"]).ok()?;
    let mut net = 0i64;
    let mut files = Vec::new();
    for line in text.lines() {
        let path = line.trim().replace('\\', "/");
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
fn line_count(path: &Path) -> i64 {
    match std::fs::metadata(path) {
        Ok(m) if m.len() > READ_CAP => 0,
        _ => std::fs::read_to_string(path)
            .map(|s| s.lines().count() as i64)
            .unwrap_or(0),
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
}
