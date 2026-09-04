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

use std::path::Path;

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

/// The judged-language gate, now shared by BOTH legs. It lived on the
/// untracked leg alone, so one `.js` file counted 0 net LOC while
/// untracked and its full numstat the moment it was committed — the
/// same file, two net-LOC universes, told apart by nothing but its git
/// status, inside the ledger the deny promotion is argued from.
/// Extension-only, so a `&str` is the whole input: no `root.join`, no
/// stat — the paths git already handed back are enough.
fn judged(rel: &str) -> bool {
    crate::scan::lang::Lang::judged_path(Path::new(rel)).is_some()
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
    // Repo-existence is its own question, asked FIRST. It used to be
    // inferred from the fallback, and `hash-object` needs no
    // repository — it answers the empty-tree id outside one, exit 0.
    // So every Stop in a non-repo directory looked like a repo whose
    // diff had failed: `.ce/` created there and a `degraded: true`
    // line written into the promotion ledger for a degradation that
    // never happened.
    crate::churn::git(root, &["rev-parse", "--git-dir"]).ok()?;
    if crate::churn::git(root, &["rev-parse", "--verify", "-q", "HEAD"]).is_ok() {
        return Some("HEAD".into());
    }
    crate::churn::git(root, &["hash-object", "-t", "tree", "--stdin"])
        .ok()
        .map(|s| s.trim().to_string())
}

/// One `-z` numstat record — `added\tdeleted\tpath`, the path RAW, so
/// `splitn(3)` keeps a tab inside the name out of the separator's
/// reach. None = a malformed row (one must not void the whole audit)
/// or a path outside the judged universe.
fn numstat_row(rec: &str) -> Option<(i64, String)> {
    let mut cols = rec.splitn(3, '\t');
    let (a, d, path) = (cols.next()?, cols.next()?, cols.next()?);
    let path = path.replace('\\', "/");
    if ce_owned(&path) || !judged(&path) {
        return None;
    }
    // '-' marks binary files: count the file, skip the arithmetic
    let net = match (a.parse::<i64>(), d.parse::<i64>()) {
        (Ok(a), Ok(d)) => a - d,
        _ => 0,
    };
    Some((net, path))
}

/// numstat over `tail` (the caller's base rev, or `--cached`) in ce's
/// path vocabulary. Git computes the arithmetic itself, so this leg
/// still reads nothing — but it answers in the SAME judged universe
/// `untracked` does (see `judged`). Only the scope test stays
/// untracked-only: it canonicalizes, and a row here may name a file
/// the diff DELETED, which has no path left to canonicalize. The
/// superproject's diff reports a submodule as ONE gitlink row (`0 0
/// cli/tests`), which is the whole of what this root sees of it: the
/// child's own edits are the child's own audit's (plan v2.18 step
/// #12, audit.rs asks each gated submodule in its own git), and a
/// submodule without a gate is a reader here, measured by nobody.
pub fn diff(root: &Path, tail: &[&str]) -> Option<(i64, Vec<String>)> {
    numstat(root, tail)
}

fn numstat(repo: &Path, tail: &[&str]) -> Option<(i64, Vec<String>)> {
    let mut args = vec!["diff", "--numstat", "--relative", "--no-renames", "-z"];
    args.extend_from_slice(tail);
    let text = crate::churn::git(repo, &args).ok()?;
    let mut net = 0i64;
    let mut files = Vec::new();
    for (n, path) in records(&text).filter_map(numstat_row) {
        net += n;
        files.push(path);
    }
    Some((net, files))
}

/// Untracked-but-not-ignored files with their line counts — `git diff
/// HEAD` cannot see a brand-new file, exactly what a probe-bypassing
/// shell write leaves behind.
///
/// Scoped through the SAME exclusion model every other whole-tree
/// reader uses: the language gate (shared with the diff leg since the
/// universe split) and then the walk's `Scope`, which only this leg
/// pays for — here every listed file is READ to count its lines, so
/// an un-ignored `node_modules/` or a stray multi-GB csv turned every
/// Stop into a full-tree read. A file ce would never index also can
/// never match a dedup block, so the scope costs the gate nothing —
/// it only stops the audit paying for what it cannot use. One ce
/// root, its OWN files (plan v2.18 step #12): a declared submodule is
/// a reader of this tree and its edits are its own gate's to audit —
/// the Stop asks each gated one (audit.rs) — and git lists no file of
/// a nested repository anyway.
pub fn untracked(root: &Path, excludes: &[String]) -> Option<(i64, Vec<String>)> {
    let mut scope = crate::scan::walk::Scope::new(root, excludes).ok()?;
    let args = ["ls-files", "--others", "--exclude-standard", "-z"];
    let mut net = 0i64;
    let mut files = Vec::new();
    let text = crate::churn::git(root, &args).ok()?;
    // raw paths again: a newline inside one no longer ends the
    // record early, as `lines()` let it
    for rec in records(&text) {
        let path = rec.replace('\\', "/");
        let full = root.join(&path);
        // `judged`: the scan-only arm (plan v2.5) never enters the
        // Stop audit's changed-file universe — through the shared
        // predicate now, so the two legs cannot drift apart again
        if ce_owned(&path) || !judged(&path) || !scope.contains(&full) {
            continue;
        }
        net += line_count(&full);
        files.push(path);
    }
    Some((net, files))
}

/// Lines of a file ce would index, 0 for anything unreadable, binary,
/// or past the read cap (a stray 2 GB csv measured 1.96 GB RSS on a
/// Stop) — the numstat `-` stance, applied to a leg that has to open
/// the file to know. The bounded reader is the tombstone leg's
/// (tombstone::texts::read_capped): one reader, one cap.
fn line_count(path: &Path) -> i64 {
    crate::tombstone::texts::read_capped(path).map_or(0, |text| text.lines().count() as i64)
}

#[cfg(test)]
#[path = "../../tests/unit/audit/changes.rs"]
mod tests;
