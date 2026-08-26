//! `ce probe --hook` end-to-end: real binary, stdin envelope per the
//! captured contract, decisions per ce.toml [guard] mode. This is the
//! M3 acceptance shape: a T1/T2 re-write of indexed content gets
//! intercepted (deny mode), warned (warn), or silently logged
//! (observe default) — and the observe feed always grows.

use std::path::Path;

use crate::common;
use crate::common::{rust_fn, tmp};

/// Project with a.rs indexed; guard mode written to ce.toml.
fn seed_project(dir: &Path, mode: &str) {
    common::seed_sources(dir, mode);
    common::build_index(dir);
}

use crate::common::pretooluse_envelope as envelope;

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
    common::silent_hook_observe(dir, &["probe", "--hook"], &env, "probe")
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
    // no daemon survives a hook run (P13), so the probe below
    // respawns against the corrupted db by construction
    common::corrupt_index(&dir);
    let line = silent_probe_observe(&dir, &rust_fn(1));
    assert_eq!(line["degraded"], true, "stamped, not silent");
}

/// §4.2 step 3 (1.0, M7-P2): with NO [guard] mode in ce.toml — here,
/// no ce.toml at all — the promoted rule classes default to DENY
/// (config::PROMOTED_DEFAULT; the FPR ledger is in the CHANGELOG).
#[test]
fn default_tier_denies_t1_rewrite() {
    let dir = tmp("guard-default-deny");
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    common::build_index(&dir);
    common::expect_decision(&dir, &envelope(&dir, "Write", &rust_fn(1)), "deny");
}

/// §4.2 step 3, second promoted class: a Write that leaves the file
/// past the 750-line hard budget denies by default — daemon-free
/// arithmetic, so not even an index is needed.
#[test]
fn budget_breach_denies_by_default() {
    let dir = tmp("guard-budget");
    let big = "// filler\n".repeat(751);
    let reason = common::expect_decision(&dir, &envelope(&dir, "Write", &big), "deny");
    assert!(reason.contains("751 lines"), "exact count named: {reason}");
}

/// plan v2.6 §A observe leg: a 400-line write sits inside the
/// graded zone (soft fallback 300, hard 750 → 222‰) — the hook
/// emits NOTHING (no enforcement below H) but the feed gains the
/// 0.5.0 `zone` line with soft/hard/position; a small write logs no
/// zone line at all.
#[test]
fn a_zone_write_logs_position_and_emits_nothing() {
    let dir = tmp("guard-zone");
    let mid = "// filler\n".repeat(400);
    let out = run_hook(&dir, &envelope(&dir, "Write", &mid));
    assert!(out.trim().is_empty(), "sub-H writes emit nothing: {out}");
    let feed = std::fs::read_to_string(dir.join(".ce/observe.ndjson")).expect("feed");
    let last: serde_json::Value =
        serde_json::from_str(feed.lines().last().expect("lines")).expect("json");
    assert_eq!(last["event"], "zone", "{last}");
    assert_eq!(last["soft"], 300, "fallback = file_lines_warn");
    assert_eq!(last["hard"], 750);
    assert_eq!(last["zone_permille"], 222, "(400-300)*1000/450");
    let small = run_hook(&dir, &envelope(&dir, "Write", "// one line\n"));
    assert!(small.trim().is_empty());
    let feed2 = std::fs::read_to_string(dir.join(".ce/observe.ndjson")).expect("feed");
    let last2: serde_json::Value =
        serde_json::from_str(feed2.lines().last().expect("lines")).expect("json");
    assert_ne!(last2["event"], "zone", "a comfortable write logs no zone");
}

/// v2.7 ①: with ce.toml's `[guard] zone_tiers` armed, the SAME zone
/// positions map to tiers — 666‰ warns (25–75%), 888‰ asks (>75%),
/// 222‰ still emits nothing — and every zone feed line records the
/// mapped tier. The DEFAULT stays the previous test's feed-only
/// behaviour; an armed warn also never rides the promoted classes'
/// deny (the strongest-fired-tier rule).
#[test]
fn an_armed_zone_maps_position_to_tier() {
    let dir = tmp("guard-zone-armed");
    std::fs::write(dir.join("ce.toml"), "[guard]\nzone_tiers = true\n").expect("ce.toml");
    let warn = common::expect_decision(
        &dir,
        &envelope(&dir, "Write", &"// filler\n".repeat(600)),
        "allow",
    );
    assert!(warn.contains("666‰"), "position named: {warn}");
    let ask = common::expect_decision(
        &dir,
        &envelope(&dir, "Write", &"// filler\n".repeat(700)),
        "ask",
    );
    assert!(
        ask.contains("split-candidates"),
        "points at the advisory: {ask}"
    );
    let low = run_hook(&dir, &envelope(&dir, "Write", &"// filler\n".repeat(400)));
    assert!(low.trim().is_empty(), "<25% stays observe: {low}");
    let feed = std::fs::read_to_string(dir.join(".ce/observe.ndjson")).expect("feed");
    let tiers: Vec<String> = feed
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter(|v| v["event"] == "zone")
        .map(|v| v["zone_tier"].as_str().unwrap_or("?").to_string())
        .collect();
    assert_eq!(
        tiers,
        ["warn", "ask", "observe"],
        "the feed records the mapped tier"
    );
}

