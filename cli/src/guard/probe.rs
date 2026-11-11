//! The duplicate-write class of the PreToolUse gate (split from
//! guard.rs in the codex review batch, 2026-09-04, at the 300-line
//! dogfood wall): the daemon's T1/T2 probe over the content about to
//! be written, the NOVEL filter that subtracts what the replaced
//! content already carried, the relocation stage (v2.35 GD4,
//! guard/relocate.rs), the class's reason and its `probe` feed line.

use super::applied::{Applied, Unapplied};
use super::envelope::Envelope;
use super::relocate::{self, Credit};
use super::speech::{Match, Said};
use crate::daemon::client;
use crate::daemon::proto::{Request, Response};
use std::path::Path;

/// The class's answer and its cost — one measurement, one record (the
/// `probe` line every probed event leaves).
pub(super) struct Probed {
    /// The novel matches; None = the probe was unavailable (degraded).
    matches: Option<Vec<serde_json::Value>>,
    /// The relocation stage's answer, asked only when matches remain.
    credit: Option<Credit>,
    elapsed_ms: u128,
}

impl Probed {
    /// The matches the class refuses for: the novel ones that are not
    /// relocations, unless every one is (then none). Empty = no refusal.
    pub(super) fn refusing(&self) -> Vec<serde_json::Value> {
        let Some(ms) = &self.matches else {
            return Vec::new();
        };
        match &self.credit {
            Some(c) if c.all => Vec::new(),
            Some(c) => ms
                .iter()
                .enumerate()
                .filter(|(i, _)| !c.moved.contains(i))
                .map(|(_, m)| m.clone())
                .collect(),
            None => ms.clone(),
        }
    }
}

/// Probe the write, subtract what it carries, credit what it moves.
pub(super) fn probed(root: &Path, env: &Envelope, applied: Option<&Applied>) -> Probed {
    let started = std::time::Instant::now();
    let content = probed_text(env);
    let matches = novel_matches(root, env, content);
    let credit = matches
        .as_deref()
        .filter(|ms| !ms.is_empty())
        .map(|ms| relocate::credit(root, env, content, applied, ms));
    Probed {
        matches,
        credit,
        elapsed_ms: started.elapsed().as_millis(),
    }
}

/// What the probe reads: a Write's payload, an Edit's `new_string`.
fn probed_text(env: &Envelope) -> &str {
    if env.tool_name == "Write" {
        &env.tool_input.content
    } else {
        &env.tool_input.new_string
    }
}

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
fn novel_matches(root: &Path, env: &Envelope, content: &str) -> Option<Vec<serde_json::Value>> {
    let file_path = &env.tool_input.file_path;
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

/// The duplicate class's reason: the count and the top three matches.
pub(super) fn reason(file_path: &str, matches: &[serde_json::Value]) -> Said {
    let int = |v: &serde_json::Value| v.as_i64().unwrap_or(-1);
    let top = matches
        .iter()
        .take(3)
        .map(|m| Match {
            file: m["file"].as_str().unwrap_or("?").to_string(),
            start: int(&m["start_line"]),
            end: int(&m["end_line"]),
            tokens: int(&m["tokens"]),
        })
        .collect();
    Said::Duplicate {
        file: file_path.to_string(),
        regions: matches.len(),
        top,
    }
}

/// One NDJSON line per probed event, all modes (M4 evaluation feed),
/// written once the event is decided so that it carries `decision` —
/// the tier the hook decided the whole event at (0.11.0), which the
/// PostToolUse leg reads to know an `ask` when the tool then runs.
/// `matches` stays the novel count (0.7.0); the relocation stage adds
/// `relocated` beside it, or `relocation` naming why it could not
/// credit (0.14.0, GD4). An Edit the hook could not apply says why
/// (0.14.0, GD3): the size, flow and tombstone legs were silent for
/// want of a text, and an I/O failure reading the target is a nested
/// degraded stamp (`apply`), which `ce doctor` counts.
pub(super) fn line(
    env: &Envelope,
    p: &Probed,
    (mode, decision): (&str, &str),
    unapplied: Option<&Unapplied>,
) -> serde_json::Value {
    let mut line = serde_json::json!({
        "event": "probe",
        "file": env.tool_input.file_path,
        "mode": mode,
        "degraded": p.matches.is_none(),
        "matches": p.matches.as_deref().map(<[serde_json::Value]>::len).unwrap_or(0),
        "elapsed_ms": p.elapsed_ms,
        "decision": decision,
    });
    if let Some(c) = &p.credit {
        line["relocated"] = c.moved.len().into();
        if let Some(why) = &c.why_not {
            line["relocation"] = why.as_str().into();
        }
    }
    if let Some(u) = unapplied {
        line["unapplied"] = u.code().into();
        if let Unapplied::Unreadable(kind) = u {
            line["apply"] = serde_json::json!({"degraded": format!("unreadable: {kind:?}")});
        }
    }
    line
}
