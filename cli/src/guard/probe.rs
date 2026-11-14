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
    let matches = novel_matches(root, env, content, applied);
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
/// replaces the on-disk file (its text off `Applied`, read once per
/// hook call), Edit replaces `old_string` — what the REPLACED content
/// already matched is carried forward, not written anew, counted with
/// multiplicity (`novel`). The baseline probe runs only when the first
/// probe matched (no read at all for a clean write), and an unreadable
/// or degraded baseline subtracts nothing: allowance must never ride an
/// unanswered question, the same discipline as probe_matches' array
/// check. None = degraded, as before.
fn novel_matches(
    root: &Path,
    env: &Envelope,
    content: &str,
    applied: Option<&Applied>,
) -> Option<Vec<serde_json::Value>> {
    let file_path = &env.tool_input.file_path;
    let matches = probe_matches(root, file_path, content)?;
    if matches.is_empty() {
        return Some(matches);
    }
    let replaced = if env.tool_name == "Write" {
        applied.and_then(|a| a.prior_text(env)).unwrap_or_default()
    } else {
        env.tool_input.old_string.as_str()
    };
    if replaced.is_empty() {
        return Some(matches);
    }
    let base = probe_matches(root, file_path, replaced).unwrap_or_default();
    Some(novel(matches, &base))
}

/// A match's place for `subtract`: its source file, first and last
/// source line, and how many copies of that region it stands for.
pub type Region = (String, u64, u64, u64);

/// Subtraction with multiplicity, by source line, counted in UNITS: a
/// match stands for `copies` units on its region (the probe's count of
/// disjoint copies in the probed text). cover(l) = the sum of `copies`
/// over the replaced text's matches covering source line l (same source
/// file). The new text's matches are walked in the order the probe
/// returned them, and each match's units one after another, keeping
/// used(l): a unit is CARRIED iff some line of the region has
/// used(l) < cover(l), and then adds 1 to used(l) on every line of the
/// region; any other unit is NOVEL. A match is novel iff one of its
/// units is, and is kept once (the deny names source regions).
///   (P1) with one copy everywhere and at most one new match per
///        replaced one on every line this is the older "any overlap
///        carries" rule: a copy the probe split in two after an
///        insertion inside it is still carried;
///   (P2) n identical copies over a replaced text that held k of them
///        are carried iff n <= k;
///   (P3) a second copy whose run ends elsewhere (another length on the
///        same source lines) is a second match and finds every line
///        used up: novel.
/// A match whose region or count is unanswered (`region` answers None,
/// or a count below 1) is novel on the new side and adds no cover on
/// the replaced side: an allowance never rides an unanswered question.
/// Generic over the match's shape so the standing false-block replay
/// (the tests' fpr_replay) reads this rule, not a copy of it.
pub fn subtract<T>(matches: Vec<T>, base: &[T], region: impl Fn(&T) -> Option<Region>) -> Vec<T> {
    use std::collections::HashMap;
    // a match's lines, keyed by source file, and its unit count
    let place = |m: &T| -> Option<(Vec<(String, u64)>, u64)> {
        let (f, s, e, copies) = region(m).filter(|r| r.3 >= 1)?;
        Some(((s..=e).map(|l| (f.clone(), l)).collect(), copies))
    };
    let mut cover: HashMap<(String, u64), u64> = HashMap::new();
    for (keys, copies) in base.iter().filter_map(&place) {
        for k in keys {
            *cover.entry(k).or_default() += copies;
        }
    }
    let mut used: HashMap<(String, u64), u64> = HashMap::new();
    let mut out = Vec::new();
    for m in matches {
        let carried = place(&m)
            .is_some_and(|(keys, copies)| (0..copies).all(|_| take_unit(&keys, &cover, &mut used)));
        if !carried {
            out.push(m);
        }
    }
    out
}

/// One unit over `keys`: carried iff some line still has used < cover,
/// and then it uses one on every line; false = a novel unit.
fn take_unit(
    keys: &[(String, u64)],
    cover: &std::collections::HashMap<(String, u64), u64>,
    used: &mut std::collections::HashMap<(String, u64), u64>,
) -> bool {
    let free =
        |k: &(String, u64)| used.get(k).copied().unwrap_or(0) < cover.get(k).copied().unwrap_or(0);
    if !keys.iter().any(free) {
        return false;
    }
    for k in keys {
        *used.entry(k.clone()).or_default() += 1;
    }
    true
}

/// The probe reply's matches through `subtract`: the source file is
/// the match's `file`, the region its `start_line`..=`end_line`, the
/// unit count its `copies` (absent or not a positive integer: None).
fn novel(matches: Vec<serde_json::Value>, base: &[serde_json::Value]) -> Vec<serde_json::Value> {
    subtract(matches, base, |v| {
        let (s, e) = (v["start_line"].as_u64()?, v["end_line"].as_u64()?);
        Some((v["file"].to_string(), s, e, v["copies"].as_u64()?))
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

#[cfg(test)]
#[path = "../../tests/unit/guard/probe.rs"]
mod tests;
