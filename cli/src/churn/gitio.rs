//! This family's git face over proc::git_output (pub(crate) since
//! M6 S3c: the structure staleness join reuses it). Split to its own
//! leaf in the headroom sprint: survival.rs importing it THROUGH
//! mod.rs made the module family a cycle the graph axis itself
//! billed. Its stance: the error carries git's own stderr — a bare
//! "failed" cost the trend battery a debugging cycle and would reach
//! users via failed[].

use anyhow::{Context, Result};
use std::path::Path;

pub(crate) fn git(root: &Path, args: &[&str]) -> Result<String> {
    let out = crate::proc::git_output(root, args).context("git")?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("git {args:?} failed: {}", err.trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// A diff base: `rev` itself when it resolves, else git's empty tree —
/// an unborn HEAD (a brand-new project, this product's headline
/// scenario) or a root commit's missing parent (attack review
/// 2026-08-11 F12: `sha^..sha` fails on the root, and a zero-line
/// commit was the reading). The empty tree's id is asked of git, since
/// it differs between sha1 and sha256 repositories. None = not a git
/// repository, asked on its own: `hash-object` needs no repository and
/// answers the empty tree's id outside one too, so a non-repo would
/// read as a repo whose diff failed (the Stop audit wrote `.ce/` and a
/// `degraded: true` line into the promotion ledger for that).
pub(crate) fn rev_or_empty_tree(root: &Path, rev: &str) -> Option<String> {
    if git(root, &["rev-parse", "--verify", "-q", rev]).is_ok() {
        return Some(rev.to_string());
    }
    git(root, &["rev-parse", "--git-dir"]).ok()?;
    git(root, &["hash-object", "-t", "tree", "--stdin"])
        .ok()
        .map(|s| s.trim().to_string())
}

/// `HEAD`'s gitlinks as (path, sha): the mode-160000 rows of
/// `ls-tree -r -z` — `-z` because every other list reader in the
/// crate is literal (`-z`) and a C-quoted path would match nothing,
/// silently (proc.rs). Trend seats them; erase refuses to write below
/// them.
pub(crate) fn gitlinks(repo: &Path) -> Result<Vec<(String, String)>> {
    let out = git(repo, &["ls-tree", "-r", "-z", "HEAD"])?;
    Ok(out
        .split('\0')
        .filter_map(|l| {
            let (meta, path) = l.split_once('\t')?;
            let mut f = meta.split(' ');
            (f.next()? == "160000").then_some(())?;
            f.next()?;
            Some((path.to_string(), f.next()?.to_string()))
        })
        .collect())
}
