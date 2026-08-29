//! `ce update`'s console body — the CLI face of the update DOCUMENT
//! (codeeraser::update): the check is measured once and rendered
//! here; `--yes` hands the same document to the apply leg. Its own
//! file because main_cmds.rs sits at the repo's soft line.

use crate::main_cmds::{OutFormat, fail, json};
use codeeraser::i18n::line;
use codeeraser::update;
use std::process::ExitCode;

pub fn update_cmd(yes: bool, installer: bool, format: OutFormat) -> ExitCode {
    let doc = update::document();
    if !yes {
        return report(&doc, json(format));
    }
    match update::apply::run(&doc, installer) {
        Ok(done) => {
            if json(format) {
                println!("{done}");
            } else {
                for l in applied_lines(&done) {
                    println!("{l}");
                }
            }
            ExitCode::SUCCESS
        }
        Err(err) => fail("update", err),
    }
}

/// The exit code IS the verdict (0 current, 1 available, 2 unknown)
/// on both formats — a JSON face that always exited 0 would be the
/// one check a script could not gate on (the doctor precedent).
fn report(doc: &serde_json::Value, as_json: bool) -> ExitCode {
    if as_json {
        println!("{doc}");
    } else {
        for l in update::console(doc) {
            println!("{l}");
        }
    }
    ExitCode::from(doc["verdict"].as_u64().unwrap_or(2) as u8)
}

fn applied_lines(done: &serde_json::Value) -> Vec<String> {
    let placed = done["placed"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|p| p.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let mut out = vec![line(
        "updated to {}: placed {}",
        "已更新到 {}：已放置 {}",
        &[&done["version"].as_str().unwrap_or("?"), &placed],
    )];
    if let Some(p) = done["installer"].as_str() {
        out.push(line(
            "installer saved (verified): {} — run it to update the GUI app",
            "安装包已保存（已校验）：{} — 运行它以更新 GUI 应用",
            &[&p],
        ));
    }
    out
}
