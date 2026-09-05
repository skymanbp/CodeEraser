// Prevents an extra console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! CodeEraser GUI shell (M6 S4a; batch 4 = the complete face): the
//! command surface lives in commands.rs — every report family the
//! CLI prints, the erase preview/apply pair, root anchoring through
//! codeeraser::root, and `ce-task` progress events. This file keeps
//! only the builder and the handler roster.

mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::default_root,
            commands::resolve_root,
            commands::structure_report,
            commands::trend_report,
            commands::join_report,
            commands::dedup_report,
            commands::scan_report,
            commands::churn_report,
            commands::sites_report,
            commands::deadcode_report,
            commands::clone_report,
            commands::docdup_report,
            commands::graphcanvas_report,
            commands::check_report,
            commands::similar_report,
            commands::erase_preview,
            commands::erase_apply,
            commands::bench_doc,
            commands::doctor_report,
            commands::update_check,
            commands::update_apply
        ])
        .run(tauri::generate_context!())
        .expect("tauri run");
}
