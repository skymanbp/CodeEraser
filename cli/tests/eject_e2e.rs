//! `ce eject` e2e (M7-P2, plan §5.9-4): dry run names every target
//! and removes nothing; --yes shuts the daemon down and leaves zero
//! residue while user files stay; the CLAUDE_PLUGIN_DATA sweep takes
//! only the starter's own `ce-*` artifacts — and NOTHING ELSE: the
//! shared dir can hold neighbours, and the old `starts_with("ce-")`
//! ownership test claimed a neighbour's `ce-cache` for recursive
//! deletion under --yes (review 2026-08-19, codex lane). The
//! ownership boundary battery was eject_ownership.rs until the
//! v0.5.0 test consolidation folded it in here.

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

/// One `ce eject` run with CLAUDE_PLUGIN_DATA bound — the sweep and
/// ownership tests' shared act stanza.
fn eject_with_data(
    dir: &std::path::Path,
    data: &std::path::Path,
    args: &[&str],
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ce"))
        .current_dir(dir)
        .env("CLAUDE_PLUGIN_DATA", data)
        .args(args)
        .output()
        .expect("run ce eject")
}

#[test]
fn plugin_data_sweep_takes_only_ce_artifacts() {
    let dir = seeded("eject-pin");
    let data = tmp("eject-pin-data");
    common::write_all(
        &data,
        &[("ce-0.1.0-x86_64-windows.exe", "x"), ("unrelated.txt", "x")],
    );
    let out = eject_with_data(&dir, &data, &["eject", ".", "--yes"]);
    assert!(out.status.success());
    assert!(
        !data.join("ce-0.1.0-x86_64-windows.exe").exists(),
        "pin swept"
    );
    assert!(data.join("unrelated.txt").exists(), "only ce-* swept");
}

/// Deleting someone else's file is not a recoverable mistake, so the
/// boundary is asserted through the real CLI rather than assumed:
/// the dry run NAMES every target, which is exactly the list --yes
/// would delete.
#[test]
fn eject_claims_the_starter_names_and_leaves_the_neighbours() {
    let dir = tmp("eject-ownership");
    let data = dir.join("plugindata");
    std::fs::create_dir_all(&data).expect("mkdir");

    let ours = ["ce-core.exe", "ce-core", "ce-0.4.0-x86_64-windows.exe"];
    let theirs = ["ce-cache", "ce-notcodeeraser", "ce", "cecore", "other"];
    for name in ours.iter().chain(&theirs) {
        std::fs::write(data.join(name), "x").expect("seed");
    }
    // a DIRECTORY whose name we would otherwise claim: the starter
    // only ever places files, and a recursive delete of a directory
    // is the worst version of this mistake
    std::fs::create_dir_all(data.join("ce-0.9.9-somedir")).expect("mkdir");

    let out = eject_with_data(&dir, &data, &["eject", "."]);
    let listed = String::from_utf8_lossy(&out.stdout).to_string();

    // the file NAMES the dry run claims, compared exactly: `ce` and
    // `cecore` are substrings of our own names, so a `contains` test
    // would pass while the wrong file was queued for deletion
    let mut claimed: Vec<String> = listed
        .lines()
        .filter_map(|l| l.strip_prefix("would remove: "))
        .filter_map(|p| p.rsplit(['/', '\\']).next())
        .map(str::to_string)
        .collect();
    claimed.sort();
    let mut want: Vec<String> = ours.iter().map(|s| (*s).to_string()).collect();
    want.sort();
    assert_eq!(
        claimed, want,
        "eject claims exactly the starter's own artifacts — no neighbour, \
         no directory (theirs: {theirs:?})"
    );
}
