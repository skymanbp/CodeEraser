//! The MCP argument adapters: one fn per catalog row, each turning
//! (root, arguments) into a family's public report string, plus the
//! three argument readers they share. Split from tools.rs when the
//! catalog reached thirteen rows and the file crossed the 300-line
//! line — the module's own doc already named these as two jobs (the
//! TABLE is the authority for tools/list and dispatch; these are the
//! transport). Read-only by construction, same charter: every body
//! ends at a family face, and `erase` reaches the PLAN alone.

use anyhow::Result;
use serde_json::Value;
use std::path::Path;

/// CE_CORE_BIN → sibling → PATH: the daemon's resolver, one authority.
fn core() -> String {
    crate::daemon::judge::core_bin().unwrap_or_else(|| "ce-core".into())
}

/// A count arg, or the default — ONE guard for every window. `as u32`
/// TRUNCATED (`days = 4294967296` became 0 and judged a zero-day
/// window) and `commits` took `as usize` bare beside it, so a zero
/// there judged an EMPTY history: absent, unparsable, oversized and
/// zero all mean the default now.
pub(super) fn count(args: &Value, key: &str, default: usize) -> usize {
    args[key]
        .as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .filter(|&v| v > 0)
        .unwrap_or(default)
}

fn days(args: &Value, default: u32) -> u32 {
    let v = count(args, "days", default as usize);
    u32::try_from(v).unwrap_or(default)
}

// Every adapter body below is one call into crate::faces — the ONE
// JSON face per family both machine surfaces consume (batch 4 lifted
// the bodies there when the GUI backend was about to copy them; the
// P4 cross-surface-shell precedent). This file keeps only the MCP
// concerns: the catalog table, arg parsing, and stringification.

// scan and graph_sites are the two STRING-native library faces, and
// mcp_precommit byte-pins this catalog to them (the catalog is a
// transport, never a second serializer) — a Value round-trip
// re-orders keys and drops the pretty form, so these two stay on the
// direct string faces; faces::scan/graph_sites serve the GUI's
// Value-shaped consumption of the same documents. Since batch-7
// slice 8 scan judges through the core like every verdict surface
// (the byte pin survives: findings ≡ mirror by the drift ensure).
pub(super) fn scan(root: &Path, _a: &Value) -> Result<String> {
    let (files, findings, summary, _fail, failed) = crate::scan::analyze_judged(root, &core())?;
    crate::scan::report_string(&files, &findings, summary, &failed)
}

pub(super) fn check_duplication(root: &Path, a: &Value) -> Result<String> {
    let n = |k: &str| {
        a[k].as_u64()
            .map(|v| usize::try_from(v).unwrap_or(usize::MAX))
    };
    Ok(crate::faces::dedup(root, n("min_tokens"), n("min_distinct"))?.to_string())
}

pub(super) fn churn(root: &Path, a: &Value) -> Result<String> {
    Ok(crate::faces::churn(root, days(a, 14))?.to_string())
}

pub(super) fn graph_sites(root: &Path, _a: &Value) -> Result<String> {
    Ok(crate::graph::sites_json(&crate::graph::analyze(root)?))
}

/// The plain judged faces (no extra knobs) share ONE adapter body —
/// the census caught their three shells chaining into clone blocks;
/// the shell exists once and thin per-name fns satisfy the table's
/// fn-pointer field (the pre-batch-4 banked shape, now over faces).
fn judged(root: &Path, which: &str) -> Result<String> {
    let core = core();
    let doc = match which {
        "deadcode" => crate::faces::deadcode(root, &core)?,
        "clone" => crate::faces::clone_t3(root, &core)?,
        "docdup" => crate::faces::docdup(root, &core)?,
        other => anyhow::bail!("not a plain judged face: {other}"),
    };
    Ok(doc.to_string())
}

