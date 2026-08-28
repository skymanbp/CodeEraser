//! `ce graph --mentions`: the pass's own console/JSON face — report-
//! only, the header counters, the convergence facts and the K23
//! per-language census of the veto, so the universe is observable
//! before any judgment consumes it (K39–K42 are library legs; this is
//! the operator's window on the same numbers).

use super::LangRates;
use crate::i18n::line;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The document's version identity — every report face carries one.
/// 0.2.0: the `rates` census rides beside the header (additive).
pub const SCHEMA_ID: &str = "ce.mentions-report/0.2.0";

pub fn run(root: &Path, db: Option<PathBuf>, json: bool) -> ExitCode {
    match refreshed(root, db) {
        Ok((stats, rates)) if json => {
            println!("{}", report_json(&stats, &rates));
            ExitCode::SUCCESS
        }
        Ok((stats, rates)) => {
            for l in console(&stats).into_iter().chain(rates_console(&rates)) {
                println!("{l}");
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("ce graph: {err:#}");
            ExitCode::from(2)
        }
    }
}

/// The judged index first (the `outside` counters compare against
/// its `files` table and the census reads its declarations), then
/// the mention pass over the same tree, then the veto counted per
/// language.
fn refreshed(
    root: &Path,
    db: Option<PathBuf>,
) -> anyhow::Result<(super::Stats, BTreeMap<&'static str, LangRates>)> {
    let (idx, _db) = crate::dedup::refreshed_index(root, db)?;
    let stats = super::refresh(root, &idx)?;
    let rates = super::rates::census(root, &idx)?;
    Ok((stats, rates))
}

pub fn report_json(stats: &super::Stats, rates: &BTreeMap<&'static str, LangRates>) -> String {
    let mut doc = serde_json::to_value(stats).expect("stats serialize");
    doc["schema"] = serde_json::Value::String(SCHEMA_ID.to_string());
    doc["mention_rev"] = serde_json::Value::from(super::MENTION_REV);
    doc["rates"] = serde_json::to_value(rates).expect("rates serialize");
    doc.to_string()
}

fn console(s: &super::Stats) -> Vec<String> {
    let rescan = if s.run.rescanned {
        " (rev changed: full rescan)"
    } else {
        ""
    };
    vec![
        line(
            "mention universe: {} files, {} mention sources, {} rows, {} files at the per-file cap (rev {})",
            "提及语料宇宙：{} 个文件，{} 个提及源文件，{} 行，{} 个文件触及单文件上限（rev {}）",
            &[
                &s.universe,
                &s.sources,
                &s.rows,
                &s.capped,
                &super::MENTION_REV,
            ],
        ),
        line(
            "  skipped: {} over 4 MiB, {} binary, {} walk errors",
            "  跳过：{} 超 4 MiB，{} 二进制，{} walk 错误",
            &[
                &s.skipped.oversize,
                &s.skipped.binary,
                &s.skipped.walk_errors,
            ],
        ),
        line(
            "  this run: {} refreshed, {} removed, {} rows clipped, {} files starved by the table cap{}",
            "  本次：刷新 {}，移除 {}，裁剪 {} 行，{} 个文件被表上限饿住{}",
            &[
                &s.run.refreshed,
                &s.run.removed,
                &s.run.clipped,
                &s.run.starved,
                &rescan,
            ],
        ),
        line(
            "  judged files outside the universe: {} over cap, {} binary, {} in nested repositories, {} ignore skew",
            "  判决文件不在宇宙内：{} 超限，{} 二进制，{} 在嵌套仓，{} 忽略语义差",
            &[
                &s.outside.oversize,
                &s.outside.binary,
                &s.outside.nested,
                &s.outside.ignored,
            ],
        ),
        line(
            "  dist/*.js bundler-suffixed runs (name$N): {}",
            "  dist/*.js 打包器去重后缀 run（name$N）：{}",
            &[&s.dist_js_dedup_runs],
        ),
    ]
}

/// One line per language (K23): the domain and its exported half,
/// what survived the veto (its exported half), and where the veto
/// stopped — with the collision-saved count beside `other`, the
/// blindness stated as a number rather than a footnote.
fn rates_console(rates: &BTreeMap<&'static str, LangRates>) -> Vec<String> {
    rates
        .iter()
        .map(|(lang, r)| {
            line(
                "  {}: {} declared ({} exported) — {} unmentioned ({} exported); vetoed by another file {} (of which {} only by a same-name declaration), by fold {}, by the file's own exceptions {}",
                "  {}：声明 {}（导出 {}）——未提及 {}（导出 {}）；他文件否决 {}（其中 {} 仅因同名声明得救），折叠否决 {}，自文件例外否决 {}",
                &[
                    lang,
                    &r.declared.all,
                    &r.declared.exported,
                    &r.unmentioned.all,
                    &r.unmentioned.exported,
                    &r.vetoed.other,
                    &r.vetoed.collision_saved,
                    &r.vetoed.fold,
                    &r.vetoed.self_text,
                ],
            )
        })
        .collect()
}
