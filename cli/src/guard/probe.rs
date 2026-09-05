//! The duplicate-write probe of the PreToolUse gate (split from
//! guard.rs in the codex review batch, 2026-09-04, at the 300-line
//! dogfood wall): the daemon's T1/T2 probe over the content about to
//! be written, the NOVEL filter that subtracts what the replaced
//! content already carried, and the class's reason.

use super::envelope::Envelope;
use super::say;
use crate::daemon::client;
use crate::daemon::proto::{Request, Response};
use std::path::Path;

/// The rule denies NEW duplication only (K step 11 root fix): the
/// FPR replay measured the whole-content probe denying full-file
/// rewrites of files that already carry budgeted blocks. Write
/// replaces the on-disk file, Edit replaces `old_string` — a match
/// whose source region overlaps a match the REPLACED content already
/// had is carried forward, not written anew. The baseline probe runs
/// only when the first probe matched (no unbounded read for a clean
/// write), and a degraded baseline subtracts nothing: allowance must
/// never ride an unanswered question, the same discipline as
/// probe_matches' array check. None = degraded, as before.
pub(super) fn novel_matches(root: &Path, env: &Envelope) -> Option<Vec<serde_json::Value>> {
    let file_path = &env.tool_input.file_path;
    let content = if env.tool_name == "Write" {
        &env.tool_input.content
    } else {
        &env.tool_input.new_string
    };
    let matches = probe_matches(root, file_path, content)?;
    if matches.is_empty() {
        return Some(matches);
    }
    let replaced = if env.tool_name == "Write" {
        std::fs::read_to_string(file_path).unwrap_or_default()
    } else {
        env.tool_input.old_string.clone()
    };
    if replaced.is_empty() {
        return Some(matches);
    }
    let base = probe_matches(root, file_path, &replaced).unwrap_or_default();
    Some(matches.into_iter().filter(|m| !carried(m, &base)).collect())
}

/// Same source file, overlapping source region: the replaced content
/// already matched it, so the new content merely carries it.
fn carried(m: &serde_json::Value, base: &[serde_json::Value]) -> bool {
    let span = |v: &serde_json::Value| (v["start_line"].as_u64(), v["end_line"].as_u64());
    let (ms, me) = span(m);
    base.iter().any(|b| {
        let (bs, be) = span(b);
        b["file"] == m["file"] && bs <= me && ms <= be
    })
}

/// None = probe unavailable (degraded); Some(vec) = verified matches.
fn probe_matches(root: &Path, file_path: &str, content: &str) -> Option<Vec<serde_json::Value>> {
    let req = Request::Probe {
        file_path: file_path.to_string(),
        content: content.to_string(),
    };
    match client::request(root, &req) {
        // Only a real array is a verified answer. `unwrap_or_default`
        // collapsed null / an object / a string into Some(vec![]) —
        // "a healthy probe with no duplicates" — laundering a
        // malformed report into an allow. Any other shape is the
        // degraded None this function's own contract names.
        Ok(Response::ProbeReport {
            matches: serde_json::Value::Array(list),
            ..
        }) => Some(list),
        _ => None,
    }
}

/// The duplicate class's reason: the top matches as `file:a-b (n tokens)`.
pub(super) fn reason(file_path: &str, matches: &[serde_json::Value]) -> String {
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
    say::duplicate(file_path, matches.len(), &top.join("; "))
}
