//! `ce probe --hook` end-to-end: real binary, stdin envelope per the
//! captured contract, decisions per ce.toml [guard] mode. This is the
//! M3 acceptance shape: a T1/T2 re-write of indexed content gets
//! intercepted (deny mode), warned (warn), or silently logged
//! (observe default) — and the observe feed always grows.

use std::path::Path;

mod common;
use common::{rust_fn, shutdown_daemon, tmp};

/// Project with a.rs indexed; guard mode written to ce.toml.
fn seed_project(dir: &Path, mode: &str) {
    common::seed_sources(dir, mode);
    common::build_index(dir);
}

use common::pretooluse_envelope as envelope;

fn run_hook(dir: &Path, envelope: &str) -> String {
    common::run_hook(dir, &["probe", "--hook"], envelope)
}

#[test]
fn deny_mode_intercepts_t1_rewrite() {
    let dir = tmp("guard-deny");
    seed_project(&dir, "deny");
    let reason = common::expect_decision(&dir, &envelope(&dir, "Write", &rust_fn(1)), "deny");
    assert!(reason.contains("a.rs"), "reason names the source: {reason}");
}

#[test]
fn warn_mode_allows_with_reason() {
    let dir = tmp("guard-warn");
    seed_project(&dir, "warn");
    common::expect_decision(&dir, &envelope(&dir, "Edit", &rust_fn(2)), "allow");
}

fn silent_probe_observe(dir: &Path, content: &str) -> serde_json::Value {
    let env = envelope(dir, "Write", content);
    let line = common::silent_hook_observe(dir, &["probe", "--hook"], &env, "probe");
    shutdown_daemon(dir);
    line
}

#[test]
fn observe_mode_is_silent_but_logs() {
    let dir = tmp("guard-observe");
    seed_project(&dir, "observe");
    let line = silent_probe_observe(&dir, &rust_fn(3));
    assert_eq!(line["degraded"], false);
    assert!(line["matches"].as_u64().expect("n") >= 1);
}

/// FAIL-OPEN intake (attack-review coverage gap): garbage stdin must
/// exit 0 with no output for every hook subcommand — the shared
/// hookio::read_envelope path.
#[test]
fn malformed_envelope_fails_open_everywhere() {
    let dir = tmp("guard-garbage");
    for sub in ["probe", "audit", "health"] {
        let out = common::run_hook(&dir, &[sub, "--hook"], "{not json");
        assert!(out.trim().is_empty(), "{sub} must stay silent: {out}");
    }
}

/// A9f guard side (attack-review coverage gap): when the probe
/// cannot deliver a verdict (index corrupt under the daemon), the
/// edit passes even in deny mode (fail-open) and the observe entry
/// stamps degraded=true — mirroring the audit-side degraded test.
#[test]
fn probe_failure_is_stamped_degraded() {
    let dir = tmp("guard-degraded");
    seed_project(&dir, "deny");
    shutdown_daemon(&dir); // next probe respawns against the bad db
    common::corrupt_index(&dir);
    let line = silent_probe_observe(&dir, &rust_fn(1));
    assert_eq!(line["degraded"], true, "stamped, not silent");
}

/// §4.2 step 2 (landed after the M4 FPR gate): with NO [guard] mode
/// in ce.toml — here, no ce.toml at all — the promoted rule classes
/// default to ask. Before step 2 this exact rewrite was silent.
#[test]
fn default_tier_asks_on_t1_rewrite() {
    let dir = tmp("guard-default-ask");
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    common::build_index(&dir);
    common::expect_decision(&dir, &envelope(&dir, "Write", &rust_fn(1)), "ask");
}

/// §4.2 step 2, second promoted class: a Write that leaves the file
/// past the 750-line hard budget asks by default — daemon-free
/// arithmetic, so not even an index is needed.
#[test]
fn budget_breach_asks_by_default() {
    let dir = tmp("guard-budget");
    let big = "// filler\n".repeat(751);
    let reason = common::expect_decision(&dir, &envelope(&dir, "Write", &big), "ask");
    assert!(reason.contains("751 lines"), "exact count named: {reason}");
}

/// The budget rule shares the scanner's exclusion model: the same
/// over-cap write into an excluded path stays silent.
#[test]
fn budget_respects_the_exclusion_model() {
    let dir = tmp("guard-budget-excl");
    std::fs::write(dir.join("ce.toml"), "exclude = [\"gen/**\"]\n").expect("ce.toml");
    std::fs::create_dir_all(dir.join("gen")).expect("gen/");
    let big = "// filler\n".repeat(751);
    let env = common::pretooluse_envelope_at(&dir, "gen/big.rs", "Write", &big);
    let out = run_hook(&dir, &env);
    shutdown_daemon(&dir);
    assert!(out.trim().is_empty(), "excluded path passes: {out}");
}

/// Edit counting uses Edit's own apply semantics: replacing a marker
/// with enough lines to cross the cap fires with the exact resulting
/// count; an old_string the file does not contain stays silent (that
/// Edit fails on its own — no guess, no prompt).
#[test]
fn budget_counts_the_applied_edit() {
    let dir = tmp("guard-budget-edit");
    let body = format!("{}marker\n", "// line\n".repeat(748));
    std::fs::write(dir.join("b.rs"), &body).expect("b.rs");
    let edit = |old: &str| {
        serde_json::json!({
            "session_id": "t", "transcript_path": "t",
            "cwd": dir.display().to_string().replace('\\', "/"),
            "hook_event_name": "PreToolUse", "tool_name": "Edit",
            "tool_input": {
                "file_path": dir.join("b.rs").display().to_string().replace('\\', "/"),
                "old_string": old, "new_string": "// a\n// b\n// c\n// d\n",
                "replace_all": false
            },
            "tool_use_id": "t"
        })
        .to_string()
    };
    let reason = common::expect_decision(&dir, &edit("marker\n"), "ask");
    assert!(reason.contains("752 lines"), "748+4 applied: {reason}");
    let silent = run_hook(&dir, &edit("absent-string\n"));
    shutdown_daemon(&dir);
    assert!(silent.trim().is_empty(), "unmatched Edit passes: {silent}");
}

#[test]
fn clean_content_and_foreign_tools_stay_silent() {
    let dir = tmp("guard-clean");
    seed_project(&dir, "deny");
    // unrelated content: no decision even in deny mode
    let clean = "fn fresh(n: u8) -> u8 { n / 2 }\n";
    let out = run_hook(&dir, &envelope(&dir, "Write", clean));
    assert!(out.trim().is_empty(), "clean content passes: {out}");
    // non-Write/Edit event is ignored outright
    let bash = serde_json::json!({
        "hook_event_name": "PreToolUse", "tool_name": "Bash",
        "cwd": dir.display().to_string(), "tool_input": {"command": "ls"}
    })
    .to_string();
    let out2 = run_hook(&dir, &bash);
    shutdown_daemon(&dir);
    assert!(out2.trim().is_empty(), "foreign tool ignored: {out2}");
}
