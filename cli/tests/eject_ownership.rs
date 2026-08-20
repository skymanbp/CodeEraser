//! `ce eject` removes ce's own state and NOTHING ELSE.
//!
//! CLAUDE_PLUGIN_DATA can be shared, and the ownership test was
//! `starts_with("ce-")` — which also claimed a neighbour's `ce-cache`
//! and removed it recursively under `--yes` (review 2026-08-19, codex
//! lane). Deleting someone else's file is not a recoverable mistake,
//! so the boundary is asserted through the real CLI rather than
//! assumed: the dry run NAMES every target, which is exactly the list
//! `--yes` would delete.

mod common;

#[test]
fn eject_claims_the_starter_names_and_leaves_the_neighbours() {
    let dir = common::tmp("eject-ownership");
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

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_ce"))
        .args(["eject", "."])
        .current_dir(&dir)
        .env("CLAUDE_PLUGIN_DATA", &data)
        .output()
        .expect("run ce eject");
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
