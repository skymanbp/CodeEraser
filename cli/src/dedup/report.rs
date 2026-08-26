//! The dedup report faces — console, JSON, SARIF — beside nothing
//! they measure: the blocks arrive judged from mod.rs (the
//! renderer-beside-measurement family shape churn, trend, join,
//! doctor and deadcode all split into).

use super::{SCHEMA_ID, Summary, groups, pairs};
use crate::scan::Format;
use anyhow::Result;
use serde::Serialize;

/// The dedup report as a self-contained JSON value (daemon wire use).
pub fn report_json(found: &pairs::Blocks, s: &Summary) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(Report {
        schema: SCHEMA_ID,
        blocks: &found.blocks,
        groups: &found.groups,
        summary: s,
    })?)
}

#[derive(Serialize)]
struct Report<'a> {
    schema: &'static str,
    blocks: &'a [pairs::Block],
    groups: &'a [groups::Group],
    summary: &'a Summary,
}

pub(super) fn emit(format: Format, found: &pairs::Blocks, s: &Summary) -> Result<()> {
    match format {
        Format::Console => print_console(found, s),
        Format::Json => {
            let rep = Report {
                schema: SCHEMA_ID,
                blocks: &found.blocks,
                groups: &found.groups,
                summary: s,
            };
            println!("{}", serde_json::to_string_pretty(&rep)?);
        }
        Format::Sarif => {
            let results = found.blocks.iter().map(sarif_block).collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&crate::sarif::report(results))?
            );
        }
    }
    Ok(())
}

/// A clone block as a SARIF "note": blocks are budget-gated facts,
/// not per-block failures — the ratchet's tolerated debt must not
/// masquerade as errors on a scanning dashboard. The message is the
/// console face's English line verbatim (SARIF is a machine face,
/// never translated).
fn sarif_block(b: &pairs::Block) -> serde_json::Value {
    crate::sarif::result(
        "ce.dedup/clone-block",
        "note",
        &format!(
            "dup {}:{}-{} <-> {}:{}-{} ({} tokens)",
            b.a_file, b.a_start, b.a_end, b.b_file, b.b_start, b.b_end, b.tokens
        ),
        crate::sarif::location(&b.a_file, b.a_start, b.a_end),
        vec![crate::sarif::location(&b.b_file, b.b_start, b.b_end)],
    )
}

/// The console face (split from emit at the 50-line fn gate when the
/// bilingual lines landed, M8-G3b).
fn print_console(found: &pairs::Blocks, s: &Summary) {
    for b in &found.blocks {
        println!(
            "{}",
            crate::i18n::line(
                "dup {}:{}-{} <-> {}:{}-{} ({} tokens)",
                "重复 {}:{}-{} <-> {}:{}-{}（{} tokens）",
                &[
                    &b.a_file, &b.a_start, &b.a_end, &b.b_file, &b.b_start, &b.b_end, &b.tokens,
                ],
            )
        );
    }
    // named binding (not inline): the inline array made this call a
    // T2 twin of the dup-line call above — the repo's own ratchet bit
    // the pair the day it landed
    let counts: [&dyn std::fmt::Display; 10] = [
        &s.files,
        &s.refreshed,
        &s.removed,
        &s.blocks,
        &s.groups,
        &s.min_tokens,
        &s.min_distinct,
        &s.low_diversity_suppressed,
        &s.hot_chained,
        &s.stale_skipped,
    ];
    println!(
        "{}",
        crate::i18n::line(
            "indexed {} files ({} refreshed, {} removed) — {} clone blocks in {} groups (min {} tokens, distinct >= {}), {} low-diversity suppressed, {} hot chained, {} stale skipped",
            "已索引 {} 个文件（刷新 {}，移除 {}）— {} 个克隆块 / {} 组（最少 {} tokens，多样性 >= {}），抑制低多样性 {}，热链 {}，跳过陈旧 {}",
            &counts,
        )
    );
}
