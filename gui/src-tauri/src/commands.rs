//! Every #[tauri::command] — the Tauri layer's whole job is to run
//! the LIBRARY pipeline and hand the webview the report documents
//! (§5 — no second report form). Batch 4: every family face routes
//! through codeeraser::faces (the ONE adapter layer the MCP catalog
//! consumes too), every root a human typed goes through
//! codeeraser::root (the CLI/MCP/hook throat — the GUI was the one
//! surface still passing it raw, planting stray .ce/ dirs), and
//! every task emits `ce-task` start/done/error events so long runs
//! are visible instead of a frozen status line.

use serde_json::{json, Value};
use std::path::Path;
use tauri::Emitter;

/// The default root offered in the UI: the process working directory
/// ANCHORED (never a baked-in machine path, and never the raw cwd —
/// an installed launch starts in the install dir, which is nobody's
/// project; ascending to a real anchor is the honest offer, and the
/// UI shows what it resolved to).
#[tauri::command]
pub fn default_root() -> Result<String, String> {
    std::env::current_dir()
        .map(|p| codeeraser::root::project_root(&p).display().to_string())
        .map_err(|e| e.to_string())
}

/// Echo where a typed root actually anchors — the picker's honesty
/// line ("resolved to ..." when the ascent moved).
#[tauri::command]
pub fn resolve_root(root: String) -> Result<Value, String> {
    let (resolved, ascended) = codeeraser::root::resolve(Path::new(&root));
    Ok(json!({"root": resolved.display().to_string(), "ascended": ascended}))
}

/// The ONE core resolution — corelink's resolver chain (CE_CORE_BIN →
/// a `ce-core` sibling of this executable → PATH). The installers
/// stage ce-core as a sidecar beside the app binary, which is exactly
/// the sibling leg. A truly missing core still fails loudly — the
/// judgment run reports the spawn failure by name, same as the CLI.
fn core_path() -> String {
    codeeraser::corelink::resolve_core("ce-core")
}

/// The one task body: anchor the root, resolve the core, run the
/// library closure off the async runtime, and bracket it with
/// `ce-task` events. Four spawn_blocking twins were the P4 ratchet's
/// first catch; the measurement-only faces ride the same shape and
/// simply ignore the core argument (resolving is a string lookup —
/// only spawning can fail, and they never spawn it).
async fn task<F>(
    win: tauri::Window,
    name: &'static str,
    root: String,
    f: F,
) -> Result<Value, String>
where
    F: FnOnce(&Path, &str) -> anyhow::Result<Value> + Send + 'static,
{
    let _ = win.emit("ce-task", json!({"command": name, "state": "start"}));
    let out = tauri::async_runtime::spawn_blocking(move || {
        let root = codeeraser::root::project_root(Path::new(&root));
        f(&root, &core_path()).map_err(|e| format!("{e:#}"))
    })
    .await
    .map_err(|e| e.to_string())?;
    let state = if out.is_ok() { "done" } else { "error" };
    let _ = win.emit("ce-task", json!({"command": name, "state": state}));
    out
}

/// One full structure judgment — the same road `ce structure` drives.
/// split = the v0.7 advisory face (plan v2.7 ③), opt-in per scan.
#[tauri::command]
pub async fn structure_report(
    win: tauri::Window,
    root: String,
    deep: bool,
    days: Option<u32>,
    split: bool,
) -> Result<Value, String> {
    task(win, "structure", root, move |r, c| {
        codeeraser::faces::structure(r, c, (deep, days, split))
    })
    .await
}

/// Score trajectory (M7-P4): small `batch` values let the webview
/// show measuring progress; points cache in the project's index db.
#[tauri::command]
pub async fn trend_report(
    win: tauri::Window,
    root: String,
    commits: usize,
    batch: Option<usize>,
) -> Result<Value, String> {
    task(win, "trend", root, move |r, c| {
        codeeraser::faces::trend(r, c, commits, batch)
    })
    .await
}

/// The plain report commands differ only in name, event tag and
/// library face — the fn scaffold exists ONCE here. Ten hand-written
/// twins were the ratchet's first bite on this file (the tauri
/// macro's surface contract demands one fn per webview command, so
/// the fn is minted by a macro instead of by hand); the second arm
/// adds the days window the churn/join faces take. The truly
/// measurement-only faces (dedup, sites) ignore the core argument on
/// purpose: a missing core must not fail a face that never judges —
/// scan is NOT one of them (it grades, and since batch-7 slice 8 it
/// judges through the core like every graded surface). check is
/// report-only (faces charter): it never writes a baseline.
macro_rules! face_cmd {
    ($name:ident, $tag:literal, $body:expr) => {
        #[tauri::command]
        pub async fn $name(win: tauri::Window, root: String) -> Result<Value, String> {
            task(win, $tag, root, $body).await
        }
    };
    ($name:ident, $tag:literal, days: $body:expr) => {
        #[tauri::command]
        pub async fn $name(win: tauri::Window, root: String, days: u32) -> Result<Value, String> {
            // parenthesized: a closure-literal $body is not callable
            // as a bare `expr(args)` under Rust's call grammar
            task(win, $tag, root, move |r, c| ($body)(r, c, days)).await
        }
    };
}

