//! The Query screen's two commands (plan v2.31 step 2) — the SAME
//! documents `ce query --format json` and `ce rules --format json`
//! print: a question over the index's facts answered by the core's
//! query/1 with its derivations under `why`, and the project's rules
//! file judged. Rendering is the webview's; the answers, the proofs
//! and the errors are the core's, and a core without the family
//! shows the document's named degraded posture, never a verdict of
//! the screen's own. Housed beside commands.rs at its size line.

use crate::commands::task;
use serde_json::Value;

#[tauri::command]
pub async fn query_report(
    win: tauri::Window,
    root: String,
    program: String,
    why: bool,
) -> Result<Value, String> {
    task(win, "query", root, move |r, c| {
        codeeraser::faces::query(r, c, &program, why, None)
    })
    .await
}

/// The clone-merge suggestions (plan v2.31 step 7) — the SAME document
/// `ce merge --format json` prints, rendered by the reports hub's own
/// merge card (hub_merge.js).
#[tauri::command]
pub async fn merge_report(win: tauri::Window, root: String) -> Result<Value, String> {
    task(win, "merge", root, codeeraser::faces::merge).await
}

#[tauri::command]
pub async fn rules_report(win: tauri::Window, root: String, why: bool) -> Result<Value, String> {
    task(win, "rules", root, move |r, c| {
        codeeraser::faces::rules(r, c, None, why)
    })
    .await
}
