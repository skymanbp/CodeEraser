//! PreToolUse cheap gate (M3, ADR-004). Input: the hook envelope on
//! stdin (empirically captured contract, contracts/fixtures/
//! hook-payloads). Output: the permissionDecision JSON proven by the
//! locally installed cc-enforcer hooks on this exact Claude Code
//! build. FAIL-OPEN: any internal failure allows the edit — a guard
//! must never brick editing; degraded runs land in the observe log.
//! Every probed event is appended to <root>/.ce/observe.ndjson in ALL
//! modes — the untainted M4 evaluation feed (plan D2-1). The PostToolUse
//! leg (settle.rs, `ce settle --hook`) closes the one record this hook
//! cannot write: what the person did with an `ask`.

mod budget;
mod probe;
mod say;
mod settle;
mod tombstone;
pub mod zone;

use crate::config::Config;
use envelope::Envelope;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod envelope;

/// Entry point for `ce probe --hook`. Never fails outward.
pub fn run_hook() -> ExitCode {
    let Some((env, root)) = write_event("PreToolUse") else {
        return ExitCode::SUCCESS;
    };
    decide(&root, &env)
}

/// The write hooks' one gate, read by the PreToolUse and the
/// PostToolUse leg alike: the (event, cwd, root, anchor) policy is the
/// throat's (hookio::gated_envelope — batch-8: the anchor rule was a
/// class drifting one copy per hook), the tool-name filter is this
/// family's own, and the project that owns the TARGET judges it
/// (root::judging_root): a nested project with a gate of its own, its
/// own config and index; one without is nobody's here, and the hook
/// stays inert. None = not this family's event.
fn write_event(event: &str) -> Option<(Envelope, PathBuf)> {
    let (env, root) =
        crate::hookio::gated_envelope(event, |e: &Envelope| (&e.hook_event_name, &e.cwd))?;
    if !matches!(env.tool_name.as_str(), "Write" | "Edit") {
        return None;
    }
    let root = crate::root::judging_root(&root, Path::new(&env.tool_input.file_path))?;
    Some((env, root))
}

/// Entry point for `ce settle --hook`, the PostToolUse leg: the record
/// that a write this hook left to the person at `ask` went through
/// (settle.rs). Never fails outward, never speaks.
pub fn run_settle_hook() -> ExitCode {
    settle::run_hook()
}

/// Both PreToolUse rule classes, one decision: T1/T2 duplicate write
/// (daemon probe) and hard-budget breach (local arithmetic). An
/// unreadable ce.toml downgrades everything to observe (fail-open);
/// an absent one resolves to the §4.2 route defaults via tier().
/// Every feed line of the event waits for the decision (0.11.0): the
/// probe line carries the tier the hook decided at, the size classes'
/// lines follow it, the tombstone line carries `applied`.
fn decide(root: &Path, env: &Envelope) -> ExitCode {
    let loaded = Config::load(root);
    let broken = loaded.as_ref().err().cloned();
    // ONE renderer for the tier, the same every other surface uses
    // (audit, health). The local map_or_else stamped a bare "observe"
    // into the feed when ce.toml would not parse — byte-identical to
    // a deliberate observe, which is exactly the drift tier_of exists
    // to prevent, on the one surface that collects the FPR record.
    let mode = crate::config::tier_of(&loaded, crate::config::PROMOTED_DEFAULT);
    // the budgets this hook judges with: the declared config, or the
    // shipped ones while it drifts from the fenced baseline (O33)
    let cfg = loaded.ok().map(|c| budget::fenced(root, c));
    let file_path = &env.tool_input.file_path;
    let probed = probed(root, env);
    // B4 suppression consults the feed BEFORE this event lands in it;
    // it shapes the warn INJECTION only — deny/ask are enforcement,
    // not context bloat, and repeat every time they hold
    let enforced = matches!(mode.as_str(), "deny" | "ask");
    let seen =
        |rule| !enforced && crate::hookio::already_warned(root, &env.session_id, rule, file_path);
    let (dup_seen, budget_seen) = (seen("probe"), seen("budget"));
    let mut reasons = Vec::new();
    if let Some(ms) = &probed.matches
        && !ms.is_empty()
        && !dup_seen
    {
        reasons.push(probe::reason(file_path, ms));
    }
    let sized = budget::size_classes(root, env, cfg.as_ref(), &mode, budget_seen);
    reasons.extend(sized.budget);
    // the third class speaks at its own tier (`[tombstone] tier`); its
    // feed line waits for the decision and records it as `applied`
    let (c, fence) = cfg.as_ref().map_or((None, None), |(c, f)| (Some(c), *f));
    let tomb: Option<tombstone::Pending> = tombstone::observe(root, env, c, fence);
    let spoken = tomb.as_ref().and_then(|p| p.speak.clone());
    let decided = emit_reasons(
        &mode,
        reasons,
        sized.zone.into_iter().chain(spoken).collect(),
        &broken,
    );
    observe_log(root, env, &probed, &mode, decided);
    for line in sized.lines {
        feed(root, env, line);
    }
    tombstone::record(root, env, tomb, decided);
    ExitCode::SUCCESS
}

