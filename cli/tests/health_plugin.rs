//! SessionStart health hook e2e + plugin manifest sanity: the shipped
//! hooks.json must parse, wire only existing `ce` subcommands, and
//! the health line must report guard mode, index size, and a warm
//! daemon after its own warm-up ping.

use std::path::{Path, PathBuf};

mod common;
use common::tmp;

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
    common::build_index(&dir);
    let envelope = serde_json::json!({
        "session_id": "t", "transcript_path": "t",
        "cwd": dir.display().to_string().replace('\\', "/"),
        "hook_event_name": "SessionStart"
    })
    .to_string();
    let out = common::run_hook(&dir, &["health", "--hook"], &envelope);
    // shut the warmed daemon down before asserting on the line
    common::shutdown_daemon(&dir);
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("json");
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
