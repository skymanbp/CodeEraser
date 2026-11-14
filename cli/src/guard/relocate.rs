//! Relocation credit for the duplicate-write class (v2.35 guard lane,
//! GD4) — the measuring half. Asked only on the would-refuse path (the
//! probe left novel matches): for each match, the occurrences of its
//! matched token run in HEAD and in the tree the write leaves, over
//! C = every path `diff HEAD` names (A / M / D / T — staged additions
//! included), U = untracked files, and the target T as applied, in
//! every fingerprinting language (the probe's candidates are not
//! language-filtered). Unchanged files count alike on both sides, so
//! the sums are the tree-wide counts; the verdict is the core's
//! (relocate/1, CE.Relocate). Two git reads spawned together, one
//! `cat-file --batch` for the HEAD blobs (the audit's reader and its
//! caps), every answer bounded: no git, no HEAD, a cap, an unread blob,
//! an Edit that does not apply or a core that does not answer is no
//! credit — the refusal as it stood, the reason in the feed
//! (`relocation`). An allowance never rides an unanswered question.

use super::applied::Applied;
use super::envelope::Envelope;
use crate::daemon::{client, proto::Request};
use crate::dedup::{relocate, tokens};
use crate::fourclass::session::PathPair;
use crate::scan::lang::Lang;
use crate::tombstone::texts::{self, Side};
use std::collections::BTreeSet;
use std::path::Path;

/// Bytes the HEAD and working texts of one measurement may total; past
/// it the write is not credited (the per-blob READ_CAP and the file
/// cap PAIR_CAP are the audit reader's own).
const RELOC_BYTES: usize = 16 << 20;

/// What the stage concluded: which matches are relocations (indices
/// into the matches it was handed), whether every one is (then the
/// class does not refuse), or why it could not answer.
pub(super) struct Credit {
    pub moved: Vec<usize>,
    pub all: bool,
    pub why_not: Option<String>,
}

impl Credit {
    fn none(why: impl Into<String>) -> Self {
        Credit {
            moved: Vec::new(),
            all: false,
            why_not: Some(why.into()),
        }
    }
}

/// The stage over one would-refuse write.
pub(super) fn credit(
    root: &Path,
    env: &Envelope,
    probed: &str,
    applied: Option<&Applied>,
    matches: &[serde_json::Value],
) -> Credit {
    match counted(root, env, probed, applied, matches) {
        Err(why) => Credit::none(why),
        Ok(rows) => match judged(root, &rows) {
            Ok(v) => Credit {
                moved: v.moved,
                all: v.all,
                why_not: None,
            },
            Err(why) => Credit::none(format!("core: {why}")),
        },
    }
}

/// The core's verdict over the daemon's held link.
fn judged(root: &Path, rows: &[[u64; 2]]) -> Result<relocate::Verdict, String> {
    let reply = client::relay(
        root,
        &Request::Relocate {
            rows: rows.to_vec(),
        },
    )?;
    relocate::consume(&reply, rows.len())
}

/// One `[occHead, occNow]` row per match, or why there are none.
fn counted(
    root: &Path,
    env: &Envelope,
    probed: &str,
    applied: Option<&Applied>,
    matches: &[serde_json::Value],
) -> Result<Vec<[u64; 2]>, String> {
    let target = Path::new(&env.tool_input.file_path);
    let lang = Lang::from_path(target)
        .filter(|l| l.fingerprints())
        .ok_or("language")?;
    let applied = applied.ok_or("unapplied")?;
    let runs = runs(probed, lang, matches)?;
    let tree = Tree::read(root, &crate::scan::walk::rel_str(root, target))?;
    let mut rows: Vec<[u64; 2]> = runs.iter().map(|r| [tree.head(r), tree.now(r)]).collect();
    let own = hashes(applied.text.as_bytes(), lang)?;
    for (row, run) in rows.iter_mut().zip(&runs) {
        row[1] += relocate::occurrences(&own, run);
    }
    Ok(rows)
}

/// Each match's run off the probed content's own token stream: the
/// daemon names where in the content it matched (`at`) and how long.
fn runs(probed: &str, lang: Lang, matches: &[serde_json::Value]) -> Result<Vec<Vec<u64>>, String> {
    let stream = hashes(probed.as_bytes(), lang)?;
    matches
        .iter()
        .map(|m| {
            let at = m["at"].as_u64().ok_or("no snippet offset")? as usize;
            let len = m["tokens"].as_u64().ok_or("no run length")? as usize;
            stream
                .get(at..at + len)
                .map(<[u64]>::to_vec)
                .ok_or_else(|| "snippet offset past the content".to_string())
        })
        .collect()
}

