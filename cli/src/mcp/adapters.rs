//! The MCP argument adapters: one fn per catalog row, each turning
//! (root, arguments) into a family's public report string, plus the
//! three argument readers they share. Split from tools.rs when the
//! catalog reached thirteen rows and the file crossed the 300-line
//! line — the module's own doc already named these as two jobs (the
//! TABLE is the authority for tools/list and dispatch; these are the
//! transport). Read-only by construction, same charter: every body
//! ends at a family face, and `erase` reaches the PLAN alone.

use crate::faces::{self, Plumb, doc};
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

/// An optional window or floor: absent stays None (the honest Option:
/// that axis unjudged, that floor unarmed), and a PRESENT value past
/// u32 saturates rather than truncating into a small one.
fn opt_u32(args: &Value, key: &str) -> Option<u32> {
    args[key]
        .as_u64()
        .map(|v| u32::try_from(v).unwrap_or(u32::MAX))
}

/// A face's answer as the tool's string: the document, one
/// serialization for every row.
fn shown(answer: Result<crate::document::Answer>) -> Result<String> {
    Ok(doc(answer)?.to_string())
}

// Every adapter body below is one call into crate::faces — the ONE
// road every face takes to a family's answer (batch 4 lifted the
// bodies there when the GUI backend was about to copy them; the CLI
// joined at plan v2.33 step 10). This file keeps only the MCP
// concerns: arg parsing and stringifying the document; no gate is
// armed here (Plumb::default()).

// scan prints the document the core lays out (plan v2.32 step 5) in
// the form its `--format json` face prints — document::rendered, the
// catalogue states scan `pretty` — and mcp_precommit byte-pins this
// catalog to that face (the catalog is a transport, never a second
// serializer). graph_sites prints its document (step 4), one
// serialization like every sibling.
pub(super) fn scan(root: &Path, _a: &Value) -> Result<String> {
    crate::document::rendered("scan", &doc(faces::scan(root, &core()))?)
}

pub(super) fn check_duplication(root: &Path, a: &Value) -> Result<String> {
    let n = |k: &str| {
        a[k].as_u64()
            .map(|v| usize::try_from(v).unwrap_or(usize::MAX))
    };
    let thresholds = (n("min_tokens"), n("min_distinct"));
    let answer = faces::dedup(root, &core(), thresholds, Plumb::default());
    Ok(faces::gated_doc(answer)?.to_string())
}

pub(super) fn graph_sites(root: &Path, _a: &Value) -> Result<String> {
    shown(faces::graph_sites(root, &core()))
}

/// The plain faces (no extra knobs) share ONE adapter body — the
/// census caught the three judged shells chaining into clone blocks,
/// and the trail reader (O50) made `erase` / `erase_log` / `doctor`
/// the next such chain; the shell exists once and thin per-name fns
/// satisfy the table's fn-pointer field (the pre-batch-4 banked
/// shape, now over faces). `erase` reaches the PLAN alone and
/// `erase_log` reads the trail it leaves (the process's core lays its
/// document out, plan v2.32 step 5); `doctor` is
/// the one tool whose FINDING may be a failure — a core that will not
/// answer rides inside its document rather than as a tool error, so
/// the caller reading it learns the state instead of an exception.
fn plain_face(root: &Path, which: &str) -> Result<String> {
    let (c, p) = (core(), Plumb::default());
    let document = match which {
        "deadcode" => doc(faces::deadcode(root, &c, p)),
        "docdup" => doc(faces::docdup(root, &c, p)),
        "erase" => faces::gated_doc(faces::erase(root, &c, p, |_, _| None)),
        "erase_log" => doc(faces::erase_log(root, &c)),
        "doctor" => Ok(faces::doctor(root, &c)),
        "merge" => doc(faces::merge(root, &c, p)),
        other => Err(anyhow::anyhow!("not a plain face: {other}")),
    };
    Ok(document?.to_string())
}

/// The adapters that differ by ONE string. The catalog needs a
/// distinct fn pointer per row, so the three-line body
/// `plain_face(root, NAME)` was typed once per family — and once the
/// split put them side by side this repo's own clone gate counted them
/// as a block, which is the correct reading: they are one function
/// wearing several names. Minted here instead (the face_cmd! precedent).
macro_rules! plain {
    ($($name:ident => $family:literal),+ $(,)?) => { $(
        pub(super) fn $name(root: &Path, _a: &Value) -> Result<String> {
            plain_face(root, $family)
        }
    )+ };
}

plain!(
    deadcode => "deadcode",
    docdup => "docdup",
    erase => "erase",
    erase_log => "erase_log",
    doctor => "doctor",
    merge_suggestions => "merge",
);

