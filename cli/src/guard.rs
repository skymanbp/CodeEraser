//! PreToolUse cheap gate (M3, ADR-004). Input: the hook envelope on
//! stdin (empirically captured contract, contracts/fixtures/
//! hook-payloads). Output: the permissionDecision JSON proven by the
//! locally installed cc-enslaver hooks on this exact Claude Code
//! build. FAIL-OPEN: any internal failure allows the edit — a guard
//! must never brick editing; degraded runs land in the observe log.
//! Every probed event is appended to <root>/.ce/observe.ndjson in ALL
//! modes — the untainted M4 evaluation feed (plan D2-1).

use crate::config::Config;
use crate::daemon::client;
use crate::daemon::proto::{Request, Response};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Deserialize)]
struct Envelope {
    #[serde(default)]
    hook_event_name: String,
    #[serde(default)]
    tool_name: String,
    #[serde(default)]
    cwd: String,
    /// Claude Code stamps this on every hook event. Carried into the
    /// observe feed (schema 0.2.0) because the M4 evaluation set is
    /// partitioned BY SESSION — both the D2-2 count and the D2-1
    /// purity rule are unanswerable without it.
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    tool_input: ToolInput,
}

#[derive(Deserialize, Default)]
struct ToolInput {
    #[serde(default)]
    file_path: String,
    /// Write payloads carry `content`; Edit payloads carry
    /// `new_string` (captured contract) — the added text either way.
    #[serde(default)]
    content: String,
    #[serde(default)]
    new_string: String,
}

/// Entry point for `ce probe --hook`. Never fails outward.
pub fn run_hook() -> ExitCode {
    let Some(env) = crate::hookio::read_envelope::<Envelope>() else {
        return ExitCode::SUCCESS;
    };
    if env.hook_event_name != "PreToolUse" || !matches!(env.tool_name.as_str(), "Write" | "Edit") {
        return ExitCode::SUCCESS;
    }
    let content = if env.tool_name == "Write" {
        &env.tool_input.content
    } else {
        &env.tool_input.new_string
    };
    let root = PathBuf::from(&env.cwd);
    decide(&root, &env.tool_input.file_path, content, &env.session_id)
}

fn decide(root: &Path, file_path: &str, content: &str, session: &str) -> ExitCode {
    let mode = Config::load(root)
        .map(|c| c.guard.mode)
        .unwrap_or_else(|_| "observe".into());
    let started = std::time::Instant::now();
    let matches = probe_matches(root, file_path, content);
    observe_log(
        root,
        ProbeEvent {
            file: file_path,
            mode: &mode,
            session,
            matches: &matches,
            elapsed_ms: started.elapsed().as_millis(),
        },
    );
    let Some(matches) = matches else {
        return ExitCode::SUCCESS; // degraded (daemon unreachable): fail open
    };
    if matches.is_empty() {
        return ExitCode::SUCCESS;
    }
    emit_decision(&mode, &reason(file_path, &matches));
    ExitCode::SUCCESS
}

/// None = probe unavailable (degraded); Some(vec) = verified matches.
fn probe_matches(root: &Path, file_path: &str, content: &str) -> Option<Vec<serde_json::Value>> {
    let req = Request::Probe {
        file_path: file_path.to_string(),
        content: content.to_string(),
    };
    match client::request(root, &req) {
        Ok(Response::ProbeReport { matches, .. }) => {
            Some(matches.as_array().cloned().unwrap_or_default())
        }
        _ => None,
    }
}

fn reason(file_path: &str, matches: &[serde_json::Value]) -> String {
    let top: Vec<String> = matches
        .iter()
        .take(3)
        .map(|m| {
            format!(
                "{}:{}-{} ({} tokens)",
                m["file"].as_str().unwrap_or("?"),
                m["start_line"],
                m["end_line"],
                m["tokens"]
            )
        })
        .collect();
    format!(
        "ce: content for {file_path} duplicates {} indexed region(s): {}. \
         Reuse the existing implementation instead of re-writing it.",
        matches.len(),
        top.join("; ")
    )
}

/// Decision JSON on stdout — the exact shape proven by cc-enslaver's
/// working hooks (allow carries the reason as a visible warning).
fn emit_decision(mode: &str, reason: &str) {
    let decision = match mode {
        "deny" => "deny",
        "ask" => "ask",
        "warn" => "allow",
        _ => return, // observe: log only, no output
    };
    let payload = serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": decision,
            "permissionDecisionReason": reason,
        }
    });
    println!("{payload}");
}

/// One probe observe entry. A struct rather than positional
/// parameters: adding session identity would have taken the old
/// signature to six, past the scan's fn-params limit of 5, and these
/// fields are one cohesive record anyway.
struct ProbeEvent<'a> {
    file: &'a str,
    mode: &'a str,
    session: &'a str,
    matches: &'a Option<Vec<serde_json::Value>>,
    elapsed_ms: u128,
}

/// One NDJSON line per probed event, all modes (M4 evaluation feed).
fn observe_log(root: &Path, ev: ProbeEvent) {
    crate::hookio::observe_append(
        root,
        Some(ev.session),
        serde_json::json!({
            "event": "probe",
            "file": ev.file,
            "mode": ev.mode,
            "degraded": ev.matches.is_none(),
            "matches": ev.matches.as_deref().map(<[serde_json::Value]>::len).unwrap_or(0),
            "elapsed_ms": ev.elapsed_ms,
        }),
    );
}
