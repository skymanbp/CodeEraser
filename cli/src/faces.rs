//! One JSON document per report family — the adapter layer BOTH
//! machine surfaces consume: the MCP catalog stringifies these, the
//! GUI backend returns them as-is. Lifted when batch 4 was about to
//! copy the MCP adapter bodies into the Tauri commands (the P4
//! ratchet precedent: cross-surface shells chain into clone blocks).
//! Read-only by construction, same as the MCP charter: every face
//! ends at a family's public report serialization — none writes a
//! baseline or config. Callers resolve the root and the core; a face
//! only turns (root, knobs) into the family's one document.

use crate::document::Answer;
use anyhow::Result;
use serde_json::Value;
use std::path::Path;

/// The document of a core-laid answer: what every machine face hands
/// out (the console lines and the veto are the CLI's alone).
fn laid(answer: Result<Answer>) -> Result<Value> {
    Ok(answer?.document)
}

/// Judged like its siblings (batch-7 slice 8): the scan face used
/// to read the mirror with no core link — the one unguarded copy of
/// a rule the core owns; it reads the core's levels like every other
/// surface, and the core lays the document out (plan v2.32 step 5).
pub fn scan(root: &Path, core: &str) -> Result<Value> {
    Ok(crate::scan::judged(root, core)?.document)
}

/// Both report thresholds ride, not one: `min_distinct` is the
/// DIVERSITY floor (`ce dedup --min-distinct`), and a face that
/// accepted only `min_tokens` could not reproduce what the CLI
/// prints — the caller was silently pinned to the core default. The
/// document is the core's (plan v2.32 step 5), laid out by the
/// process's core.
pub fn dedup(root: &Path, min_tokens: Option<usize>, min_distinct: Option<usize>) -> Result<Value> {
    let (found, summary) = crate::dedup::analyze(root, None, min_tokens, min_distinct)?;
    crate::dedup::report_json(&found, &summary)
}

/// The churn window, laid out by the core at `core` (plan v2.32 step 5).
pub fn churn(root: &Path, core: &str, days: u32) -> Result<Value> {
    laid(crate::churn::answer(
        core,
        &crate::churn::run(root, days)?,
        days,
    ))
}

pub fn graph_sites(root: &Path, core: &str) -> Result<Value> {
    laid(crate::graph::sites_document(
        core,
        &crate::graph::analyze(root)?,
    ))
}

pub fn deadcode(root: &Path, core: &str) -> Result<Value> {
    laid(crate::graph::deadcode::answer(root, None, core, false))
}

/// The T3 report, laid out by the core (plan v2.32 step 5).
pub fn clone_t3(root: &Path, core: &str) -> Result<Value> {
    Ok(crate::dedup::t3::answer(root, None, core)?.document)
}

/// The docdup report, laid out by the core (plan v2.32 step 5).
pub fn docdup(root: &Path, core: &str) -> Result<Value> {
    Ok(crate::docdup::judge::answer(root, None, core, false)?.document)
}

pub fn join(root: &Path, core: &str, days: u32) -> Result<Value> {
    laid(crate::join::run(root, None, core, days))
}

pub fn structure(root: &Path, core: &str, knobs: (bool, Option<u32>, bool)) -> Result<Value> {
    laid(crate::structure::judge::run(root, None, core, knobs))
}

/// Report-only: this face never writes a baseline (MCP charter ③;
/// the GUI apply road goes through erase, never through establish).
/// `floor` is the CLI's `--fail-under`, opt-in on every road — but a
/// face that could not arm it could not reproduce the verdict CI
/// prints, and the report now echoes which it judged under.
pub fn check(root: &Path, core: &str, floor: Option<u32>) -> Result<Value> {
    // the one read (O31): a face judges the document it read, and a
    // file that is not a baseline is this face's named error too
    let baseline = crate::score::baseline::read(root)?;
    let mut o = crate::score::run(
        root,
        crate::score::Opts {
            db: None,
            core: core.into(),
            days: None,
            floor,
            establish: false,
            pinned_soft: None,
            baseline,
        },
    )?;
    laid(crate::score::document::document(core, &mut o, false))
}