fn hashes(src: &[u8], lang: Lang) -> Result<Vec<u64>, String> {
    Ok(tokens::stream(src, lang)
        .map_err(|e| format!("tokenise: {e}"))?
        .iter()
        .map(|t| t.hash)
        .collect())
}

/// The changed part of the tree as token streams: HEAD's side (C and
/// T when HEAD has them) and the working side (C ∖ deleted, U, never T
/// — the write's own text is counted from its applied form).
struct Tree {
    head: Vec<Vec<u64>>,
    now: Vec<Vec<u64>>,
}

impl Tree {
    fn read(root: &Path, target: &str) -> Result<Tree, String> {
        let (changed, untracked) = std::thread::scope(|s| {
            let diff = s.spawn(|| {
                let args = ["diff", "--name-status", "-z", "--no-renames", "HEAD"];
                crate::churn::git(root, &args)
            });
            let others = s.spawn(|| {
                let args = ["ls-files", "--others", "--exclude-standard", "-z"];
                crate::churn::git(root, &args)
            });
            (diff.join(), others.join())
        });
        let changed = changed
            .map_err(|_| "git diff panicked")?
            .map_err(|e| format!("git: {e}"))?;
        let untracked = untracked
            .map_err(|_| "git ls-files panicked")?
            .map_err(|e| format!("git: {e}"))?;
        let pairs = pairs(root, &changed, &untracked, target);
        if pairs.len() > texts::PAIR_CAP {
            return Err(format!("cap_files: {} files differ from HEAD", pairs.len()));
        }
        let (loaded, unread) =
            texts::load(root, &pairs, Side::Rev("HEAD"), Side::Worktree).ok_or("git: cat-file")?;
        if unread > 0 {
            return Err(format!("unread: {unread} file(s)"));
        }
        let bytes: usize = loaded.iter().map(|l| l.before.len() + l.after.len()).sum();
        if bytes > RELOC_BYTES {
            return Err(format!("cap_bytes: {bytes}"));
        }
        let mut tree = Tree {
            head: Vec::new(),
            now: Vec::new(),
        };
        for l in &loaded {
            tree.head.push(hashes(l.before.as_bytes(), l.lang)?);
            tree.now.push(hashes(l.after.as_bytes(), l.lang)?);
        }
        Ok(tree)
    }

    fn head(&self, run: &[u64]) -> u64 {
        self.head
            .iter()
            .map(|s| relocate::occurrences(s, run))
            .sum()
    }

    fn now(&self, run: &[u64]) -> u64 {
        self.now.iter().map(|s| relocate::occurrences(s, run)).sum()
    }
}

/// `(HEAD side, working side)` per file: C from `diff --name-status -z`
/// (an added path has no HEAD side, a deleted one no working side), U
/// from `ls-files --others` (working side only), and T with its HEAD
/// side only (when HEAD has it) — in scope, of a fingerprinting
/// language. A declared submodule's files never appear: the diff names
/// its gitlink, and ls-files does not descend into it.
fn pairs(root: &Path, changed: &str, untracked: &str, target: &str) -> Vec<PathPair> {
    let mut scope = crate::scan::walk::Scope::new(root, &[]).ok();
    let mut keep = |rel: &str| {
        Lang::from_path(Path::new(rel)).is_some_and(|l| l.fingerprints())
            && scope.as_mut().is_some_and(|s| s.contains(Path::new(rel)))
    };
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut toks = changed.split('\0').filter(|t| !t.is_empty());
    while let (Some(status), Some(rel)) = (toks.next(), toks.next()) {
        if !keep(rel) || !seen.insert(rel.to_string()) {
            continue;
        }
        let head = (!status.starts_with('A')).then(|| rel.to_string());
        let now = (!status.starts_with('D') && rel != target).then(|| rel.to_string());
        out.push((head, now));
    }
    for rel in untracked.split('\0').filter(|t| !t.is_empty()) {
        // the target among them is marked seen (no HEAD side to read)
        // and counted from its applied text
        if keep(rel) && seen.insert(rel.to_string()) && rel != target {
            out.push((None, Some(rel.to_string())));
        }
    }
    if !seen.contains(target) && keep(target) && root.join(target).is_file() {
        // tracked and unchanged: HEAD has it (its working text is the
        // applied one); an untracked or absent target has no HEAD side
        out.push((Some(target.to_string()), None));
    }
    out
}
