//! PreToolUse cheap gate (M3, ADR-004). Input: the hook envelope on
//! stdin (empirically captured contract, contracts/fixtures/
//! hook-payloads). Output: the permissionDecision JSON proven by the
//! locally installed cc-enforcer hooks on this exact Claude Code
//! build. FAIL-OPEN: any internal failure allows the edit — a guard
//! must never brick editing; degraded runs land in the observe log.
//! Every probed event is appended to <root>/.ce/observe.ndjson in ALL
//! modes — the untainted M4 evaluation feed (plan D2-1).

mod budget;
mod probe;
mod say;
mod tombstone;

use crate::config::Config;
use envelope::Envelope;
use std::path::Path;
use std::process::ExitCode;

mod envelope;

/// Entry point for `ce probe --hook`. Never fails outward. The
/// (event, cwd, root, anchor) policy is the throat's
/// (hookio::gated_envelope — batch-8: the anchor rule was a class
/// drifting one copy per hook); only the tool-name filter is this
/// hook's own.
pub fn run_hook() -> ExitCode {
    let Some((env, root)) =
        crate::hookio::gated_envelope("PreToolUse", |e: &Envelope| (&e.hook_event_name, &e.cwd))
    else {
        return ExitCode::SUCCESS;
    };
    if !matches!(env.tool_name.as_str(), "Write" | "Edit") {
        return ExitCode::SUCCESS;
    }
    // the project that owns the TARGET judges it (root::judging_root):
    // a nested project with a gate of its own, its own config and
    // index; one without is nobody's here, and the hook stays inert
    let Some(root) = crate::root::judging_root(&root, Path::new(&env.tool_input.file_path)) else {
        return ExitCode::SUCCESS;
    };
    decide(&root, &env)
}

/// Both PreToolUse rule classes, one decision: T1/T2 duplicate write
/// (daemon probe) and hard-budget breach (local arithmetic). An
/// unreadable ce.toml downgrades everything to observe (fail-open);
/// an absent one resolves to the §4.2 route defaults via tier().
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
    let started = std::time::Instant::now();
    let matches = probe::novel_matches(root, env);
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
        reasons.push(probe::reason(file_path, ms));
    }
    let mut zone = None;
    let sized = cfg
        .as_ref()
        .and_then(|(c, _)| budget::sized_write(root, c, env));
    if let (Some((c, fence)), Some(lines)) = (cfg.as_ref(), sized) {
        // the lines THIS file is measured against: its class's, or
        // the global table (plan v2.13 ① P4)
        let t = budget::lines_for(root, c, file_path);
        if let Some(why) = budget::budget_breach(&t, env, lines, *fence) {
            budget::budget_log(root, env, &mode, lines);
            if !budget_seen {
                reasons.push(why);
            }
        } else {
            // sub-H writes: the zone observer, plus the v2.7 ①
            // OPT-IN tier map (default stays feed-only)
            zone = budget::zone_assess(root, &budget::ZoneLines::of(&t, c), env, &mode, lines);
        }
    }
    // the third class speaks at its own tier (`[tombstone] tier`); its
    // feed line waits for the decision and records it as `applied`
    let (c, fence) = cfg.as_ref().map_or((None, None), |(c, f)| (Some(c), *f));
    let tomb: Option<tombstone::Pending> = tombstone::observe(root, env, c, fence);
    let spoken = tomb.as_ref().and_then(|p| p.speak.clone());
    let decided = emit_reasons(
        &mode,
        reasons,
        zone.into_iter().chain(spoken).collect(),
        &broken,
    );
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
