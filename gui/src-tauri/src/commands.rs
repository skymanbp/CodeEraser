//! Every #[tauri::command] — the Tauri layer's whole job is to run
//! the LIBRARY pipeline and hand the webview the report documents
//! (§5 — no second report form). Batch 4: every family face routes
//! through codeeraser::faces (the ONE road the CLI and the MCP catalog
//! take too, plan v2.33 step 10: each report command's body is one
//! faces call, its parameters the family's document options — the
//! parity gate reads them off these signatures), every root a human
//! typed goes through
//! codeeraser::root (the CLI/MCP/hook throat — the GUI was the one
//! surface still passing it raw, planting stray .ce/ dirs), and
//! every task emits `ce-task` start/done/error events so long runs
//! are visible instead of a frozen status line.

use codeeraser::faces::{self, doc, Plumb};
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

/// The one task body: anchor the root, read the definition package
/// (its absence is the task's named error, shown as any other), resolve
/// the core (faces::core — the installers stage ce-core as a sidecar
/// beside the app binary, which is exactly its sibling leg; a truly
/// missing core still fails loudly, by name, same as the CLI), run the
/// library closure off the async runtime, and bracket
/// it with `ce-task` events. Four spawn_blocking twins were the P4 ratchet's
/// first catch; the measurement-only faces ride the same shape and
/// simply ignore the core argument (resolving is a string lookup —
/// only spawning can fail, and they never spawn it).
pub(crate) async fn task<F>(
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
        codeeraser::tables::anchor(&root);
        codeeraser::tables::load()?;
        f(&root, &faces::core()).map_err(|e| format!("{e:#}"))
    })
    .await
    .map_err(|e| e.to_string())?;
    let state = if out.is_ok() { "done" } else { "error" };
    let _ = win.emit("ce-task", json!({"command": name, "state": state}));
    out
}

// One full structure judgment — the same road `ce structure` drives.
// split = the v0.7 advisory face (plan v2.7 ③), opt-in per scan.
face_cmd!(structure_report, "structure", (deep: bool, days: Option<u32>, split: bool) =>
    |r, c, deep, days, split| doc(faces::structure(r, c, (deep, days, split), Plumb::default())));

// Score trajectory (M7-P4): small `batch` values let the webview
// show measuring progress; points cache in the project's index db.
face_cmd!(trend_report, "trend", (commits: usize, batch: Option<usize>) =>
    |r, c, n, b| doc(faces::trend(r, c, (n, b), Plumb::default())));

/// The report commands differ only in name, event tag, the document
/// options they take and the library face — the fn scaffold exists
/// ONCE here. Ten hand-written twins were the ratchet's first bite on
/// this file (the tauri macro's surface contract demands one fn per
/// webview command, so the fn is minted by a macro instead of by
/// hand); the second arm adds the options, each handed to the body
/// after the root and the core. Every face here takes the core: since
/// plan v2.33 step 10 each report the core lays out goes through the
/// same faces call the CLI makes. Shared with commands_query.rs and
/// commands_flow.rs, whose hand-written commands were the same clone.
macro_rules! face_cmd {
    // first: a plain $body:expr arm tried first would commit to
    // parsing `(days: u32)` as an expression and fail hard
    ($name:ident, $tag:literal, ($($a:ident: $t:ty),+) => $body:expr) => {
        #[tauri::command]
        pub async fn $name(
            win: tauri::Window,
            root: String,
            $($a: $t),+
        ) -> Result<serde_json::Value, String> {
            // parenthesized: a closure-literal $body is not callable
            // as a bare `expr(args)` under Rust's call grammar
            $crate::commands::task(win, $tag, root, move |r, c| ($body)(r, c, $($a),+)).await
        }
    };
    ($name:ident, $tag:literal, $body:expr) => {
        #[tauri::command]
        pub async fn $name(win: tauri::Window, root: String) -> Result<serde_json::Value, String> {
            $crate::commands::task(win, $tag, root, $body).await
        }
    };
}
pub(crate) use face_cmd;

