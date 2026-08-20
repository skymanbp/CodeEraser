//! PreToolUse cheap gate (M3, ADR-004). Input: the hook envelope on
//! stdin (empirically captured contract, contracts/fixtures/
//! hook-payloads). Output: the permissionDecision JSON proven by the
//! locally installed cc-enforcer hooks on this exact Claude Code
//! build. FAIL-OPEN: any internal failure allows the edit — a guard
//! must never brick editing; degraded runs land in the observe log.
//! Every probed event is appended to <root>/.ce/observe.ndjson in ALL
//! modes — the untainted M4 evaluation feed (plan D2-1).

mod budget;

use crate::config::Config;
use crate::daemon::client;
use crate::daemon::proto::{Request, Response};
use serde::Deserialize;
use std::path::Path;
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
    /// observe feed (schema: hookio::OBSERVE_SCHEMA) because the M4
    /// evaluation set is partitioned BY SESSION — both the D2-2 count
    /// and the D2-1 purity rule are unanswerable without it.
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
    /// Edit-only (captured contract): what `new_string` replaces, and
    /// whether every occurrence is replaced — enough to apply the
    /// edit in memory for an exact post-write line count.
    #[serde(default)]
    old_string: String,
    #[serde(default)]
    replace_all: bool,
}

/// Entry point for `ce probe --hook`. Never fails outward.
pub fn run_hook() -> ExitCode {
    let Some(env) = crate::hookio::read_envelope::<Envelope>() else {
        return ExitCode::SUCCESS;
    };
    if env.hook_event_name != "PreToolUse" || !matches!(env.tool_name.as_str(), "Write" | "Edit") {
        return ExitCode::SUCCESS;
    }
    let root = crate::hookio::project_root(&env.cwd);
    decide(&root, &env)
}

/// Both PreToolUse rule classes, one decision: T1/T2 duplicate write
/// (daemon probe) and hard-budget breach (local arithmetic). An
/// unreadable ce.toml downgrades everything to observe (fail-open);
/// an absent one resolves to the §4.2 route defaults via tier().
fn decide(root: &Path, env: &Envelope) -> ExitCode {
    let loaded = Config::load(root);
    let broken = loaded.as_ref().err().cloned();
    let cfg = loaded.ok();
    let mode = cfg.as_ref().map_or_else(
        || "observe".to_string(),
        |c| c.guard.tier(crate::config::PROMOTED_DEFAULT),
    );
    let file_path = &env.tool_input.file_path;
    let content = if env.tool_name == "Write" {
        &env.tool_input.content
    } else {
        &env.tool_input.new_string
    };
    let started = std::time::Instant::now();
    let matches = probe_matches(root, file_path, content);
    // B4 suppression consults the feed BEFORE this event lands in it;
    // it shapes the warn INJECTION only — deny/ask are enforcement,
    // not context bloat, and repeat every time they hold
    let enforced = matches!(mode.as_str(), "deny" | "ask");
    let seen =
        |rule| !enforced && crate::hookio::already_warned(root, &env.session_id, rule, file_path);
    let (dup_seen, budget_seen) = (seen("probe"), seen("budget"));
    observe_log(
        root,
        ProbeEvent {
            file: file_path,
            mode: &mode,
            session: &env.session_id,
            matches: &matches,
            elapsed_ms: started.elapsed().as_millis(),
        },
    );
    let mut reasons = Vec::new();
    if let Some(ms) = &matches
        && !ms.is_empty()
        && !dup_seen
    {
        reasons.push(reason(file_path, ms));
    }
    let sized = cfg.as_ref().and_then(|c| budget::sized_write(root, c, env));
    if let (Some(c), Some(lines)) = (cfg.as_ref(), sized) {
        if let Some(why) = budget::budget_breach(c, env, lines) {
            budget::budget_log(root, env, &mode, lines);
            if !budget_seen {
                reasons.push(why);
            }
        } else {
            // sub-H writes: the v2.6 zone observer (feed-only)
            budget::zone_log(root, c, env, &mode, lines);
        }
    }
    emit_reasons(&mode, reasons, &broken);
    ExitCode::SUCCESS
}

/// Fired reasons → one decision line. A BROKEN ce.toml still fails
/// open (a typo must never brick an edit) but no longer fails
/// SILENT: when a rule fires anyway, the decision surfaces as a
/// visible warn naming the config error instead of vanishing into
/// observe (review C2: deny→observe with zero operator signal; the
/// health line carries the same error at SessionStart). Split from
/// decide() at the E01 line.
fn emit_reasons(mode: &str, mut reasons: Vec<String>, broken: &Option<String>) {
    if reasons.is_empty() {
        return;
    }
    if let Some(e) = broken {
        reasons.push(format!(
            "(ce.toml unreadable, guard degraded to observe: {e})"
        ));
        emit_decision("warn", &reasons.join(" "));
    } else {
        emit_decision(mode, &reasons.join(" "));
    }
}

/// §4.4 B4: every injected reason rides the warn token budget (the
/// clip marker points at the observe feed, where the full record
/// already lives). Applied at the one emission throat.
fn clipped(reason: &str) -> String {
    crate::hookio::clip(reason, crate::hookio::WARN_BUDGET_TOKENS)
}

// The hard-budget rule class lives in budget.rs (split at the
// 300-line dogfood wall), where scope is judged BEFORE any read.

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

/// Decision JSON on stdout — the exact shape proven by cc-enforcer's
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
            "permissionDecisionReason": clipped(reason),
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
