//! SessionStart health hook e2e + plugin manifest sanity: the shipped
//! hooks.json must parse, wire only existing `ce` subcommands, and
//! the health line must report guard mode, index size, and a warm
//! daemon after its own warm-up ping.

use crate::common;
use crate::common::repo_root;
use crate::common::tmp;

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
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("json");
    let ctx = v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("additionalContext");
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "SessionStart");
    // no ce.toml mode here: the line reports the §4.2 step-3 route
    // default for the promoted PreToolUse classes (1.0, M7-P2)
    assert!(ctx.contains("guard: deny"), "mode in line: {ctx}");
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
    let hooks: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(plugin.join("hooks/hooks.json")).expect("hooks.json"),
    )
    .expect("hooks.json parses");
    // M7-P1: hooks enter through the ADR-007 starter (verified pinned
    // copy -> pinned download -> PATH ce); the literals ARE the
    // contract — a drifted wiring is a broken plugin install.
    let known = ["health", "probe", "audit"]
        .map(|sub| format!("sh \"${{CLAUDE_PLUGIN_ROOT}}/bin/ce.sh\" {sub} --hook"));
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
                assert!(known.iter().any(|k| k == cmd), "unknown wiring: {cmd}");
            }
        }
    }
    // the wire target and its manifest ship with the plugin
    for shipped in ["bin/ce.sh", "bin/manifest.env"] {
        assert!(plugin.join(shipped).is_file(), "{shipped} missing");
    }
}

/// Every shipped version mirror equals the crate version — ONE
/// source (release.yml pins the dispatch input == crate; this pins
/// the mirrors, replacing a hardcoded "0.1." prefix the 0.2.0 bump
/// broke). The v0.1.0 era proved the drift class: a 0.1.0
/// plugin.json beside a 0.1.1 crate while binaries reported 0.0.1.
/// Cargo.lock mirrors are already machine-checked by --locked.
#[test]
fn version_mirrors_move_with_the_crate() {
    let crate_version = env!("CARGO_PKG_VERSION");
    let root = repo_root();
    for json_mirror in [
        "plugin/.claude-plugin/plugin.json",
        "gui/src-tauri/tauri.conf.json",
    ] {
        let doc: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(json_mirror)).expect(json_mirror),
        )
        .expect("mirror parses");
        assert_eq!(doc["version"], crate_version, "{json_mirror}");
    }
    let gui_toml =
        std::fs::read_to_string(root.join("gui/src-tauri/Cargo.toml")).expect("gui Cargo.toml");
    let gui_version = gui_toml
        .lines()
        .find_map(|l| l.strip_prefix("version = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .expect("gui version line");
    assert_eq!(gui_version, crate_version, "gui/src-tauri/Cargo.toml");
    // the judgment core rides the same release train (its binary
    // self-reports this via the cabal-generated Paths_ce_core)
    let cabal = std::fs::read_to_string(root.join("core/ce-core.cabal")).expect("ce-core.cabal");
    let core_version = cabal
        .lines()
        .find_map(|l| l.strip_prefix("version:"))
        .map(str::trim)
        .expect("cabal version line");
    assert_eq!(core_version, crate_version, "core/ce-core.cabal");
}

/// The marketplace manifest lives at the REPO root — the only
/// location `/plugin marketplace add owner/repo` reads (official
/// plugin-marketplaces docs, verified 2026-08-18; the old
/// plugin-local copy was undiscoverable by the one-command add) —
/// and points at plugin/; the entry carries NO version so
/// plugin.json stays the one version authority.
#[test]
fn marketplace_manifest_sits_at_repo_root_and_points_home() {
    let market: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join(".claude-plugin/marketplace.json"))
            .expect("marketplace.json at repo root"),
    )
    .expect("marketplace.json parses");
    assert_eq!(market["plugins"][0]["name"], "codeeraser");
    assert_eq!(market["plugins"][0]["source"], "./plugin");
    assert!(
        market["plugins"][0].get("version").is_none(),
        "entry version must stay unset — plugin.json is the authority"
    );
}
