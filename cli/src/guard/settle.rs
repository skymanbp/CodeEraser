//! The PostToolUse leg (plan v2.30 step 5b, the 2026-09-27 ruling):
//! `ce settle --hook`. Claude Code fires PostToolUse only after a tool
//! ran, and stamps it with the same `tool_use_id` as the PreToolUse
//! event (the hooks reference; the captured envelopes carry the id),
//! so a write the guard left to the person at `ask` has the witness
//! the guard itself never had: the tool running IS the person letting
//! it through. A refusal at the prompt fires nothing — PermissionDenied
//! is auto mode's, PostToolUseFailure a tool that ran and failed — so
//! the absence of a `settled` line is the record of a refusal. The leg
//! writes one `settled` line, only for an event this session's feed
//! shows decided at `ask` (every other outcome is already in the
//! PreToolUse lines), once, and never speaks: PostToolUse output goes
//! to the model, and a recording leg that talks is the context-entropy
//! source the plan retired the deep PostToolUse leg over (§3, B4).

use std::path::Path;
use std::process::ExitCode;

/// Entry point for `ce settle --hook`: the write hooks' shared gate
/// (guard::write_event — the throat's policy, the tool filter, the
/// judging root that keeps the feed), then one bounded read of the
/// session's lines. Never fails outward.
pub fn run_hook() -> ExitCode {
    let Some((env, root)) = super::write_event("PostToolUse") else {
        return ExitCode::SUCCESS;
    };
    if !env.tool_use_id.is_empty() && asked(&root, &env.session_id, &env.tool_use_id) {
        super::feed(
            &root,
            &env,
            serde_json::json!({"event": "settled", "file": env.tool_input.file_path}),
        );
    }
    ExitCode::SUCCESS
}

/// Whether this session's feed holds tool call `id` decided at `ask`
/// and not yet settled: the `probe` line every probed event leaves
/// carries the tier the hook decided at (`decision`, 0.11.0); a
/// second PostToolUse under the same id — the harness re-firing — adds
/// no second line.
fn asked(root: &Path, session: &str, id: &str) -> bool {
    let lines = crate::hookio::session_lines(root, session);
    let of = |event: &str| {
        lines
            .iter()
            .any(|v| v["event"] == event && v["tool_use_id"] == id)
    };
    lines
        .iter()
        .any(|v| v["event"] == "probe" && v["tool_use_id"] == id && v["decision"] == "ask")
        && !of("settled")
}
