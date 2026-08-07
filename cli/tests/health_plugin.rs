//! SessionStart health hook e2e + plugin manifest sanity: the shipped
//! hooks.json must parse, wire only existing `ce` subcommands, and
//! the health line must report guard mode, index size, and a warm
//! daemon after its own warm-up ping.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .to_path_buf()
}

#[test]
fn health_reports_mode_index_and_warm_daemon() {
    let dir = tmp("health-e2e");
    std::fs::write(dir.join("a.rs"), "fn seed(a: i64) -> i64 { a * 2 + 1 }\n").expect("a.rs");
    let out = Command::new(env!("CARGO_BIN_EXE_ce"))
        .args(["dedup", "."])
        .current_dir(&dir)
        .output()
        .expect("seed index");
    assert!(out.status.success());
    let envelope = serde_json::json!({
        "session_id": "t", "transcript_path": "t",
        "cwd": dir.display().to_string().replace('\\', "/"),
        "hook_event_name": "SessionStart"
    })
    .to_string();
    let mut child = Command::new(env!("CARGO_BIN_EXE_ce"))
        .args(["health", "--hook"])
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn health");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(envelope.as_bytes())
        .expect("write");
    let out = child.wait_with_output().expect("wait");
    // shut the warmed daemon down before asserting
    use codeeraser::daemon::{client, proto::Request};
    let _ = client::request(&dir, &Request::Shutdown);
    assert!(out.status.success(), "health must exit 0");
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("json");
    let ctx = v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("additionalContext");
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "SessionStart");
    assert!(ctx.contains("guard: observe"), "mode in line: {ctx}");
    assert!(ctx.contains("1 files"), "index size in line: {ctx}");
    assert!(ctx.contains("warm ("), "daemon warmed: {ctx}");
}

#[test]
fn plugin_manifests_parse_and_wire_real_subcommands() {
    let plugin = repo_root().join("plugin");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(plugin.join(".claude-plugin/plugin.json")).expect("plugin.json"),
    )
    .expect("plugin.json parses");
    assert_eq!(manifest["name"], "codeeraser");
    assert!(
        manifest["version"]
            .as_str()
            .expect("version")
            .starts_with("0.1.")
    );
    let hooks: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(plugin.join("hooks/hooks.json")).expect("hooks.json"),
    )
    .expect("hooks.json parses");
    let known = ["ce health --hook", "ce probe --hook", "ce audit --hook"];
    let events = hooks["hooks"].as_object().expect("events");
    assert_eq!(events.len(), 3, "SessionStart + PreToolUse + Stop");
    for (event, entries) in events {
        assert!(
            ["SessionStart", "PreToolUse", "Stop"].contains(&event.as_str()),
            "unexpected event {event}"
        );
        for entry in entries.as_array().expect("array") {
            for h in entry["hooks"].as_array().expect("hooks") {
                let cmd = h["command"].as_str().expect("command");
                assert!(known.contains(&cmd), "unknown wiring: {cmd}");
            }
        }
    }
    // marketplace manifest parses and points at this plugin
    let market: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(plugin.join(".claude-plugin/marketplace.json"))
            .expect("marketplace.json"),
    )
    .expect("marketplace.json parses");
    assert_eq!(market["plugins"][0]["name"], "codeeraser");
}