/// Fired reasons → one decision line, at the STRONGEST tier among
/// the rules that fired: the two promoted classes carry the class
/// mode, the zone rule (v2.7 ①, opt-in) its own mapped tier and the
/// tombstone class its own declared one (`tiered`) — a zone warn
/// never rides a deny-class escalator, nor the reverse.
/// A BROKEN ce.toml still fails open (a typo must never brick an
/// edit) but not SILENT: the decision surfaces as a visible warn
/// naming the config error (review C2). Split from decide() at the
/// E01 line. Returns the tier it decided at (`observe` when nothing
/// was emitted) — the tombstone leg records whether its write went
/// through.
fn emit_reasons<'a>(
    mode: &'a str,
    class_reasons: Vec<String>,
    tiered: Vec<(&'static str, String)>,
    broken: &Option<String>,
) -> &'a str {
    let rank = |t: &str| {
        crate::config::TIERS
            .iter()
            .position(|x| *x == t)
            .unwrap_or(0)
    };
    let mut tier = if class_reasons.is_empty() {
        "observe"
    } else {
        mode
    };
    let mut reasons = class_reasons;
    for (t, why) in tiered {
        if rank(t) > rank(tier) {
            tier = t;
        }
        reasons.push(why);
    }
    // a broken ce.toml is a visible degradation (A9f) even when no
    // rule fired — the early return used to swallow the notice
    // exactly when it was the ONLY thing to say (batch-7 defect
    // sweep)
    if reasons.is_empty() && broken.is_none() {
        return "observe";
    }
    if let Some(e) = broken {
        reasons.push(say::config_unreadable(e));
        emit_decision("warn", &reasons.join(" "));
        return "warn";
    }
    emit_decision(tier, &reasons.join(" "));
    tier
}

/// §4.4 B4: every injected reason rides the warn token budget (the
/// clip marker points at the observe feed, where the full record
/// already lives). Applied at the one emission throat.
fn clipped(reason: &str) -> String {
    crate::hookio::clip(reason, crate::hookio::WARN_BUDGET_TOKENS)
}

// The hard-budget rule class lives in budget.rs (split at the
// 300-line dogfood wall), where scope is judged BEFORE any read; the
// duplicate probe and its novel filter in probe.rs.

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

/// The duplicate probe's answer and its cost — one measurement, one
/// record (the `probe` line every probed event leaves).
struct Probed {
    matches: Option<Vec<serde_json::Value>>,
    elapsed_ms: u128,
}

fn probed(root: &Path, env: &Envelope) -> Probed {
    let started = std::time::Instant::now();
    let matches = probe::novel_matches(root, env);
    Probed {
        matches,
        elapsed_ms: started.elapsed().as_millis(),
    }
}

/// One NDJSON line per probed event, all modes (M4 evaluation feed),
/// written once the event is decided so that it carries `decision` —
/// the tier the hook decided the whole event at (0.11.0), which the
/// PostToolUse leg reads to know an `ask` when the tool then runs.
fn observe_log(root: &Path, env: &Envelope, p: &Probed, mode: &str, decision: &str) {
    feed(
        root,
        env,
        serde_json::json!({
            "event": "probe",
            "file": env.tool_input.file_path,
            "mode": mode,
            "degraded": p.matches.is_none(),
            "matches": p.matches.as_deref().map(<[serde_json::Value]>::len).unwrap_or(0),
            "elapsed_ms": p.elapsed_ms,
            "decision": decision,
        }),
    );
}

/// Every line this hook and its PostToolUse leg write, stamped with
/// the tool call's identity when the envelope carries one (0.11.0): a
/// `settled` line names the same id, so a reader joins a decision left
/// to the person with what the person did (settle.rs, the tombstone
/// leg's session union).
fn feed(root: &Path, env: &Envelope, mut line: serde_json::Value) {
    if !env.tool_use_id.is_empty() {
        line["tool_use_id"] = serde_json::json!(env.tool_use_id);
    }
    crate::hookio::observe_append(root, Some(&env.session_id), line);
}
