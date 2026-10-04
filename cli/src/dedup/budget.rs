//! The R12 only-shrink budget gate (`ce dedup --check`), split from
//! mod.rs when the review-repair asserts pushed it past the 300-line
//! dogfood ceiling. The comparison is the CORE's since ADR-008 P2: a
//! minimal verdict.request carries [blocks, budget] and the fail bit
//! answers; since plan v2.32 step 5 the ratchet's lines and the exit
//! are the dedup document's too (CE.Dedup.Lines reads the budget and
//! the bit), so this side never re-derives either.

use crate::config::Config;
use anyhow::Result;
use std::path::Path;

/// One `--check` judgment as the document reads it: whether a check
/// ran, the declared budget and verdict/1's fail bit (the budget
/// held). The default is "no check".
#[derive(Clone, Copy, Default)]
pub struct Gate {
    pub checked: bool,
    pub budget: usize,
    pub fail: bool,
}

/// The `--check` road judges at the calibrated operating point
/// (t = 50, diversity floor 7). An override may TIGHTEN a filter —
/// more blocks admitted, a stricter gate — but never loosen it:
/// `--min-distinct 40` drove this repository's own budget from 182
/// to 0 with no clone repaid (k4 fence attack). Refused BEFORE any
/// measurement, by name: a `--min-tokens` above the guarantee and a
/// `--min-distinct 0` before any core contact, a `--min-distinct` above
/// the floor once the floor is read off the core's package (plan v2.33
/// W3: this side holds no copy of it). Report-only runs, the MCP tool
/// and the GUI keep both overrides free: calibration controls.
pub(super) fn gate_filters(min_tokens: Option<usize>, min_distinct: Option<usize>) -> Result<()> {
    let t = super::Params::default().guarantee();
    if let Some(m) = min_tokens.filter(|&m| m > t) {
        anyhow::bail!(crate::i18n::line(
            "--check judges at the calibrated operating point: --min-tokens {} is above {} and would admit fewer blocks (default or tighter only)",
            "--check 按校准工作点判决：--min-tokens {} 高于 {}，会少收克隆块（只准默认或更紧）",
            &[&m, &t],
        ));
    }
    match min_distinct {
        Some(0) => anyhow::bail!(crate::i18n::line(
            "--check cannot judge without a diversity floor: --min-distinct 0 disables the floor the core's contract needs (default or tighter only)",
            "--check 不能在无多样性地板下判决：--min-distinct 0 关闭了核契约所需的地板（只准默认或更紧）",
            &[],
        )),
        Some(m) => {
            let d = super::pairs::default_min_distinct();
            anyhow::ensure!(
                m <= d,
                crate::i18n::line(
                    "--check judges at the calibrated operating point: --min-distinct {} is above {} and would suppress more blocks (default or tighter only)",
                    "--check 按校准工作点判决：--min-distinct {} 高于 {}，会多抑制克隆块（只准默认或更紧）",
                    &[&m, &d],
                )
            );
            Ok(())
        }
        None => Ok(()),
    }
}

pub fn check(
    root: &Path,
    blocks: usize,
    distincts: &[u64],
    floor_override: Option<usize>,
    core: &str,
) -> Result<Gate> {
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
    let (reply, _) = crate::score::wire::judge(core, &req)?;
    anyhow::ensure!(
        reply.degraded.is_none(),
        "dedup check: core degraded the judgment ({:?}) — refusing to gate on it",
        reply.degraded
    );
    // batch-7 slice 1: the core re-derived the admitted count with
    // ITS floor and judged the budget from that — the printed report
    // and the gated number must be the same number, proven here on
    // every run (the scan-mirror ensure, dedup form)
    // null = no distinct rows rode (the core's null-absence stance,
    // 2.19.0), which happens exactly when the walk found no block at
    // all — so null reads as zero admitted, and a clean tree passes
    // its own budget instead of failing as a drift (plan v2.18 step
    // #12 found it: the first `--check` on a tree without clones)
    anyhow::ensure!(
        reply.dedup_blocks.unwrap_or(0) == blocks as u64,
        "core admitted {:?} blocks, the local filter kept {blocks} — pairs.rs and CE.Dedup.Cost have drifted",
        reply.dedup_blocks
    );
    // attribute by the HELD names, never by construction-time
    // coincidence (review C8): the ratchet's line claims the budget,
    // so a fail must BE the budget condition
    anyhow::ensure!(
        !reply.fail || reply.failed == ["dedup_budget"],
        "dedup check failed on {:?}, not the budget — message would misattribute",
        reply.failed
    );
    Ok(Gate {
        checked: true,
        budget,
        fail: reply.fail,
    })
}