// BOTH report thresholds ride since the face widened (K step 7):
// the screen pins neither, so the core's defaults answer — but a
// call site that named only min_tokens is exactly the silent pin
// the widening exists to make impossible
face_cmd!(dedup_report, "dedup", |r, _| codeeraser::faces::dedup(
    r, None, None
));
face_cmd!(scan_report, "scan", codeeraser::faces::scan);
face_cmd!(
    sites_report,
    "sites",
    |r, _| codeeraser::faces::graph_sites(r)
);
face_cmd!(deadcode_report, "deadcode", codeeraser::faces::deadcode);
face_cmd!(clone_report, "clone", codeeraser::faces::clone_t3);
face_cmd!(docdup_report, "docdup", codeeraser::faces::docdup);
face_cmd!(
    graphcanvas_report,
    "graphcanvas",
    codeeraser::faces::graph_canvas
);
/// The one face that takes an OPT-IN gate knob rather than a
/// measurement window, so it does not fit either macro arm: `floor`
/// is `--fail-under`, absent = the ratchet alone (the CLI's default).
/// Without it this screen could not reproduce the verdict CI prints,
/// and the two faces of one gate disagreed with nothing on screen to
/// say why.
#[tauri::command]
pub async fn check_report(
    win: tauri::Window,
    root: String,
    floor: Option<u32>,
) -> Result<Value, String> {
    task(win, "check", root, move |r, c| {
        codeeraser::faces::check(r, c, floor)
    })
    .await
}
face_cmd!(churn_report, "churn", days: |r, _c, d| codeeraser::faces::churn(r, d));
face_cmd!(join_report, "join", days: codeeraser::faces::join);

/// The machine's own state (K round step 6) — the SAME document
/// `ce doctor` renders, so two faces of a diagnostic cannot disagree
/// about the machine they diagnose. It never fails outward: a core
/// that will not answer IS the finding, and it rides the report's
/// `core.handshake` rather than an error the webview would show as a
/// red status line with no detail. No task() bracket either — this
/// is the face a user reaches for WHEN something is wrong, and it
/// must not depend on the event plumbing being healthy. It routes
/// through faces like every sibling: reaching health::doctor directly
/// worked, and made this module's opening claim ("every family face
/// routes through codeeraser::faces") false for exactly one screen —
/// which is how the two machine surfaces drift apart in the first
/// place. The face cannot fail, so the map_err arm is unreachable by
/// construction and stays only because the signature is shared.
#[tauri::command]
pub fn doctor_report(root: String) -> Result<Value, String> {
    codeeraser::faces::doctor(Path::new(&root), &core_path()).map_err(|e| format!("{e:#}"))
}

/// The update DOCUMENT — the same one `ce update` prints — off the
/// blocking pool because it reads the network. No root: the question
/// is about this build, not the opened project. The face cannot
/// fail (an unreachable index rides inside the document).
#[tauri::command]
pub async fn update_check() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(|| {
        codeeraser::faces::update_check().map_err(|e| format!("{e:#}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The apply leg — the SAME library entry `ce update --yes` drives
/// (one implementation, two faces): the check is re-measured here
/// rather than trusted from the webview, both binaries are verified
/// against the tag's pins before either is placed, and every refusal
/// (a plugin's or cargo's copy, a pin mismatch) surfaces by name.
#[tauri::command]
pub async fn update_apply(installer: bool) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let doc = codeeraser::update::document();
        codeeraser::update::apply::run(&doc, installer).map_err(|e| format!("{e:#}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The bench dashboard document, compiled in at build time — the
/// shipped GUI shows the bench of its OWN release (single source:
/// contracts/bench/bench.json; the renderers and CI gate live in
/// cli/tests/it/bench*.rs). No filesystem read: the user's opened
/// project has nothing to do with CE's own benchmarks.
#[tauri::command]
pub fn bench_doc() -> Result<Value, String> {
    serde_json::from_str(include_str!("../../../contracts/bench/bench.json"))
        .map_err(|e| e.to_string())
}

/// Both erase commands open with the SAME fresh plan (their two
/// bodies were a token twin by this repo's own measure) — the plan
/// is always re-measured, never carried over from the webview.
async fn erase_task(
    win: tauri::Window,
    tag: &'static str,
    root: String,
    f: fn(&Path, &str, codeeraser::erase::Plan) -> anyhow::Result<Value>,
) -> Result<Value, String> {
    task(win, tag, root, move |r, c| {
        let plan = codeeraser::erase::plan(r, None, c)?;
        f(r, c, plan)
    })
    .await
}

/// The erase PLAN plus its unified-diff preview in one measurement —
/// dry-run by definition (erase.md: the plan is read-only and both
/// faces come from the SAME plan, so the preview can never show
/// bytes the plan did not hash).
#[tauri::command]
pub async fn erase_preview(win: tauri::Window, root: String) -> Result<Value, String> {
    erase_task(win, "erase", root, |r, _, plan| {
        let diff = codeeraser::erase::render::diff(r, &plan)?;
        let mut doc = codeeraser::erase::render::report_json(&plan);
        doc["diff"] = json!(diff);
        Ok(doc)
    })
    .await
}

/// The destructive phase — the SAME library entry the CLI's --apply
/// drives (one implementation, two faces): preconditions in contract
/// order (git repo whose toplevel equals the root, clean worktree,
/// unchanged targets), writes, the audit log, and the convergence
/// re-plan. Every refusal surfaces by name in the webview.
#[tauri::command]
pub async fn erase_apply(win: tauri::Window, root: String) -> Result<Value, String> {
    erase_task(win, "erase_apply", root, |r, c, plan| {
        let applied = codeeraser::erase::apply_plan(r, None, c, &plan)?;
        Ok(json!({"applied": applied}))
    })
    .await
}
