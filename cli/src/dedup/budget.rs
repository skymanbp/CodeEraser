//! The R12 only-shrink budget gate (`ce dedup --check`), split from
//! mod.rs when the review-repair asserts pushed it past the 300-line
//! dogfood ceiling. The comparison is the CORE's since ADR-008 P2: a
//! minimal verdict.request carries [blocks, budget] and the fail bit
//! answers — Rust renders the report lines from its own numbers but
//! never re-derives the exit code (the under-budget advisory is
//! reporting, not judgment).

use crate::config::Config;
use anyhow::Result;
use std::path::Path;
use std::process::ExitCode;

pub fn check(
    root: &Path,
    blocks: usize,
    distincts: &[u64],
    floor_override: Option<usize>,
    core: &str,
) -> Result<ExitCode> {
    let cfg = Config::load(root).map_err(anyhow::Error::msg)?;
    let Some(budget) = cfg.dedup.budget else {
        anyhow::bail!("--check needs [dedup] budget in ce.toml");
    };
    let req = crate::score::wire::Request::dedup_only(
        blocks as u64,
        budget as u64,
        distincts.to_vec(),
        floor_override.map(|f| f as u64),
    );
    let reply = crate::score::wire::judge(core, &req)?;
    anyhow::ensure!(
        reply.degraded.is_none(),
        "dedup check: core degraded the judgment ({:?}) — refusing to gate on it",
        reply.degraded
    );
    // batch-7 slice 1: the core re-derived the admitted count with
    // ITS floor and judged the budget from that — the printed report
    // and the gated number must be the same number, proven here on
    // every run (the scan-mirror ensure, dedup form)
    anyhow::ensure!(
        reply.dedup_blocks == Some(blocks as u64),
        "core admitted {:?} blocks, the local filter kept {blocks} —          pairs.rs and CE.Dedup.Cost have drifted",
        reply.dedup_blocks
    );
    if reply.fail {
        // attribute by the HELD names, never by construction-time
        // coincidence (review C8): this message claims the budget,
        // so the fail must BE the budget condition
        anyhow::ensure!(
            reply.failed == ["dedup_budget"],
            "dedup check failed on {:?}, not the budget — message would misattribute",
            reply.failed
        );
        eprintln!(
            "{}",
            crate::i18n::line(
                "dedup ratchet: {} clone blocks > budget {} — new duplication must not land",
                "去冗棘轮：{} 个克隆块 > 预算 {} — 新增重复不得落地",
                &[&blocks, &budget],
            )
        );
        return Ok(ExitCode::FAILURE);
    }
    if blocks < budget {
        println!(
            "{}",
            crate::i18n::line(
                "dedup ratchet: {} clone blocks < budget {} — ratchet the budget down",
                "去冗棘轮：{} 个克隆块 < 预算 {} — 请把预算下调咬合",
                &[&blocks, &budget],
            )
        );
    }
    Ok(ExitCode::SUCCESS)
}
