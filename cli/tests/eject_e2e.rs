//! `ce eject` e2e (M7-P2, plan §5.9-4): dry run names every target
//! and removes nothing; --yes shuts the daemon down and leaves zero
//! residue while user files stay; the CLAUDE_PLUGIN_DATA sweep takes
//! only the starter's own `ce-*` artifacts.

use std::process::Command;

mod common;
use common::tmp;

fn seeded(name: &str) -> std::path::PathBuf {
    let dir = tmp(name);
    std::fs::write(dir.join("a.rs"), common::rust_fn(1)).expect("a.rs");
    common::build_index(&dir);
    std::fs::write(dir.join("ce-baseline.json"), "{}").expect("baseline");
    dir
}

#[test]
fn dry_run_names_targets_and_removes_nothing() {
    let dir = seeded("eject-dry");
    let text = common::run_expect(&dir, &["eject", "."]);
    assert!(text.contains("would remove"), "targets named: {text}");
    assert!(text.contains("dry run"), "mode said out loud: {text}");
    assert!(dir.join(".ce").exists(), "dry run must not delete");
    assert!(dir.join("ce-baseline.json").exists());
}

#[test]
fn yes_removes_everything_even_with_the_daemon_up() {
    let dir = seeded("eject-yes");
    // A REAL daemon (never the lazy-spawning client::request — inside
    // a test harness that respawns the TEST binary, the process-spray
    // flake class common/mod.rs documents) so the shutdown-then-remove
    // ordering is exercised, not just the cold path.
    let child = common::spawn_daemon_ready(&dir);
    common::run_expect(&dir, &["eject", ".", "--yes"]);
    common::wait_exit(child, "ejected daemon");
    assert!(!dir.join(".ce").exists(), "cache removed");
    assert!(!dir.join("ce-baseline.json").exists(), "baseline removed");
    assert!(dir.join("a.rs").exists(), "user files untouched");
}

#[test]
fn plugin_data_sweep_takes_only_ce_artifacts() {
    let dir = seeded("eject-pin");
    let data = tmp("eject-pin-data");
    common::write_all(
        &data,
        &[("ce-0.1.0-x86_64-windows.exe", "x"), ("unrelated.txt", "x")],
    );
    let out = Command::new(env!("CARGO_BIN_EXE_ce"))
        .current_dir(&dir)
        .env("CLAUDE_PLUGIN_DATA", &data)
        .args(["eject", ".", "--yes"])
        .output()
        .expect("run eject");
    assert!(out.status.success());
    assert!(
        !data.join("ce-0.1.0-x86_64-windows.exe").exists(),
        "pin swept"
    );
    assert!(data.join("unrelated.txt").exists(), "only ce-* swept");
}
