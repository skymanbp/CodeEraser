//! A hook that cannot measure leaves a trace (ADR-003 A9f: degraded
//! state is surfaced, never silent). Since plan v2.32 step 2 every
//! table a hook measures with is the core's `tables/1` package; with
//! no package (no core, a pre-7.7.0 core) the hook stays inert — exit
//! 0, never a deny — and a stderr line alone is invisible in Claude
//! Code. So the gate writes the observe-feed line every other degraded
//! path writes (`degraded` true, `reason` = the CLI's own refusal), and
//! the SessionStart line names it once per session (health.rs).

use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// What the hooks' intake gate answered: not this hook's event, or no
/// project here (`Shut`); a project the hook cannot measure, its trace
/// already written (`Inert`, the package's named refusal); a project
/// to measure (`Open`).
pub enum Gate<T> {
    Shut,
    Inert(String),
    Open(T, PathBuf),
}

/// hookio::gated_envelope with its answer named, for the SessionStart
/// line, which must say when the hooks cannot measure. No package =
/// nothing to measure with: the hook stays inert — exit 2 here would
/// read as a PreToolUse deny — and leaves its trace.
pub fn gate<T: serde::de::DeserializeOwned>(
    event: &str,
    base: impl Fn(&T) -> (&str, &str),
) -> Gate<T> {
    let raw = match super::read_envelope::<Value>() {
        Ok(raw) => raw,
        Err(super::Intake::Refused(why)) => return untraced(event, &why),
        Err(super::Intake::Ignored) => return Gate::Shut,
    };
    let Ok(env) = T::deserialize(&raw) else {
        return Gate::Shut;
    };
    let (got, cwd) = base(&env);
    // another event's envelope is not this hook's call: silence; this
    // event's envelope with no cwd is, and says so (it went quiet with
    // no trace until release 1.10.0)
    if got != event {
        return Gate::Shut;
    }
    if cwd.is_empty() {
        return untraced(event, "hook envelope names no cwd — not judged");
    }
    // a POSIX-form cwd on Windows (`/d/x`) names no directory here: the
    // ascent fell to an unanchored path and the hook went quiet with no
    // trace (v2.35 G4, P-A3). Claude Code sends the native form; any
    // other sender is told, through the feed of the project it runs in
    if cfg!(windows) && cwd.starts_with('/') {
        let why = format!("hook envelope cwd `{cwd}` is not a Windows path — not judged");
        return untraced(event, &why);
    }
    let root = super::project_root(cwd);
    if !crate::root::is_anchored(&root) {
        return Gate::Shut;
    }
    crate::tables::anchor(&root);
    match crate::tables::load() {
        Ok(_) => Gate::Open(env, root),
        Err(why) => {
            trace(event, &root, &raw, &why);
            Gate::Inert(why)
        }
    }
}

/// The feed event each hook's lines carry: the probe and the Stop
/// audit keep their own; the PostToolUse leg and the SessionStart line
/// write none of their own when they measure, so the trace is named
/// after their command.
const EVENTS: [(&str, &str); 4] = [
    ("PreToolUse", "probe"),
    ("PostToolUse", "settle"),
    ("Stop", "stop_audit"),
    ("SessionStart", "health"),
];

/// A hook call refused before its envelope named a project: the trace
/// goes to the project the hook process runs in (Claude Code starts
/// hooks in the session's directory), or nowhere when that is no
/// project — the same rule an unanchored envelope cwd keeps.
fn untraced<T>(event: &str, why: &str) -> Gate<T> {
    let root = std::env::current_dir().map(|here| crate::root::project_root(&here));
    if let Ok(root) = root
        && crate::root::is_anchored(&root)
    {
        trace(event, &root, &Value::Null, why);
    }
    Gate::Shut
}

/// The inert hook's trace: the refusal on stderr and one feed line.
fn trace(event: &str, root: &Path, raw: &Value, why: &str) {
    eprintln!("ce: {why}");
    degraded(event, root, raw, why);
}

/// The feed line of a face left inert by an unreadable package: the
/// hooks' (named after their command) and the git-hook faces' (`event`
/// as given, `raw` null), the tool call's file and identity on it when
/// the envelope carries them.
pub(crate) fn degraded(event: &str, root: &Path, raw: &Value, why: &str) {
    let name = EVENTS
        .iter()
        .find(|(e, _)| *e == event)
        .map_or(event, |(_, n)| n);
    let mut line = json!({"event": name, "degraded": true, "reason": why});
    if let Some(file) = raw["tool_input"]["file_path"].as_str() {
        line["file"] = json!(file);
    }
    if let Some(id) = raw["tool_use_id"].as_str().filter(|s| !s.is_empty()) {
        line["tool_use_id"] = json!(id);
    }
    super::observe_append(root, raw["session_id"].as_str(), line);
}
