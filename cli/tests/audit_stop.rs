//! `ce audit --hook` end-to-end: real git repo, Stop envelope on
//! stdin. Deny mode blocks when the working tree adds duplication
//! touching changed files; observe stays silent but logs; the
//! stop_hook_active loop guard short-circuits.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn rust_fn(seed: u32) -> String {
    format!(
        "fn work_{seed}(input_{seed}: &[i64], limit_{seed}: i64) -> i64 {{
    let mut total_{seed} = {seed};
    for value_{seed} in input_{seed} {{
        if *value_{seed} > limit_{seed} {{
            total_{seed} += value_{seed} * {seed} + 7;
        }} else {{
            total_{seed} -= value_{seed} / 3;
        }}
    }}
    total_{seed}
}}
"
    )
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {:?}", out);
}

/// Repo with a.rs committed; working tree adds b.rs = T2 clone of a.rs.
fn seed_repo(dir: &Path, mode: &str) {
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("ce.toml"), format!("[guard]\nmode = \"{mode}\"\n")).expect("ce.toml");
    git(dir, &["init", "-q"]);
    git(
        dir,
        &["-c", "user.email=t@t", "-c", "user.name=t", "add", "."],
    );
    git(
        dir,
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "seed",
        ],
    );
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs (uncommitted clone)");
    git(dir, &["add", "b.rs"]); // numstat vs HEAD sees staged new files
}

fn run_audit(dir: &Path, envelope: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ce"))
        .args(["audit", "--hook"])
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn audit");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(envelope.as_bytes())
        .expect("write");
    let out = child.wait_with_output().expect("wait");
    assert!(out.status.success(), "audit must exit 0 (fail-open)");
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn envelope(dir: &Path, stop_hook_active: bool) -> String {
    serde_json::json!({
        "session_id": "t", "transcript_path": "t",
        "cwd": dir.display().to_string().replace('\\', "/"),
        "hook_event_name": "Stop", "stop_hook_active": stop_hook_active
    })
    .to_string()
}

#[test]
fn deny_mode_blocks_on_touched_duplication() {
    let dir = tmp("audit-deny");
    seed_repo(&dir, "deny");
    let out = run_audit(&dir, &envelope(&dir, false));
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("block json");
    assert_eq!(v["decision"], "block");
    let reason = v["reason"].as_str().expect("reason");
    assert!(reason.contains("b.rs"), "reason names the clone: {reason}");
}

#[test]
fn observe_mode_is_silent_but_logs() {
    let dir = tmp("audit-observe");
    seed_repo(&dir, "observe");
    let out = run_audit(&dir, &envelope(&dir, false));
    assert!(out.trim().is_empty(), "observe emits nothing: {out}");
    let log = std::fs::read_to_string(dir.join(".ce/observe.ndjson")).expect("log");
    let line: serde_json::Value =
        serde_json::from_str(log.lines().last().expect("line")).expect("ndjson");
    assert_eq!(line["event"], "stop_audit");
    assert!(line["dup_blocks"].as_u64().expect("n") >= 1);
    assert!(line["net_loc"].as_i64().expect("net") > 0);
}

#[test]
fn loop_guard_and_clean_tree_stay_silent() {
    let dir = tmp("audit-loop");
    seed_repo(&dir, "deny");
    // stop_hook_active: a prior Stop block already fired — pass through
    let out = run_audit(&dir, &envelope(&dir, true));
    assert!(out.trim().is_empty(), "loop guard passes: {out}");
    // clean tree: commit everything, audit stays silent even in deny
    git(
        &dir,
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "b",
        ],
    );
    let out2 = run_audit(&dir, &envelope(&dir, false));
    assert!(out2.trim().is_empty(), "clean tree passes: {out2}");
}
