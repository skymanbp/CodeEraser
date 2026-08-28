//! The detached temp worktree one trend point is judged in — plus the
//! submodule seats it needs. `git worktree add` renders a gitlink as
//! an EMPTY directory, so a commit whose tests ride at `cli/tests` as
//! the CodeEraser-tests submodule (plan v2.18) would score without its
//! tests and the trajectory would step where the tree did not; each
//! gitlink of the commit therefore gets a nested worktree of the
//! superproject's own submodule checkout at the recorded sha — offline
//! and deterministic (the checkout must exist; a missing one is a
//! named refusal, never a silent shortfall), torn down before the tree.

use crate::churn;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static WT_SEQ: AtomicU64 = AtomicU64::new(0);

/// A detached worktree that tears itself down. The name (which also
/// names git's worktree metadata dir) is sha+pid+SEQ-unique: two
/// threads of one process measuring the same sha — or two test repos
/// whose seeded commits hash identically — must not race one path,
/// and a crash-leaked dir stays `git worktree prune`-able.
pub(super) struct Worktree {
    root: PathBuf,
    pub(super) path: PathBuf,
    /// (submodule checkout in `root`, its seat in `path`), seated order
    seats: Vec<(PathBuf, PathBuf)>,
}

impl Worktree {
    pub(super) fn add(root: &Path, sha: &str) -> Result<Self> {
        let seq = WT_SEQ.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ce-trend-{}-{}-{seq}",
            &sha[..12],
            std::process::id()
        ));
        let p = path.to_str().context("worktree path not utf8")?;
        churn::git(root, &["worktree", "add", "--detach", p, sha])?;
        let mut wt = Self {
            root: root.to_path_buf(),
            path,
            seats: Vec::new(),
        };
        for (rel, sub) in gitlinks(&wt.path)? {
            let home = root.join(&rel);
            anyhow::ensure!(
                home.join(".git").exists(),
                "trend: submodule {rel} is not checked out under {} — `git submodule update --init` first",
                root.display()
            );
            let seat = wt.path.join(&rel);
            let s = seat.to_str().context("seat path not utf8")?;
            churn::git(&home, &["worktree", "add", "--detach", s, &sub])?;
            wt.seats.push((home, seat));
        }
        Ok(wt)
    }
}

/// The commit's gitlinks as (path, sha): the mode-160000 rows of
/// `ls-tree -r HEAD`.
fn gitlinks(wt: &Path) -> Result<Vec<(String, String)>> {
    let out = churn::git(wt, &["ls-tree", "-r", "HEAD"])?;
    Ok(out
        .lines()
        .filter_map(|l| {
            let (meta, path) = l.split_once('\t')?;
            let mut f = meta.split(' ');
            (f.next()? == "160000").then_some(())?;
            f.next()?;
            Some((path.to_string(), f.next()?.to_string()))
        })
        .collect())
}

impl Drop for Worktree {
    fn drop(&mut self) {
        for (home, seat) in self.seats.drain(..).rev() {
            remove(&home, &seat, "seat");
        }
        remove(&self.root, &self.path, "worktree");
    }
}

/// `git worktree remove --force`, with the fallback Drop needs: a
/// swallowed failure left BOTH residues silently — the tree (with its
/// .ce) in the temp dir and the .git/worktrees metadata entry
/// accumulating in the real repo. Fall back to a plain directory
/// remove, prune the metadata, and say one line — a leak someone can
/// see is prune-able; a silent one just grows.
fn remove(repo: &Path, path: &Path, what: &str) {
    let named = path.to_string_lossy().into_owned();
    if churn::git(repo, &["worktree", "remove", "--force", &named]).is_err() {
        let _ = std::fs::remove_dir_all(path);
        let _ = churn::git(repo, &["worktree", "prune"]);
        eprintln!("ce trend: {what} {named} needed a filesystem teardown");
    }
}