pub fn trend(root: &Path, core: &str, commits: usize, batch: Option<usize>) -> Result<Value> {
    Ok(crate::trend::judged(root, None, core, commits, batch)?.document)
}

/// The GUI's graph screen (plan v2.30 step 5b item 31): the canvas
/// document and the deadcode report off ONE deadcode-family judgment
/// — the assembly lives in graph::canvas. Read-only like every face.
pub fn graph_screen(root: &Path, core: &str) -> Result<Value> {
    crate::graph::canvas::screen(root, core)
}

/// The cached unit universe (`ce clone --units`): the document used
/// to be built inline in the CLI's own command body, so it was the
/// one family document no machine surface could reach. The identity
/// assertion travels with it — a face that listed units without
/// checking the unitsig/symbols agreement would hand out a universe
/// nobody had checked. The core lays the listing out (plan v2.32
/// step 5).
pub fn clone_units(root: &Path, core: &str) -> Result<Value> {
    Ok(crate::dedup::t3::units_answer(core, &listed_units(root)?)?.document)
}

/// The cached unit universe, the identity agreement checked.
pub fn listed_units(root: &Path) -> Result<Vec<crate::dedup::unitcache::UnitRow>> {
    let (idx, _db) = crate::dedup::refreshed_index(root, None)?;
    let orphans = crate::dedup::unitcache::identity_orphans(&idx)?;
    anyhow::ensure!(
        orphans == 0,
        "{orphans} unitsig rows missing their symbols identity — nth throat drift"
    );
    crate::dedup::unitcache::unit_rows(&idx)
}

/// The erase PLAN — dry-run by definition and by construction: this
/// face reaches `erase::plan`, which is read-only, and there is no
/// face for `apply_plan` at all. That absence is the charter, not an
/// omission: a machine surface that could delete files on its own
/// authority is the one thing an eraser must never ship.
pub fn erase(root: &Path, core: &str) -> Result<Value> {
    Ok(erase_laid(root, core)?.0)
}

/// The GUI's erase preview: the same plan document and, under `diff`,
/// the unified diff over the SAME plan — one measurement, so the
/// preview can never show bytes the plan did not hash (erase.md). The
/// document is the core's (plan v2.32 step 6), the diff this side's.
pub fn erase_preview(root: &Path, core: &str) -> Result<Value> {
    let (mut doc, diffs) = erase_laid(root, core)?;
    doc["diff"] = Value::String(diffs.unified());
    Ok(doc)
}

/// One fresh plan laid out by the core, and its per-file diffs.
fn erase_laid(root: &Path, core: &str) -> Result<(Value, crate::erase::document::Diffs)> {
    let (plan, held) = crate::erase::planned(root, None, core)?;
    let diffs = crate::erase::document::Diffs::of(root, &plan)?;
    let run = crate::erase::document::Run::default();
    let doc = crate::erase::document::answer(core, held, &plan, &diffs, run)?.document;
    Ok((doc, diffs))
}

/// The applied-erase audit trail (plan v2.29 step 9, O50): apply.rs
/// is its one writer and this is its reader — every face renders the
/// same document, and none can append to it. Read-only like every
/// sibling; a line the reader cannot parse rides inside the document
/// by line number rather than failing the face. The process's core lays
/// the document out (plan v2.32 step 5); the face keeps its signature,
/// so its GUI and MCP callers pass no core.
pub fn erase_log(root: &Path) -> Result<Value> {
    let log = crate::erase::log::read(root)?;
    let core = crate::tables::core_flag();
    Ok(crate::erase::document::trail_answer(core, &log)?.document)
}

