//! `ce eject` (plan §5.9-4, M7-P2): the uninstall path. Shuts the
//! project daemon down, then removes the analysis cache (.ce/), the
//! committed baseline, and any pinned starter binaries in
//! CLAUDE_PLUGIN_DATA. Dry-run by default — destructive removal only
//! under --yes, and every target is named either way: a delete that
//! cannot be previewed is a delete nobody trusts. ce.toml stays (it
//! is the user's declaration, not our state — §5.9-4 lists it
//! nowhere).

use crate::daemon::{client, proto::Request};
use crate::i18n::line;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn run(root: &Path, yes: bool) -> ExitCode {
    let targets = targets(root);
    if targets.is_empty() {
        println!(
            "{}",
            crate::i18n::t(
                "eject: nothing to remove (no .ce/, baseline, or pinned binaries)",
                "eject：无可移除项（没有 .ce/、基线或钉扎二进制）",
            )
        );
        return ExitCode::SUCCESS;
    }
    if !yes {
        for t in &targets {
            println!(
                "{}",
                line("would remove: {}", "将移除：{}", &[&t.display()])
            );
        }
        println!(
            "{}",
            line(
                "eject: dry run — pass --yes to remove {} target(s)",
                "eject：试运行 — 传 --yes 以移除 {} 个目标",
                &[&targets.len()],
            )
        );
        return ExitCode::SUCCESS;
    }
    // Never the lazy-spawn path: spawning a daemon in order to shut
    // it down would recreate the state being removed.
    let _ = client::request_if_running(root, &Request::Shutdown);
    remove_all(&targets)
}

/// The destructive phase (split at the 50-line fn gate): remove each
/// target, name every success and failure, and tally the exit.
fn remove_all(targets: &[PathBuf]) -> ExitCode {
    let mut failed = 0;
    for t in targets {
        match remove(t) {
            Ok(()) => println!("{}", line("removed: {}", "已移除：{}", &[&t.display()])),
            Err(e) => {
                failed += 1;
                eprintln!("eject: {} — {e}", t.display());
            }
        }
    }
    if failed > 0 {
        eprintln!(
            "{}",
            line(
                "eject: {} target(s) not removed",
                "eject：{} 个目标未移除",
                &[&failed],
            )
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Everything eject owns, existing entries only: the §5.9-4 list —
/// .ce/ (index + observe feed), ce-baseline.json, pinned `ce-*`
/// starter binaries under CLAUDE_PLUGIN_DATA.
fn targets(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let ce = root.join(".ce");
    if ce.exists() {
        out.push(ce);
    }
    let baseline = root.join(crate::score::baseline::BASELINE_FILE);
    if baseline.exists() {
        out.push(baseline);
    }
    if let Ok(data) = std::env::var("CLAUDE_PLUGIN_DATA")
        && let Ok(entries) = std::fs::read_dir(&data)
    {
        for e in entries.flatten() {
            if e.file_name().to_string_lossy().starts_with("ce-") {
                out.push(e.path());
            }
        }
    }
    out
}

/// Remove one target with a bounded teardown wait: the daemon just
/// received Shutdown and Windows keeps the index file locked until
/// its exiting process closes handles — this is ordering the
/// teardown (the M5 SQLite drop-before-delete lesson), not masking a
/// race: after 10 tries the error surfaces with the path named.
fn remove(path: &Path) -> std::io::Result<()> {
    let mut last = None;
    for _ in 0..10 {
        let r = if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };
        match r {
            Ok(()) => return Ok(()),
            Err(e) => {
                last = Some(e);
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
    Err(last.expect("ten failed attempts recorded an error"))
}
