// Prevents an extra console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! CodeEraser GUI (M6 S4a): the Tauri shell's whole job is to run
//! the LIBRARY pipeline and hand the webview the one report
//! document (§5 — no second report form). Judgment stays in the
//! Haskell core; measurement stays in the codeeraser crate; the JS
//! side is rendering glue and nothing else.

use std::path::Path;

/// The default root offered in the UI: the process working
/// directory (never a baked-in machine path).
#[tauri::command]
fn default_root() -> Result<String, String> {
    std::env::current_dir()
        .map(|p| p.display().to_string())
        .map_err(|e| e.to_string())
}

/// One full structure judgment — the same road `ce structure`
/// drives, blocking work moved off the async runtime. The core
/// binary resolves like the CLI: CE_CORE_BIN, spoken loudly when
/// absent.
#[tauri::command]
async fn structure_report(
    root: String,
    deep: bool,
    days: Option<u32>,
) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let core = std::env::var("CE_CORE_BIN")
            .map_err(|_| "CE_CORE_BIN is unset — build the core and export it".to_string())?;
        let r = codeeraser::structure::judge::run(Path::new(&root), None, &core, deep, days)
            .map_err(|e| format!("{e:#}"))?;
        Ok(codeeraser::structure::judge::report_json(&r))
    })
    .await
    .map_err(|e| e.to_string())?
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![default_root, structure_report])
        .run(tauri::generate_context!())
        .expect("tauri run");
}
