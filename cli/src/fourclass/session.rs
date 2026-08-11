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
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--name-status", "-z", "-M", "-C", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&out.stdout).into_owned();
    Some(parse_name_status(&raw))
}

/// `--name-status -z` tokens → (before, after) path pairs in the
/// supported languages. Statuses beyond A/D/M/T/R (copies) have no
/// "before side consumed" reading and are skipped, not misread.
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
    let name = |idx: usize, before: bool| -> serde_json::Value {
        let (b, a) = &pairs[idx];
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
    serde_json::json!({
        "added_novel": totals[0], "added_moved": totals[1],
        "removed_deleted": totals[2], "removed_moved": totals[3],
        "relocations": relocations,
        "degraded": batch.degraded,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_status_pairs_and_language_filter() {
        let raw = "A\0new.rs\0D\0gone.py\0M\0kept.md\0R100\0old.rs\0moved.rs\0M\0skip.json\0";
        let pairs = parse_name_status(raw);
        assert_eq!(
            pairs,
            vec![
                (None, Some("new.rs".into())),
                (Some("gone.py".into()), None),
                (Some("kept.md".into()), Some("kept.md".into())),
                (Some("old.rs".into()), Some("moved.rs".into())),
            ]
        );
    }
}
