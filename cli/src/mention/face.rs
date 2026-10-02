//! `ce graph --mentions`: the pass's own console/JSON face — report-
//! only, the header counters, the convergence facts and the K23
//! per-language census of the veto, so the universe is observable
//! before any judgment consumes it (K39–K42 are library legs; this is
//! the operator's window on the same numbers). The core lays the
//! document out (document/1, CE.Mention.Document) from this side's
//! numbers; the console reads the document back.

use super::LangRates;
use crate::document::{self, Request, Resolve};
use crate::i18n::line;
use crate::scan::lang::Lang;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn run(root: &Path, db: Option<PathBuf>, core: &str, json: bool) -> ExitCode {
    let doc = document(root, db, core);
    crate::report::print_read(doc, json, "mentions", |r: Read| {
        for l in console(&r).into_iter().chain(rates_console(&r.rates)) {
            println!("{l}");
        }
    })
}

/// The judged index first (the `outside` counters compare against
/// its `files` table and the census reads its declarations), then
/// the mention pass over the same tree, then the veto counted per
/// language — and the document the core lays out over them.
pub fn document(root: &Path, db: Option<PathBuf>, core: &str) -> Result<Value> {
    let (idx, _db) = crate::dedup::refreshed_index(root, db)?;
    let stats = super::refresh(root, &idx)?;
    let rates = super::rates::census(root, &idx)?;
    document::assemble(core, request(&stats, &rates)?, &Nothing)
}

/// The header as facts — every counter of `Stats` under its dotted
/// path (`skipped.signed`), a flag as 0 / 1 — beside the index
/// revision, and one census row per language.
pub(super) fn request(
    s: &super::Stats,
    rates: &BTreeMap<&'static str, LangRates>,
) -> Result<Request> {
    let mut facts = vec![("mention_rev".to_string(), super::MENTION_REV)];
    flatten("", &serde_json::to_value(s)?, &mut facts)?;
    let rows = rates
        .iter()
        .map(|(name, r)| {
            let lang = Lang::ALL
                .iter()
                .find(|l| l.name() == *name)
                .with_context(|| format!("census language {name} has no code"))?;
            let v = &r.vetoed;
            Ok([
                *lang as i64,
                r.declared.all as i64,
                r.declared.exported as i64,
                r.unmentioned.all as i64,
                r.unmentioned.exported as i64,
                v.other as i64,
                v.fold as i64,
                v.self_text as i64,
                v.collision_saved as i64,
            ])
        })
        .collect::<Result<Vec<_>>>()?;
    let req = facts
        .into_iter()
        .fold(Request::new("mentions"), |req, (k, n)| req.fact(&k, n));
    Ok(req.rows("rates", rows))
}

/// `v`'s integer leaves under their dotted paths.
fn flatten(at: &str, v: &Value, out: &mut Vec<(String, i64)>) -> Result<()> {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                let path = if at.is_empty() {
                    k.clone()
                } else {
                    format!("{at}.{k}")
                };
                flatten(&path, x, out)?;
            }
        }
        Value::Bool(b) => out.push((at.to_string(), i64::from(*b))),
        _ => out.push((
            at.to_string(),
            v.as_i64().with_context(|| format!("{at}: {v}"))?,
        )),
    }
    Ok(())
}

/// The mentions document names no repository string.
struct Nothing;

impl Resolve for Nothing {
    fn resolve(&self, _: &str, _: &[i128]) -> Option<String> {
        None
    }
}

/// The document as the console reads it.
#[derive(Deserialize)]
struct Read {
    mention_rev: i64,
    #[serde(flatten)]
    stats: super::Stats,
    rates: BTreeMap<String, LangRates>,
}

fn console(r: &Read) -> Vec<String> {
    let s = &r.stats;
    let k = &s.skipped;
    let rescan = if s.run.rescanned {
        " (rev changed: full rescan)"
    } else {
        ""
    };
    vec![
        line(
            "mention universe: {} files, {} mention sources, {} rows, {} files at the per-file cap (rev {})",
            "提及语料宇宙：{} 个文件，{} 个提及源文件，{} 行，{} 个文件触及单文件上限（rev {}）",
            &[&s.universe, &s.sources, &s.rows, &s.capped, &r.mention_rev],
        ),
        line(
            "  skipped: {} over 4 MiB, {} binary, {} signed, {} walk errors",
            "  跳过：{} 超 4 MiB，{} 二进制，{} 产品签名，{} walk 错误",
            &[&k.oversize, &k.binary, &k.signed, &k.walk_errors],
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
fn rates_console(rates: &BTreeMap<String, LangRates>) -> Vec<String> {
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
