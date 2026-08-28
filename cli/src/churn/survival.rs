//! The window's survival leg: of the lines HEAD's blame still dates
//! inside the window, how many are left (churn = 1 - survival). Split
//! out of churn/mod.rs so the fix's clock-basis pin fits under the
//! 300-line dogfood ceiling without a ratchet-breaking growth step.
//!
//! The clock basis and why it changed are recorded in the churn
//! module header, beside the nth caveat — one statement, not two.

use super::gitio::git;
use anyhow::{Context, Result};
use std::path::Path;

/// Lines at HEAD blame-dated inside the window, over the touched set.
pub(super) fn survival(root: &Path, days: u32, touched: &[String]) -> Result<usize> {
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("clock")?
        .as_secs()
        .saturating_sub(u64::from(days) * 86_400);
    let mut surviving = 0usize;
    for (n, file) in touched.iter().enumerate() {
        // the blame storm is the expensive half of the window (155 s
        // of the first M5-3h measurement); it ticks per file so the
        // second half of a churn run is not a second silence
        crate::progress::at(crate::progress::Phase::Survival, n, touched.len());
        let Ok(out) = git(root, &["blame", "--line-porcelain", "HEAD", "--", file]) else {
            // deleted since: zero survivors by definition (a path that
            // became a gitlink would land here too, but never reaches
            // `touched` — a gitlink pair has no judged extension)
            continue;
        };
        surviving += surviving_lines(&out, cutoff);
    }
    Ok(surviving)
}

/// Porcelain records inside the window, dated by COMMITTER time — the
/// one clock `git log --since` already picked the window's commits
/// by. `--line-porcelain` repeats the header block per line, so this
/// is a per-line count; the blamed content itself arrives
/// TAB-prefixed, which is why a source line that reads
/// `committer-time 0` cannot forge a record here.
fn surviving_lines(porcelain: &str, cutoff: u64) -> usize {
    porcelain
        .lines()
        .filter_map(|l| l.strip_prefix("committer-time "))
        .filter_map(|t| t.trim().parse::<u64>().ok())
        .filter(|&t| t >= cutoff)
        .count()
}

#[cfg(test)]
#[path = "survival_tests.rs"]
mod tests;
