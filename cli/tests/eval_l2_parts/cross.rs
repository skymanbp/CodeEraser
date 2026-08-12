//! The per-file cross-GT view and its gates' row/ledger builders —
//! split from mod.rs when the M5-1d below-floor register pushed it
//! past the file budget.

use super::{FileKey, Texts, side_text};
use crate::eval_commit_review as review;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Per-file cross ground truth: counts for every file, line
/// identities for files without a reviewed correction (attack review
/// F2 — the corrected lines' identities were never archived, so
/// those files stay count-gated plus the coincidence-exact gate).
pub struct CrossGt {
    pub counts: BTreeMap<FileKey, u64>,
    pub lines: BTreeMap<FileKey, Vec<usize>>,
    /// Reviewed below-floor register (M5-1d): true relocated lines
    /// with no >=2-distinct contiguous companion — structurally no
    /// site can open. Counts/lines above stay TRUTH; every gate
    /// requires zero misses BEYOND this itemized waiver (the
    /// miss-side mirror of the extras ledger).
    pub waived: BTreeMap<FileKey, Vec<usize>>,
}

/// Reviewed per-file cross GT via the labels machinery (mechanical
/// partition + the reviewed corrections), same code that generated
/// commit-labels-v1.json.
pub fn cross_gt(sha: &str) -> CrossGt {
    let mut p = review::partition(sha);
    let _ = review::apply_corrections(sha, &mut p);
    let mut gt = CrossGt {
        counts: BTreeMap::new(),
        lines: BTreeMap::new(),
        waived: BTreeMap::new(),
    };
    for (side, buckets) in [("out", &p.out), ("in", &p.into)] {
        for (f, n) in &buckets.cross {
            gt.counts.insert((side.to_string(), f.clone()), *n);
        }
        for (f, lines) in &buckets.cross_lines {
            let mut lines = lines.clone();
            lines.sort_unstable();
            gt.lines.insert((side.to_string(), f.clone()), lines);
        }
    }
    for (side, file, line) in review::below_floor_for(sha) {
        let key = (side, file);
        // a register row must describe a real GT line — rot fails loud
        let known = gt.lines.get(&key).map(|l| l.contains(&line));
        assert!(known.unwrap_or(false), "{sha}: {key:?}:{line} not in GT");
        gt.waived.entry(key).or_default().push(line);
    }
    gt.waived.values_mut().for_each(|v| v.sort_unstable());
    gt
}

/// Per-commit cross rows {file, side, gt, pred, gt_lines}; misses
/// must be zero — at LINE IDENTITY where the GT has identities
/// (attack review F2: a same-count substitution passed a count-only
/// gate), at count level on the reviewed-correction files, zero
/// beyond the reviewed below-floor register. A miss dumps the
/// predicted lines with content so the divergence is diagnosable
/// from the failure alone.
pub fn cross_rows(
    sha: &str,
    texts: &Texts,
    gt: &CrossGt,
    delta: &BTreeMap<FileKey, Vec<usize>>,
) -> Vec<Value> {
    let mut keys: Vec<&FileKey> = gt.counts.keys().chain(delta.keys()).collect();
    keys.sort();
    keys.dedup();
    let none: Vec<usize> = Vec::new();
    keys.iter()
        .map(|key| {
            let g = gt.counts.get(*key).copied().unwrap_or(0);
            let lines = delta.get(*key).cloned().unwrap_or_default();
            let p = lines.len() as u64;
            let waived = gt.waived.get(*key).unwrap_or(&none);
            let w = waived.len() as u64;
            assert!(
                p + w >= g,
                "{sha}: cross miss on {key:?}: gt {g} pred {p}\npred lines:\n{}",
                dump(texts, key, &lines)
            );
            let gt_lines = gt.lines.get(*key);
            if let Some(want) = gt_lines {
                let missed: Vec<usize> = want
                    .iter()
                    .copied()
                    .filter(|l| !lines.contains(l) && !waived.contains(l))
                    .collect();
                assert!(
                    missed.is_empty(),
                    "{sha}: line-identity miss on {key:?}: gt {want:?} missing \
                     {missed:?}\npred lines:\n{}",
                    dump(texts, key, &lines)
                );
            }
            let mut row = json!({"side": key.0, "file": key.1, "gt": g, "pred": p,
                   "gt_lines": gt_lines});
            if !waived.is_empty() {
                row["below_floor"] = json!(waived);
            }
            row
        })
        .collect()
}

fn dump(texts: &Texts, (side, file): &FileKey, lines: &[usize]) -> String {
    let text: Vec<&str> = side_text(texts, side, file).lines().collect();
    lines
        .iter()
        .map(|&l| format!("  {l}: {}\n", text[l - 1].trim()))
        .collect()
}

/// Extras (pred above GT) itemized with content for review.
pub fn extras_ledger(
    sha: &str,
    texts: &Texts,
    gt: &CrossGt,
    delta: &BTreeMap<FileKey, Vec<usize>>,
) -> Vec<Value> {
    delta
        .iter()
        .filter(|(key, lines)| (lines.len() as u64) > gt.counts.get(*key).copied().unwrap_or(0))
        .map(|((side, file), lines)| {
            let text: Vec<&str> = side_text(texts, side, file).lines().collect();
            let dump: Vec<String> = lines
                .iter()
                .map(|&l| format!("{l}: {}", text[l - 1].trim()))
                .collect();
            json!({"sha": &sha[..9], "side": side, "file": file,
                   "gt": gt.counts.get(&(side.clone(), file.clone())).copied().unwrap_or(0),
                   "pred": lines.len(), "lines": dump})
        })
        .collect()
}

/// A cross row's (gt, pred) counts.
pub fn gt_pred(c: &Value) -> (u64, u64) {
    (c["gt"].as_u64().unwrap(), c["pred"].as_u64().unwrap())
}
