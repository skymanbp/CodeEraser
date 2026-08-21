//! One JSON document per report family — the adapter layer BOTH
//! machine surfaces consume: the MCP catalog stringifies these, the
//! GUI backend returns them as-is. Lifted when batch 4 was about to
//! copy the MCP adapter bodies into the Tauri commands (the P4
//! ratchet precedent: cross-surface shells chain into clone blocks).
//! Read-only by construction, same as the MCP charter: every face
//! ends at a family's public report serialization — none writes a
//! baseline or config. Callers resolve the root and the core; a face
//! only turns (root, knobs) into the family's one document.

use anyhow::Result;
use serde_json::Value;
use std::path::Path;

pub fn scan(root: &Path) -> Result<Value> {
    let (files, findings, summary) = crate::scan::analyze(root)?;
    Ok(serde_json::from_str(&crate::scan::report_string(
        &files, &findings, summary,
    )?)?)
}

pub fn dedup(root: &Path, min_tokens: Option<usize>) -> Result<Value> {
    let (found, summary) = crate::dedup::analyze(root, None, min_tokens, None)?;
    crate::dedup::report_json(&found, &summary)
}

pub fn churn(root: &Path, days: u32) -> Result<Value> {
    Ok(crate::churn::report_json(&crate::churn::run(root, days)?))
}

pub fn graph_sites(root: &Path) -> Result<Value> {
    Ok(serde_json::from_str(&crate::graph::sites_json(
        &crate::graph::analyze(root)?,
    ))?)
}

pub fn deadcode(root: &Path, core: &str) -> Result<Value> {
    Ok(crate::report::deadcode_json(&crate::graph::deadcode::run(
        root, None, core,
    )?))
}

/// The envelope-shaped pair share one throat (their two bodies were
/// a token-identical twin by this repo's own measure).
fn enveloped<M: serde::Serialize, C: serde::Serialize>(
    pair: (&str, &str),
    r: anyhow::Result<crate::report::Report<M, C>>,
) -> Result<Value> {
    Ok(crate::report::envelope(pair, &r?))
}

pub fn clone_t3(root: &Path, core: &str) -> Result<Value> {
    enveloped(
        (crate::dedup::t3::SCHEMA_ID, "clones"),
        crate::dedup::t3::run(root, None, core),
    )
}

pub fn docdup(root: &Path, core: &str) -> Result<Value> {
    enveloped(
        (crate::docdup::judge::SCHEMA_ID, "dups"),
        crate::docdup::judge::run(root, None, core),
    )
}

pub fn join(root: &Path, core: &str, days: u32) -> Result<Value> {
    Ok(crate::join::report_json(&crate::join::run(
        root, None, core, days,
    )?))
}

pub fn structure(root: &Path, core: &str, knobs: (bool, Option<u32>, bool)) -> Result<Value> {
    Ok(crate::structure::judge::report_json(
        &crate::structure::judge::run(root, None, core, knobs)?,
    ))
}

/// Report-only: this face never writes a baseline (MCP charter ③;
/// the GUI apply road goes through erase, never through establish).
pub fn check(root: &Path, core: &str) -> Result<Value> {
    let o = crate::score::run(
        root,
        crate::score::Opts {
            db: None,
            core: core.into(),
            days: None,
            floor: None,
            establish: false,
            pinned_soft: None,
        },
    )?;
    Ok(crate::score::report_json(&o))
}

pub fn trend(root: &Path, core: &str, commits: usize, batch: Option<usize>) -> Result<Value> {
    Ok(crate::trend::report_json(&crate::trend::run(
        root, None, core, commits, batch,
    )?))
}

/// The erase PLAN document (dry-run by definition); the apply road is
/// erase::apply_plan and stays outside this read-only layer.
pub fn erase_plan(root: &Path, core: &str) -> Result<Value> {
    Ok(crate::erase::render::report_json(&crate::erase::plan(
        root, None, core,
    )?))
}
