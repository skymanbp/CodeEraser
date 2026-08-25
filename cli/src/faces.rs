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

/// Judged like its siblings (batch-7 slice 8): the scan face used
/// to read the mirror with no core link — the one unguarded copy of
/// a rule the core owns; analyze_judged carries the drift ensure to
/// every surface.
pub fn scan(root: &Path, core: &str) -> Result<Value> {
    let (files, findings, summary, _fail) = crate::scan::analyze_judged(root, core)?;
    Ok(serde_json::from_str(&crate::scan::report_string(
        &files, &findings, summary,
    )?)?)
}

/// Both report thresholds ride, not one: `min_distinct` is the
/// DIVERSITY floor (`ce dedup --min-distinct`), and a face that
/// accepted only `min_tokens` could not reproduce what the CLI
/// prints — the caller was silently pinned to the core default.
pub fn dedup(root: &Path, min_tokens: Option<usize>, min_distinct: Option<usize>) -> Result<Value> {
    let (found, summary) = crate::dedup::analyze(root, None, min_tokens, min_distinct)?;
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
    Ok(crate::structure::report::report_json(
        &crate::structure::judge::run(root, None, core, knobs)?,
    ))
}

/// Report-only: this face never writes a baseline (MCP charter ③;
/// the GUI apply road goes through erase, never through establish).
/// `floor` is the CLI's `--fail-under`, opt-in on every road — but a
/// face that could not arm it could not reproduce the verdict CI
/// prints, and the report now echoes which it judged under.
pub fn check(root: &Path, core: &str, floor: Option<u32>) -> Result<Value> {
    let o = crate::score::run(
        root,
        crate::score::Opts {
            db: None,
            core: core.into(),
            days: None,
            floor,
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

/// The graph canvas document (batch 9 P18): ONE deadcode-family
/// judgment answers verdicts AND position — the assembly and the
/// file-tier projection live in graph::canvas. Read-only like every
/// face.
pub fn graph_canvas(root: &Path, core: &str) -> Result<Value> {
    crate::graph::canvas::run(root, core)
}

/// The cached unit universe (`ce clone --units`): the document used
/// to be built inline in the CLI's own command body, so it was the
/// one family document no machine surface could reach. The identity
/// assertion travels with it — a face that listed units without
/// checking the unitsig/symbols agreement would hand out a universe
/// nobody had checked.
pub fn clone_units(root: &Path) -> Result<Value> {
    let (idx, _db) = crate::dedup::refreshed_index(root, None)?;
    let orphans = crate::dedup::unitcache::identity_orphans(&idx)?;
    anyhow::ensure!(
        orphans == 0,
        "{orphans} unitsig rows missing their symbols identity — nth throat drift"
    );
    let rows = crate::dedup::unitcache::unit_rows(&idx)?;
    Ok(serde_json::json!({
        "schema": crate::dedup::unitcache::UNITS_SCHEMA_ID,
        "units": rows.iter().map(|u| serde_json::json!({
            "path": u.path, "key": u.key, "nth": u.nth, "nodes": u.nodes,
        })).collect::<Vec<_>>(),
    }))
}

/// The erase PLAN — dry-run by definition and by construction: this
/// face reaches `erase::plan`, which is read-only, and there is no
/// face for `apply_plan` at all. That absence is the charter, not an
/// omission: a machine surface that could delete files on its own
/// authority is the one thing an eraser must never ship.
pub fn erase(root: &Path, core: &str) -> Result<Value> {
    Ok(crate::erase::render::report_json(&crate::erase::plan(
        root, None, core,
    )?))
}

/// The machine's own state. Unlike every sibling it cannot fail: a
/// core that will not answer IS the finding, and it rides inside the
/// document (health::doctor).
pub fn doctor(root: &Path, core: &str) -> Result<Value> {
    Ok(crate::health::doctor::document(root, core))
}
