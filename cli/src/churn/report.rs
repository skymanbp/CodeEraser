//! The churn report shapes and emitters, split from mod.rs at the
//! repo's own 300-line dogfood gate when the M5-3h per-unit ledger
//! landed. Totals are METHODS over the ledger, never stored fields —
//! the conservation-by-construction half of the ledger design.

use super::COCHANGE_FILE_CAP;
use crate::i18n::line;

/// One ledger row: lines the window added inside this unit. `key` ""
/// (with nth 0) is the file's top level — `owner()` found no
/// containing unit, which is a real place, not an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitRow {
    pub path: String,
    pub key: String,
    pub nth: i64,
    pub appended: usize,
    pub rewrote: usize,
}

pub struct Report {
    pub commits: usize,
    /// Per-unit ledger, sorted by (path, key, nth).
    pub units: Vec<UnitRow>,
    pub surviving: usize,
    pub cochange: Vec<(String, String, usize)>,
    pub skipped_large: usize,
}

impl Report {
    /// Every added line lands in exactly one ledger row, so the
    /// window totals are sums over it, never separate counters.
    pub fn append_lines(&self) -> usize {
        self.units.iter().map(|u| u.appended).sum()
    }
    pub fn rewrite_lines(&self) -> usize {
        self.units.iter().map(|u| u.rewrote).sum()
    }
    pub fn added_in_window(&self) -> usize {
        self.append_lines() + self.rewrite_lines()
    }
}

pub fn report_json(r: &Report) -> serde_json::Value {
    let added = r.added_in_window();
    let churned = added.saturating_sub(r.surviving);
    serde_json::json!({
        "schema": "ce.churn-report/0.1.0",
        "commits": r.commits,
        "append_lines": r.append_lines(),
        "rewrite_lines": r.rewrite_lines(),
        "added_in_window": added,
        "surviving": r.surviving,
        "churned": churned,
        "cochange": r.cochange.iter()
            .map(|(a, b, n)| serde_json::json!({"a": a, "b": b, "count": n}))
            .collect::<Vec<_>>(),
        "skipped_large_commits": r.skipped_large,
    })
}

pub fn print_console(r: &Report, days: u32) {
    let added = r.added_in_window();
    let churned = added.saturating_sub(r.surviving);
    println!(
        "{}",
        line(
            "churn window {}d: {} commits, appended {} / rewrote {} lines",
            "改动窗口 {} 天：{} 个提交，追加 {} / 重写 {} 行",
            &[&days, &r.commits, &r.append_lines(), &r.rewrite_lines()],
        )
    );
    println!(
        "{}",
        line(
            "window survival: {} of {} added lines survive at HEAD ({} churned)",
            "窗口存活：新增 {} / {} 行存活至 HEAD（{} 已翻改）",
            &[&r.surviving, &added, &churned],
        )
    );
    for (a, b, n) in &r.cochange {
        println!(
            "{}",
            line(
                "co-change x{}: {} <-> {}",
                "共变 x{}：{} <-> {}",
                &[n, a, b]
            )
        );
    }
    if r.skipped_large > 0 {
        println!(
            "{}",
            line(
                "note: {} commit(s) above {} files skipped for pairing",
                "注：{} 个提交超过 {} 文件上限，未参与配对",
                &[&r.skipped_large, &COCHANGE_FILE_CAP],
            )
        );
    }
}
