// Prevents an extra console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! CodeEraser GUI (M6 S4a shell + M7-P4 second screen): the Tauri
//! layer's whole job is to run the LIBRARY pipeline and hand the
//! webview the report documents (§5 — no second report form). P4
//! adds the trend and deletion-candidate faces over the very
//! report_json throats the CLI prints — judgment stays in the
//! Haskell core; measurement stays in the codeeraser crate; the JS
//! side is rendering glue and nothing else.

use serde_json::Value;
use std::path::Path;

/// The default root offered in the UI: the process working
/// directory (never a baked-in machine path).
#[tauri::command]
fn default_root() -> Result<String, String> {
    std::env::current_dir()
        .map(|p| p.display().to_string())
        .map_err(|e| e.to_string())
}

/// The ONE core resolution — corelink's resolver chain (CE_CORE_BIN →
/// a `ce-core` sibling of this executable → PATH). The installers
/// stage ce-core as a sidecar beside the app binary, which is exactly
/// the sibling leg; the S4a env-only read told installer users to go
/// build the Haskell core while it sat next door. A truly missing
/// core still fails loudly — the judgment run reports the spawn
/// failure by name, same as the CLI.
fn core_path() -> String {
    codeeraser::corelink::resolve_core("ce-core")
}

/// The one judged-command body: resolve the core, run the library
/// closure off the async runtime, format errors. Four spawn_blocking
/// twins were the P4 ratchet's first catch — the shape now exists
/// once and each command is its one-expression library delegation.
async fn judged<F>(f: F) -> Result<Value, String>
where
    F: FnOnce(&str) -> anyhow::Result<Value> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let core = core_path();
        f(&core).map_err(|e| format!("{e:#}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// One full structure judgment — the same road `ce structure` drives.
#[tauri::command]
async fn structure_report(
    root: String,
    deep: bool,
    days: Option<u32>,
    split: bool,
) -> Result<Value, String> {
    use codeeraser::structure::judge;
    judged(move |c| {
        Ok(judge::report_json(&judge::run(
            Path::new(&root),
            None,
            c,
            // split = the v0.7 advisory face (plan v2.7 ③): the same
            // priced rows the CLI prints, opt-in per scan (the seam
            // measurement rides dedup + churn and is not first-screen
            // freight)
            (deep, days, split),
        )?))
    })
    .await
}

/// Score trajectory over mainline history (M7-P4): the same
/// ce.trend-report document `ce trend --format json` prints. Small
/// `batch` values let the webview show measuring progress; the
/// points cache in the project's own index db and rebuild from git.
#[tauri::command]
async fn trend_report(root: String, commits: usize, batch: Option<usize>) -> Result<Value, String> {
    use codeeraser::trend;
    judged(move |c| {
        Ok(trend::report_json(&trend::run(
            Path::new(&root),
            None,
            c,
            commits,
            batch,
        )?))
    })
    .await
}

/// Three-signal join rows for the deletion-candidate browser — the
/// same ce.join-report document `ce join --format json` prints.
#[tauri::command]
async fn join_report(root: String, days: u32) -> Result<Value, String> {
    use codeeraser::join;
    judged(move |c| {
        Ok(join::report_json(&join::run(
            Path::new(&root),
            None,
            c,
            days,
        )?))
    })
    .await
}

/// T1/T2 clone blocks (the candidate browser's block list) — the
/// same ce.dedup-report document. Measurement only, so it stays OFF
/// the judged() road on purpose: a missing core must not fail a
/// face that never judges.
#[tauri::command]
async fn dedup_report(root: String) -> Result<Value, String> {
    use codeeraser::dedup;
    tauri::async_runtime::spawn_blocking(move || {
        let (found, summary) =
            dedup::analyze(Path::new(&root), None, None, None).map_err(|e| format!("{e:#}"))?;
        dedup::report_json(&found, &summary).map_err(|e| format!("{e:#}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            default_root,
            structure_report,
            trend_report,
            join_report,
            dedup_report
        ])
        .run(tauri::generate_context!())
        .expect("tauri run");
}