/// The budget rule shares the scanner's exclusion model, including
/// directory-only patterns (attack review F10): a ce.toml glob, a
/// ce.toml directory entry, and a root .gitignore directory entry
/// all cover the files inside them — the old direct-path probe
/// matched none of the directory forms.
#[test]
fn budget_respects_the_exclusion_model() {
    let dir = tmp("guard-budget-excl");
    std::fs::write(dir.join("ce.toml"), "exclude = [\"gen/**\", \"dir/\"]\n").expect("ce.toml");
    std::fs::write(dir.join(".gitignore"), "dropped/\n").expect(".gitignore");
    let big = "// filler\n".repeat(751);
    for rel in ["gen/big.rs", "dir/big.rs", "dropped/big.rs"] {
        std::fs::create_dir_all(dir.join(rel).parent().expect("parent")).expect("mkdir");
        let env = common::pretooluse_envelope_at(&dir, rel, "Write", &big);
        let out = run_hook(&dir, &env);
        assert!(out.trim().is_empty(), "{rel} is excluded: {out}");
    }
}

/// The rulepack's hook half (plan v2.13 ① P4, zero wire): the hard
/// budget a write is measured against is the FILE'S class line. The
/// same 800-line write passes inside a class whose hard line sits at
/// 900 and denies outside it on the global 750; 901 lines deny inside
/// too, and the reason names the class's own line.
#[test]
fn budget_honors_the_files_class_line() {
    let dir = tmp("guard-budget-class");
    std::fs::write(
        dir.join("ce.toml"),
        "[[rules.class]]\nname = \"gen\"\nglobs = [\"gen/**\"]\n[rules.class.knobs]\nfile_lines_fail = 900\n",
    )
    .expect("ce.toml");
    for rel in ["gen", "src"] {
        std::fs::create_dir_all(dir.join(rel)).expect("mkdir");
    }
    let at = |rel: &str, n: usize| {
        common::pretooluse_envelope_at(&dir, rel, "Write", &"// filler\n".repeat(n))
    };
    let out = run_hook(&dir, &at("gen/big.rs", 800));
    assert!(
        out.trim().is_empty(),
        "800 lines inside the 900-line class pass: {out}"
    );
    let reason = common::expect_decision(&dir, &at("src/big.rs", 800), "deny");
    assert!(
        reason.contains("of 750"),
        "the global line outside the class: {reason}"
    );
    let reason = common::expect_decision(&dir, &at("gen/huge.rs", 901), "deny");
    assert!(
        reason.contains("of 900"),
        "the class's own line inside it: {reason}"
    );
}

/// b.rs Edit envelope with a chosen old_string (new_string fixed at
/// four lines) — shared by the applied-edit and semantics tests.
fn edit_envelope(dir: &Path, old: &str) -> String {
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
}

/// Edit counting uses Edit's OWN apply semantics (attack review
/// F11), one row per shape: a unique marker fires with the exact
/// resulting count; an absent old_string stays silent (that Edit
/// fails on its own); an ambiguous one stays silent (the real tool
/// rejects non-unique matches — judging that never-landing write
/// would be a spurious ask); CRLF on disk is normalized, so the same
/// edit is judged, not evaded.
#[test]
fn budget_counts_the_applied_edit() {
    let dir = tmp("guard-budget-edit");
    let lf = "// line\n".repeat(748);
    let crlf = "// line\r\n".repeat(748);
    let cases = [
        (format!("{lf}marker\n"), "marker\n", Some("752 lines")),
        (format!("{lf}marker\n"), "absent-string\n", None),
        (format!("{lf}marker\nmarker\n"), "marker\n", None),
        (format!("{crlf}marker\r\n"), "marker\n", Some("752 lines")),
    ];
    for (content, old, want) in cases {
        std::fs::write(dir.join("b.rs"), &content).expect("b.rs");
        let env = edit_envelope(&dir, old);
        match want {
            Some(count) => {
                let reason = common::expect_decision(&dir, &env, "deny");
                assert!(reason.contains(count), "{old:?}: {reason}");
            }
            None => {
                let out = run_hook(&dir, &env);
                assert!(out.trim().is_empty(), "{old:?} passes: {out}");
            }
        }
    }
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
    assert!(out2.trim().is_empty(), "foreign tool ignored: {out2}");
}
