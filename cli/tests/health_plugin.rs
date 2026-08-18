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
    // no ce.toml mode here: the line reports the §4.2 step-2 route
    // default for the promoted PreToolUse classes
    assert!(ctx.contains("guard: ask"), "mode in line: {ctx}");
    assert!(ctx.contains("1 files"), "index size in line: {ctx}");
    assert!(ctx.contains("warm ("), "daemon warmed: {ctx}");
}

/// `ce doctor` (attack-review coverage gap): reports the pointed-at
/// project's status WITHOUT starting a daemon, and counts degraded
/// observe entries. Exit is non-zero here (no ce-core on PATH) — the
/// project lines must print regardless.
#[test]
fn doctor_reports_project_without_spawning_daemon() {
    let dir = tmp("doctor-e2e");
    std::fs::write(dir.join("a.rs"), "fn seed(a: i64) -> i64 { a * 2 + 1 }\n").expect("a.rs");
    common::build_index(&dir);
    // one degraded observe entry to count (shape matches the feed)
    std::fs::write(
        dir.join(".ce/observe.ndjson"),
        "{\"ts_ms\":1,\"event\":\"probe\",\"degraded\":true}\n{\"ts_ms\":2,\"event\":\"probe\",\"degraded\":false}\n",
    )
    .expect("observe seed");
    for _ in 0..2 {
        // twice: if the first run had spawned a daemon, the second
        // would report "warm" and betray the side effect
        let out = common::run_ce(&dir, &["doctor", "--core", "ce-core-missing", "."]);
        assert!(!out.status.success(), "handshake must fail without core");
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("index: 1 files"), "project status: {text}");
        assert!(
            text.contains("not running (lazy-starts on first probe)"),
            "doctor must not spawn the daemon: {text}"
        );
        assert!(
            text.contains("degraded runs (observe feed): 1 of 2 entries"),
            "degraded counter with lifetime frame: {text}"
        );
    }
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
    // M7-P1: hooks enter through the ADR-007 starter (verified pinned
    // copy -> pinned download -> PATH ce); the literals ARE the
    // contract — a drifted wiring is a broken plugin install.
    let known = [
        "sh \"${CLAUDE_PLUGIN_ROOT}/bin/ce.sh\" health --hook",
        "sh \"${CLAUDE_PLUGIN_ROOT}/bin/ce.sh\" probe --hook",
        "sh \"${CLAUDE_PLUGIN_ROOT}/bin/ce.sh\" audit --hook",
    ];
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
    // the wire target and its manifest ship with the plugin
    assert!(plugin.join("bin/ce.sh").is_file(), "starter missing");
    assert!(plugin.join("bin/manifest.env").is_file(), "manifest missing");
    // marketplace manifest parses and points at this plugin
    let market: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(plugin.join(".claude-plugin/marketplace.json"))
            .expect("marketplace.json"),
    )
    .expect("marketplace.json parses");
    assert_eq!(market["plugins"][0]["name"], "codeeraser");
}