/// Not plain: `units` switches this row to the OTHER document its own
/// CLI flag produces (`ce clone --units`).
pub(super) fn clone_report(root: &Path, a: &Value) -> Result<String> {
    let units = a["units"].as_bool().unwrap_or(false);
    shown(faces::clone(root, &core(), units, Plumb::default()))
}

/// The faces over a git window (`days`, default 14) that ask the core:
/// one body, a name per catalog row (the plain! precedent; churn
/// joined join here when its document moved into the core, plan v2.32
/// step 5 R0, and the two bodies were one clone block).
pub(super) fn churn(root: &Path, a: &Value) -> Result<String> {
    shown(faces::churn(root, &core(), days(a, 14)))
}

pub(super) fn join(root: &Path, a: &Value) -> Result<String> {
    let answer = faces::join(root, &core(), days(a, 14), Plumb::default());
    shown(answer)
}

pub(super) fn structure(root: &Path, a: &Value) -> Result<String> {
    let deep = a["deep"].as_bool().unwrap_or(false);
    // a PRESENT and unusable value must not become a zero-day window
    let d = opt_u32(a, "days").map(|v| v.max(1));
    // the split advisory joins the MCP face in v0.7 (plan v2.7 ③):
    // same report schema, same rows the CLI prints
    let split = a["split"].as_bool().unwrap_or(false);
    let answer = faces::structure(root, &core(), (deep, d, split), Plumb::default());
    shown(answer)
}

pub(super) fn check(root: &Path, a: &Value) -> Result<String> {
    // absent = the ratchet alone, the CLI's own default; a PRESENT
    // value arms the same floor `--fail-under` does, so this surface
    // can reproduce the verdict a pipeline prints rather than a
    // weaker one that happens to agree most days
    // the churn window `--days` arms (absent = the churn tables stay
    // empty, the CLI's own default)
    let window = (opt_u32(a, "days"), opt_u32(a, "floor"));
    let answer = faces::check(root, &core(), window, Plumb::default());
    shown(answer)
}

/// The one tool about the BINARY rather than the project: `path`
/// rides its schema like every row's and is ignored, because the
/// answer does not depend on which tree asked.
pub(super) fn update_check(_root: &Path, _a: &Value) -> Result<String> {
    Ok(faces::update_check().to_string())
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
    let answer = faces::similar(root, &core(), &ask, widen, Plumb::default());
    shown(answer)
}

/// `prelude` (true) answers the pack text instead of a question — the
/// prelude and the `[rules.pack]` program, `ce query --prelude`'s two
/// strings as one document; `body` is required otherwise.
pub(super) fn query(root: &Path, a: &Value) -> Result<String> {
    if a["prelude"].as_bool().unwrap_or(false) {
        return Ok(faces::pack_text(root, &core())?.to_string());
    }
    let body = a["body"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("query: `body` (the question) is required"))?;
    let (file, why) = file_why(a);
    let answer = faces::query(root, &core(), (body, why), file, Plumb::default());
    shown(answer)
}

pub(super) fn rules(root: &Path, a: &Value) -> Result<String> {
    let (file, why) = file_why(a);
    shown(faces::rules(root, &core(), file, why, Plumb::default()))
}

pub(super) fn flow(root: &Path, a: &Value) -> Result<String> {
    let kinds: Vec<String> = a["kind"].as_str().map(String::from).into_iter().collect();
    shown(faces::flow(root, &core(), &kinds, Plumb::default()))
}

/// The two arguments both query tools take: the rules file (root-
/// relative, optional) and whether every answer carries its derivation.
fn file_why(a: &Value) -> (Option<&Path>, bool) {
    (
        a["file"].as_str().map(Path::new),
        a["why"].as_bool().unwrap_or(false),
    )
}

/// `impact`: the root-relative paths whose dependents to trace; a
/// non-string entry is refused by name rather than dropped.
pub(super) fn architecture(root: &Path, a: &Value) -> Result<String> {
    let impact = match &a["impact"] {
        Value::Null => Vec::new(),
        v => serde_json::from_value::<Vec<String>>(v.clone())
            .map_err(|e| anyhow::anyhow!("architecture: `impact` must be a list of paths: {e}"))?,
    };
    let answer = faces::arch(root, &core(), &impact, Plumb::default());
    shown(answer)
}

pub(super) fn trend(root: &Path, a: &Value) -> Result<String> {
    let commits = count(a, "commits", crate::trend::DEFAULT_COMMITS);
    // absent = measure every uncached commit, but a PRESENT batch of 0
    // measured NOTHING and left `pending` pinned for a GUI polling it
    let batch = a["batch"]
        .as_u64()
        .map(|v| usize::try_from(v).unwrap_or(usize::MAX).max(1));
    let answer = faces::trend(root, &core(), (commits, batch), Plumb::default());
    shown(answer)
}