/// The same-role advisor's document (plan v2.29 step 6): one ask —
/// a unit at `file:line`, a unit by key, free text — and the
/// associative view under `widen`. Advisory like the deadcode
/// symbol table: order and role bits are the core's answer over
/// similar/1, a core that cannot answer is NAMED in the document, and
/// nothing here reaches `ce check`.
pub fn similar(
    root: &Path,
    core: &str,
    ask: &crate::similar::query::Ask,
    widen: bool,
) -> Result<Value> {
    Ok(crate::similar::document::answer(root, None, core, ask, widen)?.document)
}

/// One question over the index's facts (plan v2.31 step 2): the
/// rules file's rules (the one named, else the project's) under the
/// question, judged by the core's query/1 and labelled back — the
/// SAME document `ce query --format json` prints. Report-only: a
/// program error or a core that could not judge is named in the
/// document, never a verdict this side reached.
pub fn query(root: &Path, core: &str, body: &str, why: bool, file: Option<&Path>) -> Result<Value> {
    let ask = crate::query::face::Ask {
        rules: crate::query::rules_source(root, file)?,
        query: Some(body.to_string()),
        why,
    };
    laid(crate::query::face::run(root, None, core, &ask))
}

/// The rules file judged (plan v2.31 step 2): every assertion's
/// violations with their derivations — the document `ce rules`
/// prints and exits on; here it is a report, the exit code is the
/// core's veto the CLI reads.
pub fn rules(root: &Path, core: &str, file: Option<&Path>, why: bool) -> Result<Value> {
    let ask = crate::query::face::Ask {
        rules: crate::query::rules_source(root, file)?,
        query: None,
        why,
    };
    laid(crate::query::face::run(root, None, core, &ask))
}

/// The dead code inside functions (plan v2.31 step 5): every unit
/// of the tree judged over flow/1 and placed back through its legend
/// — the SAME document `ce flow --format json` prints, `kinds`
/// narrowing the listed findings as `--kind` does. Report-only: the
/// CLI's `--check` is the core's veto over the same document.
pub fn flow(root: &Path, core: &str, kinds: &[String]) -> Result<Value> {
    laid(crate::flow_report::face::run(
        root,
        core,
        kinds,
        (false, false),
    ))
}

/// The clone-merge suggestions (plan v2.31 step 7): every clone
/// group anti-unified by the core's merge/1 and labelled back — the
/// SAME document `ce merge --format json` prints. Advisory: a core
/// that cannot judge is named in the document, and no gate reads it.
pub fn merge(root: &Path, core: &str) -> Result<Value> {
    laid(crate::merge::face::run(root, None, core, None))
}

/// The architecture of the tree (plan v2.31 step 9): layers, the
/// arcs to cut out of the directory cycles, clusters, misplaced
/// files, the impact of `impact`'s files and per-directory metrics,
/// judged by the core's arch/1 — the SAME document `ce arch --format
/// json` prints. Advisory: a core without the family is named in the
/// document, and nothing here reaches a gate.
pub fn arch(root: &Path, core: &str, impact: &[String]) -> Result<Value> {
    laid(crate::arch::face::run(root, None, core, impact))
}

/// The machine's own state. Unlike every sibling it cannot fail: a
/// core that will not answer IS the finding, and it rides inside the
/// document (health::doctor).
pub fn doctor(root: &Path, core: &str) -> Result<Value> {
    Ok(crate::health::doctor::document(root, core))
}

/// Whether a newer release exists — the update DOCUMENT (the
/// doctor's shape of charter: one measurement, every face renders
/// it). Takes no root and no core: the question is about THIS
/// binary. Read-only like every face — it reads the release index
/// and the tag's committed pins and places nothing; the apply leg
/// (`update::apply::run`) is a human act at the CLI or the GUI, the
/// same line the erase plan draws.
pub fn update_check() -> Result<Value> {
    Ok(crate::update::document())
}
