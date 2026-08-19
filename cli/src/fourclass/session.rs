//! Session-level four-class plumbing shared by the Stop audit
//! (client side: which pairs changed) and the daemon (report shape).
//! Cross-file relocations stay INFORMATIONAL until a multi-file FPR
//! instrument exists (R-L2-4): claiming a move where there is
//! duplication would hide duplication inside a health signal, so no
//! deny path may lean on this report yet.

use super::batch::BatchClassification;
use crate::scan::lang::Lang;
use std::path::Path;

pub type PathPair = (Option<String>, Option<String>);

/// The session's changed file pairs: working tree vs HEAD, pairing by
/// `git -M -C` (a pure rename is explained by the pairing), filtered
/// to the supported languages. None = not a git repo / git failed.
pub fn head_pairs(root: &Path) -> Option<Vec<PathPair>> {
    diff_pairs(root, &["diff", "--name-status", "-z", "-M", "-C", "HEAD"])
}

/// One commit's file pairs against its first parent (same pairing
/// and language filter as head_pairs) — the churn module's walk.
/// A root commit has no parent; its base is git's empty tree, so its
/// additions are counted instead of silently dropped (attack review
/// 2026-08-11 F12: `sha^..sha` fails on the root and the caller's
/// unwrap_or_default turned that into a zero-line commit while blame
/// still counted the root's surviving lines).
pub fn commit_pairs(root: &Path, sha: &str) -> Option<Vec<PathPair>> {
    let base = rev_id(root, &format!("{sha}^")).or_else(|| empty_tree(root))?;
    diff_pairs(
        root,
        &["diff", "--name-status", "-z", "-M", "-C", &base, sha],
    )
}

/// Run git in `root`, returning stdout on success — the ONE git
/// spawner for the session/judge family (the self-ratchet caught the
/// fourth copy of this shape). stdin is always the null device: no
/// caller feeds input, and `hash-object --stdin` wants EOF.
pub(crate) fn git_stdout(root: &Path, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .stdin(std::process::Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn rev_id(root: &Path, rev: &str) -> Option<String> {
    git_stdout(root, &["rev-parse", "--verify", "-q", rev]).map(|s| s.trim().to_string())
}

/// The empty tree's id from git itself (differs between sha1 and
/// sha256 repositories, so never a hard-coded constant).
fn empty_tree(root: &Path) -> Option<String> {
    git_stdout(root, &["hash-object", "-t", "tree", "--stdin"]).map(|s| s.trim().to_string())
}

fn diff_pairs(root: &Path, args: &[&str]) -> Option<Vec<PathPair>> {
    let raw = git_stdout(root, args)?;
    Some(parse_name_status(&raw))
}

/// `--name-status -z` tokens → (before, after) path pairs in the
/// supported languages. A copy record carries TWO paths (source
/// survives, destination is new material — exactly the duplication
/// signal this product exists for), so it maps to an added file;
/// consuming only one token would desynchronize the whole stream
/// (attack review 2026-08-11 F13). Unknown one-path statuses are
/// skipped, not misread.
fn parse_name_status(raw: &str) -> Vec<PathPair> {
    let mut toks = raw.split('\0').take_while(|t| !t.is_empty());
    let mut pairs = Vec::new();
    while let Some(status) = toks.next() {
        let mut path = || toks.next().map(str::to_string);
        let pair = match status.chars().next() {
            Some('A') => (None, path()),
            Some('D') => (path(), None),
            Some('M') | Some('T') => {
                let p = path();
                (p.clone(), p)
            }
            Some('R') => (path(), path()),
            Some('C') => {
                let _source_still_exists = path();
                (None, path())
            }
            _ => {
                let _ = path();
                continue;
            }
        };
        let side = pair.1.as_deref().or(pair.0.as_deref());
        if side.is_some_and(|p| Lang::from_path(Path::new(p)).is_some()) {
            pairs.push(pair);
        }
    }
    pairs
}

/// The wire/report shape of one batch: aggregate four-class totals,
/// named relocations, and the degradation flag — numbers and
/// symbols only, no source text.
pub fn report_json(batch: &BatchClassification, pairs: &[PathPair]) -> serde_json::Value {
    let mut totals = [0u64; 4];
    for c in &batch.pairs {
        totals[0] += c.counts.added_novel as u64;
        totals[1] += c.counts.added_moved as u64;
        totals[2] += c.counts.removed_deleted as u64;
        totals[3] += c.counts.removed_moved as u64;
    }
    // The index is the CORE's, so it is checked here rather than
    // trusted: `&pairs[idx]` on a bad reply panics, and this runs
    // inside the daemon, which has no catch_unwind and would take
    // every connected client down with it.
    let name = |idx: usize, before: bool| -> serde_json::Value {
        let Some((b, a)) = pairs.get(idx) else {
            return serde_json::Value::Null;
        };
        let side = if before { b } else { a };
        side.as_deref().or(a.as_deref()).or(b.as_deref()).into()
    };
    let relocations: Vec<serde_json::Value> = batch
        .relocations
        .iter()
        .map(|r| {
            serde_json::json!({
                "from": name(r.from_pair, true), "from_unit": r.from_unit,
                "to": name(r.to_pair, false), "to_unit": r.to_unit,
                "lines": r.lines,
            })
        })
        .collect();
    let suspicions: Vec<serde_json::Value> = batch
        .suspicions
        .iter()
        .map(|(i, kind)| serde_json::json!({"file": name(*i, false), "kind": kind}))
        .collect();
    serde_json::json!({
        "added_novel": totals[0], "added_moved": totals[1],
        "removed_deleted": totals[2], "removed_moved": totals[3],
        "relocations": relocations,
        "suspicions": suspicions,
        "degraded": batch.degraded,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_status_pairs_and_language_filter() {
        let raw = "A\0new.rs\0D\0gone.py\0M\0kept.md\0R100\0old.rs\0moved.rs\0\
                   C100\0src.rs\0copy.rs\0M\0skip.json\0M\0after_copy.rs\0";
        let pairs = parse_name_status(raw);
        assert_eq!(
            pairs,
            vec![
                (None, Some("new.rs".into())),
                (Some("gone.py".into()), None),
                (Some("kept.md".into()), Some("kept.md".into())),
                (Some("old.rs".into()), Some("moved.rs".into())),
                (None, Some("copy.rs".into())),
                // the record AFTER the copy proves the stream stayed
                // in sync (F13's failure mode was desync from here on)
                (Some("after_copy.rs".into()), Some("after_copy.rs".into())),
            ]
        );
    }
}
