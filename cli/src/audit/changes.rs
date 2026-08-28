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

/// The seated declared submodules (gitmodules.rs) — one level, by
/// declaration only. The superproject's `git diff`/`ls-files` report
/// a submodule as ONE gitlink row (`0 0 cli/tests`, and no untracked
/// file of the child at all), so a tracked edit or a brand-new clone
/// inside it was invisible to both legs while `dedup::analyze` walked
/// the very same files — the module's own defect class, two spellings
/// of one universe (plan §4.2: the Stop audit is the backstop for
/// Bash writes). Each seated child is asked the same question and its
/// answer prefixed `{sub}/`, the ce-root spelling the block paths use.
fn subs(root: &Path) -> Vec<String> {
    crate::gitmodules::seated(root)
}

/// numstat over `tail` (the caller's base rev, or `--cached`) in ce's
/// path vocabulary — the root, then every seated submodule. Git
/// computes the arithmetic itself, so this leg still reads nothing —
/// but it answers in the SAME judged universe `untracked` does (see
/// `judged`). Only the scope test stays untracked-only: it
/// canonicalizes, and a row here may name a file the diff DELETED,
/// which has no path left to canonicalize. `--cached` forwards to a
/// child verbatim; a rev is the PARENT's HEAD-or-empty-tree and names
/// nothing in the child's odb, so it is re-resolved per repository.
pub fn diff(root: &Path, tail: &[&str]) -> Option<(i64, Vec<String>)> {
    let (mut net, mut files) = numstat(root, tail)?;
    for sub in subs(root) {
        let home = root.join(&sub);
        let child_tail = match tail {
            ["--cached"] => String::from("--cached"),
            _ => base_rev(&home)?,
        };
        let (n, f) = numstat(&home, &[child_tail.as_str()])?;
        net += n;
        files.extend(f.iter().map(|p| format!("{sub}/{p}")));
    }
    Some((net, files))
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
/// it only stops the audit paying for what it cannot use. The root,
/// then every seated submodule (`subs`), the PARENT's scope applied
/// to the child's paths: one ce root, one judgment.
pub fn untracked(root: &Path, excludes: &[String]) -> Option<(i64, Vec<String>)> {
    let mut scope = crate::scan::walk::Scope::new(root, excludes).ok()?;
    let args = ["ls-files", "--others", "--exclude-standard", "-z"];
    let mut net = 0i64;
    let mut files = Vec::new();
    let repos = std::iter::once(String::new()).chain(subs(root).into_iter().map(|s| s + "/"));
    for prefix in repos {
        let text = crate::churn::git(&root.join(&prefix), &args).ok()?;
        // raw paths again: a newline inside one no longer ends the
        // record early, as `lines()` let it
        for rec in records(&text) {
            let path = format!("{prefix}{}", rec.replace('\\', "/"));
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
        assert_eq!(
            numstat_row(rec),
            Some((1, String::from("weird\tname.rs"))),
            "and the row parser keeps it too"
        );
    }

    /// The two legs answer in ONE universe. Every row here is one git
    /// would print for a real file: without the judged gate on this
    /// leg, `app.js` and `ce.toml` counted their full numstat once
    /// committed and exactly 0 while untracked.
    #[test]
    fn numstat_rows_share_the_untracked_legs_judged_universe() {
        assert_eq!(numstat_row("11\t0\ta.rs"), Some((11, "a.rs".into())));
        assert_eq!(
            numstat_row("-\t-\ta.rs"),
            Some((0, "a.rs".into())),
            "binary"
        );
        assert_eq!(numstat_row("3\t2\tapp.js"), None, "scan-only arm");
        assert_eq!(numstat_row("3\t2\tce.toml"), None, "no language at all");
        assert_eq!(numstat_row("1\t0\t.ce/observe.ndjson"), None, "ce's own");
        assert_eq!(numstat_row("1\t0"), None, "malformed rows still skip");
    }
}
