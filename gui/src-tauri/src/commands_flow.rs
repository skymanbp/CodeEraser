//! The flow family's hub command (plan v2.31 step 5) — the SAME
//! document `ce flow --format json` prints: the dead code inside every
//! function of the tree, judged by the core's flow/1 and placed through
//! the lowering's legend; `kinds` narrows the listed findings as
//! `--kind` does (absent = every kind: the hub filters on its own
//! chips and asks for the whole document). Rendering is the webview's
//! (hub_flow.js), and a core without the family shows the document's
//! named degraded reason.

use crate::commands::task;
use serde_json::{json, Value};

#[tauri::command]
pub async fn flow_report(
    win: tauri::Window,
    root: String,
    kinds: Option<Vec<String>>,
) -> Result<Value, String> {
    let kinds = kinds.unwrap_or_default();
    task(win, "flow", root, move |r, c| {
        codeeraser::faces::flow(r, c, &kinds)
    })
    .await
}

/// The flow kinds as the core's catalogue lists them (plan v2.32 step
/// 5, R8): each kind's name and its label in both languages, read off
/// the definition package the task loads — the hub's chips and kind
/// column carry no label map of their own.
#[tauri::command]
pub async fn flow_kinds(win: tauri::Window, root: String) -> Result<Value, String> {
    task(win, "flow_kinds", root, |_, _| {
        let rows = codeeraser::flow_report::kinds().iter();
        let labelled = rows.map(|k| json!({"name": k.name, "en": k.en, "zh": k.zh}));
        Ok(Value::Array(labelled.collect()))
    })
    .await
}