/// The adapters that differ by ONE string. The catalog needs a
/// distinct fn pointer per row, so the three-line body `judged(root,
/// NAME)` was typed once per family — and once the split put them
/// side by side this repo's own clone gate counted them as a block,
/// which is the correct reading: they are one function wearing three
/// names. Minted here instead (the face_cmd! precedent).
macro_rules! plain {
    ($($name:ident => $family:literal),+ $(,)?) => { $(
        pub(super) fn $name(root: &Path, _a: &Value) -> Result<String> {
            judged(root, $family)
        }
    )+ };
}

plain!(deadcode => "deadcode", docdup => "docdup");

/// Not plain: `units` switches this row to the OTHER document its own
/// CLI flag produces, so the branch is real and stays written out.
pub(super) fn clone_report(root: &Path, a: &Value) -> Result<String> {
    if a["units"].as_bool().unwrap_or(false) {
        return Ok(crate::faces::clone_units(root)?.to_string());
    }
    judged(root, "clone")
}

pub(super) fn join(root: &Path, a: &Value) -> Result<String> {
    Ok(crate::faces::join(root, &core(), days(a, 14))?.to_string())
}

pub(super) fn structure(root: &Path, a: &Value) -> Result<String> {
    let deep = a["deep"].as_bool().unwrap_or(false);
    // absent = axis 5 unjudged (the honest Option), but a PRESENT and
    // unusable value must not truncate into a zero-day window
    let d = a["days"]
        .as_u64()
        .map(|v| u32::try_from(v).unwrap_or(u32::MAX).max(1));
    // the split advisory joins the MCP face in v0.7 (plan v2.7 ③):
    // same report schema, same rows the CLI prints
    let split = a["split"].as_bool().unwrap_or(false);
    Ok(crate::faces::structure(root, &core(), (deep, d, split))?.to_string())
}

pub(super) fn check(root: &Path, a: &Value) -> Result<String> {
    // absent = the ratchet alone, the CLI's own default; a PRESENT
    // value arms the same floor `--fail-under` does, so this surface
    // can reproduce the verdict a pipeline prints rather than a
    // weaker one that happens to agree most days
    let floor = a["floor"]
        .as_u64()
        .map(|v| u32::try_from(v).unwrap_or(u32::MAX));
    Ok(crate::faces::check(root, &core(), floor)?.to_string())
}

pub(super) fn erase(root: &Path, _a: &Value) -> Result<String> {
    Ok(crate::faces::erase(root, &core())?.to_string())
}

/// The one tool whose FINDING may be a failure: a core that will not
/// answer rides inside the document rather than as a tool error, so
/// the caller reading it learns the state instead of an exception.
pub(super) fn doctor(root: &Path, _a: &Value) -> Result<String> {
    Ok(crate::faces::doctor(root, &core())?.to_string())
}

/// The one tool about the BINARY rather than the project: `path`
/// rides its schema like every row's and is ignored, because the
/// answer does not depend on which tree asked.
pub(super) fn update_check(_root: &Path, _a: &Value) -> Result<String> {
    Ok(crate::faces::update_check()?.to_string())
}

/// Exactly one ask (the CLI's clap group, spelled here for JSON):
/// `at` / `text` / `unit`; `widen` opts into the associative view.
pub(super) fn similar_units(root: &Path, a: &Value) -> Result<String> {
    let ask = crate::similar::query::Ask::from_parts(
        a["at"].as_str(),
        a["text"].as_str(),
        a["unit"].as_str(),
    )?;
    let widen = a["widen"].as_bool().unwrap_or(false);
    Ok(crate::faces::similar(root, &core(), &ask, widen)?.to_string())
}

pub(super) fn trend(root: &Path, a: &Value) -> Result<String> {
    let commits = count(a, "commits", crate::trend::DEFAULT_COMMITS);
    // absent = measure every uncached commit, but a PRESENT batch of 0
    // measured NOTHING and left `pending` pinned for a GUI polling it
    let batch = a["batch"]
        .as_u64()
        .map(|v| usize::try_from(v).unwrap_or(usize::MAX).max(1));
    Ok(crate::faces::trend(root, &core(), commits, batch)?.to_string())
}
