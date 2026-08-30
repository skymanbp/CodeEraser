//! The churn report shapes and emitters, split from mod.rs at the
//! repo's own 300-line dogfood gate when the M5-3h per-unit ledger
//! landed. Totals are METHODS over the ledger, never stored fields —
//! the conservation-by-construction half of the ledger design.

use crate::i18n::line;

/// Commits with more changed files than this are skipped for pair
/// counting (quadratic) and reported, never silently dropped. The
/// report prints the cap beside the skip count it explains, so this
/// leaf owns the binding and the measurement (mod.rs) reads the
/// SAME one — the parent-hub import was a module cycle the graph
/// axis itself billed (headroom sprint, 2026-08-24).
pub(crate) const COCHANGE_FILE_CAP: usize = 20;

/// The report's schema id; 0.2.0: additive
/// `submodules_without_file_history`. A named constant, not an inline
/// literal: the derived-fact registry (plan v2.21) scans cli/src for
/// value-shaped ids and every family names its own.
const SCHEMA: &str = "ce.churn-report/0.2.0";

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
    /// Declared submodules holding judged files whose history is not
    /// this repository's (mod.rs `unhistoried`) — the ledger's named
    /// shortfall, never an unnamed exclusion.
    pub submodules_without_history: Vec<String>,
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
        "schema": SCHEMA,
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
        "submodules_without_file_history": r.submodules_without_history,
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
    // display cut only — the report struct and the wire carry the
    // full table (batch-7 slice 12); the remainder is counted out
    // loud, never silently absent
    for (a, b, n) in r.cochange.iter().take(20) {
        println!(
            "{}",
            line(
                "co-change x{}: {} <-> {}",
                "共变 x{}：{} <-> {}",
                &[n, a, b]
            )
        );
    }
    if r.cochange.len() > 20 {
        println!(
            "{}",
            line(
                "co-change: {} more pairs below the display cut",
                "共变：另有 {} 对低于显示截断",
                &[&(r.cochange.len() - 20)]
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
    if !r.submodules_without_history.is_empty() {
        println!(
            "{}",
            line(
                "note: no file history for declared submodule(s) {} — the ledger is the superproject's own history",
                "注：声明的 submodule {} 无文件历史——账本只计超仓自身的历史",
                &[&r.submodules_without_history.join(", ")],
            )
        );
    }
}