// BOTH report thresholds ride (K step 7, `--min-tokens` /
// `--min-distinct`): absent = the core's defaults, the CLI's own.
face_cmd!(dedup_report, "dedup", (min_tokens: Option<usize>, min_distinct: Option<usize>) =>
    |r, c, t, d| faces::gated_doc(faces::dedup(r, c, (t, d), Plumb::default())));
face_cmd!(scan_report, "scan", |r, c| doc(faces::scan(r, c)));
face_cmd!(sites_report, "sites", |r, c| doc(faces::graph_sites(r, c)));
face_cmd!(deadcode_report, "deadcode", |r, c| doc(faces::deadcode(
    r,
    c,
    Plumb::default()
)));
face_cmd!(docdup_report, "docdup", |r, c| doc(faces::docdup(
    r,
    c,
    Plumb::default()
)));
// the audit trail's reader (O50), never a writer: the erase screen's
// third command reads what its second one appended
face_cmd!(erase_log_report, "erase_log", |r, c| doc(faces::erase_log(
    r, c
)));
face_cmd!(graphscreen_report, "graphscreen", faces::graph_screen);

// The T3 report, or under `units` the cached unit universe (`ce clone
// --units`; absent = the T3 report).
face_cmd!(clone_report, "clone", (units: Option<bool>) =>
    |r, c, u: Option<bool>| doc(faces::clone(r, c, u.unwrap_or(false), Plumb::default())));

// The gate's report: `floor` is `--fail-under` (absent = the ratchet
// alone, the CLI's default) and `days` the churn window `--days`
// arms (absent = the churn tables stay empty). Without them this
// screen could not reproduce the verdict CI prints, and two faces of
// one gate disagreed with nothing on screen to say why. Report-only:
// it never writes a baseline.
face_cmd!(check_report, "check", (days: Option<u32>, floor: Option<u32>) =>
    |r, c, d, f| doc(faces::check(r, c, (d, f), Plumb::default())));
face_cmd!(churn_report, "churn", (days: u32) => |r, c, d| doc(faces::churn(r, c, d)));
face_cmd!(join_report, "join", (days: u32) =>
    |r, c, d| doc(faces::join(r, c, d, Plumb::default())));

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
/// place. The face cannot fail; the `Ok` is the signature every
/// command shares.
#[tauri::command]
pub fn doctor_report(root: String) -> Result<Value, String> {
    Ok(faces::doctor(Path::new(&root), &faces::core()))
}

/// The update DOCUMENT — the same one `ce update` prints — off the
/// blocking pool because it reads the network. No root: the question
/// is about this build, not the opened project. The face cannot
/// fail (an unreachable index rides inside the document).
#[tauri::command]
pub async fn update_check() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(faces::update_check)
        .await
        .map_err(|e| e.to_string())
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

// The erase PLAN plus its unified-diff preview in one measurement —
// dry-run by definition (erase.md: the plan is read-only and both
// faces come from the SAME plan, so the preview can never show bytes
// the plan did not hash); the document is the core's, the one the CLI
// prints and the MCP tool hands out (plan v2.32 step 6)
face_cmd!(erase_preview, "erase", faces::erase_preview);

// The destructive phase — the SAME road the CLI's --apply drives: the
// erase face plans afresh (never a plan carried over from the webview)
// and hands that plan to the mutation, the preconditions in contract
// order (git repo whose toplevel equals the root, clean worktree,
// unchanged targets), writes, the audit log and the convergence
// re-plan; the answer is the erase DOCUMENT of the plan applied, and a
// refusal surfaces by name in the webview.
face_cmd!(erase_apply, "erase_apply", |r, c| {
    use codeeraser::erase::{self, document::Diffs, Plan};
    let apply = |plan: &Plan, _: &Diffs| Some(erase::apply_plan(r, None, c, plan));
    match faces::erase(r, c, Plumb::default(), apply)? {
        (_, Some(refused)) => Err(refused),
        (answer, None) => Ok(answer.document),
    }
});
