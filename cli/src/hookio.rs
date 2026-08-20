//! Shared hook plumbing: stdin envelope intake and the observe-feed
//! writer. Extracted from three near-identical copies in
//! guard/audit/health after the self-ratchet flagged the envelope
//! family. Hooks are FAIL-OPEN — intake errors surface as None and
//! the caller exits 0.

// Root anchoring is its own concern (two anchor kinds, one with a
// target that must resolve) — re-exported so every hookio::
// project_root caller keeps its path.
mod anchor;
pub use anchor::project_root;

use std::path::Path;

/// Observe-feed line schema id, stamped on every entry; bump on any
/// shape change (plan §7.1 discipline — the feed is the M4
/// evaluation-set raw material, so its shape is a contract, pinned
/// by contracts/fixtures/observe-feed/feed.golden.json).
///
/// 0.5.0 adds the `zone` event (plan v2.6 §A observe leg): a write
/// landing inside the graded size zone (S, H] — soft/hard/position
/// per line, the per-rule record any future zone→tier promotion
/// must argue its FPR case from.
///
/// 0.4.0 adds the `budget` event (the §4.2 step-2 hard-budget rule
/// keeps per-rule firing records for the step-3 decision at 1.0).
///
/// 0.2.0 adds `session_id`. 0.1.0 carried no session identity at all,
/// which left two M3/M4 acceptance criteria unmeasurable: "dogfood
/// sessions >= 10, of which observe-mode >= 5" (D2-2) cannot be
/// counted from a feed that does not say which session a line came
/// from, and D2-1 sample purity — excluding edits a guard intervened
/// in — needs the same partition. Measured before the bump: 49
/// entries, all from one hour, with no way to tell whether that was
/// one session or ten.
pub const OBSERVE_SCHEMA: &str = "ce.observe/0.5.0";

/// Read the whole hook envelope from stdin and deserialize it.
/// None = unreadable stdin or unparseable JSON — the caller treats
/// that as "not for me" and exits 0 (fail-open).
pub fn read_envelope<T: serde::de::DeserializeOwned>() -> Option<T> {
    use std::io::Read as _;
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Append one entry to `<root>/.ce/observe.ndjson` (the untainted M4
/// evaluation feed, plan D2-1), stamping `schema`, `session_id` and
/// `ts_ms`. Failures are swallowed by design: the feed is telemetry,
/// never worth failing a hook over.
///
/// `session` is `None` for producers that genuinely have no session —
/// `ce precommit` runs in a terminal, not as a hook. An EMPTY string
/// is normalized to null here rather than at the call sites, so a
/// payload that omitted `session_id` can never leave a `""` behind
/// that later reads as a real (and shared) session id.
pub fn observe_append(root: &Path, session: Option<&str>, mut line: serde_json::Value) {
    let dir = root.join(".ce");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let epoch_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    line["schema"] = serde_json::json!(OBSERVE_SCHEMA);
    line["session_id"] = match session.filter(|s| !s.is_empty()) {
        Some(s) => serde_json::json!(s),
        None => serde_json::Value::Null,
    };
    line["ts_ms"] = serde_json::json!(epoch_ms);
    use std::io::Write as _;
    // ONE write_all: `writeln!` gives every fmt fragment its own write()
    // on an unbuffered File, so concurrent hooks interleaved INSIDE a
    // record — and every reader filter_map(ok)'d the wreckage away.
    let record = format!("{line}\n");
    if let Ok(mut fh) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("observe.ndjson"))
    {
        let _ = fh.write_all(record.as_bytes());
    }
}

/// §4.4 B4 anti-bloat budgets — the guard must not itself become a
/// context entropy source. The plan writes budgets in tokens; the
/// hook sees chars, so the conversion rides one declared measurement
/// constant (4 chars/token, the common English heuristic). Deep
/// reports stay on disk (the observe feed) — the clip marker says so.
pub const WARN_BUDGET_TOKENS: usize = 200;
pub const STOP_BUDGET_TOKENS: usize = 400;
const CHARS_PER_TOKEN: usize = 4;

/// Clip an injected reason to its token budget, on a char boundary,
/// with a marker pointing at the on-disk full record.
pub fn clip(reason: &str, budget_tokens: usize) -> String {
    let cap = budget_tokens * CHARS_PER_TOKEN;
    if reason.len() <= cap {
        return reason.to_string();
    }
    const MARK: &str = "… (clipped; full report in .ce/observe.ndjson)";
    let mut cut = cap.saturating_sub(MARK.len());
    while cut > 0 && !reason.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}{MARK}", &reason[..cut])
}

/// §4.4 B4 session-level suppression: has this (rule, file) already
/// FIRED for this session? The observe feed IS the accumulator the
/// clause's "silently accumulate" half names — probe lines count only
/// when matches > 0 (a clean probe never warned anyone). Any read or
/// parse failure = not warned: fail open toward REPORTING, the
/// opposite bias from enforcement fail-open.
pub fn already_warned(root: &Path, session: &str, rule: &str, file: &str) -> bool {
    if session.is_empty() {
        return false;
    }
    let Ok(feed) = std::fs::read_to_string(root.join(".ce/observe.ndjson")) else {
        return false;
    };
    feed.lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .any(|v| {
            v["session_id"] == session
                && v["event"] == rule
                && v["file"] == file
                && (rule != "probe" || v["matches"].as_u64().unwrap_or(0) > 0)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// B4 acceptance half 1: the clip is identity under budget, caps
    /// at budget with the on-disk pointer over it, and never splits a
    /// multi-byte char (the marker itself starts with one).
    #[test]
    fn clip_caps_at_budget_and_respects_char_boundaries() {
        assert_eq!(clip("short", WARN_BUDGET_TOKENS), "short");
        let long = "预算".repeat(400); // 800 chars, 2400 bytes
        let clipped = clip(&long, WARN_BUDGET_TOKENS);
        assert!(clipped.len() <= WARN_BUDGET_TOKENS * CHARS_PER_TOKEN);
        assert!(clipped.ends_with("observe.ndjson)"));
        assert!(clipped.chars().count() > 0); // boundary-safe slice
    }

    /// B4 acceptance half 2: one warn per (rule, file) per session —
    /// a clean probe line never counts, a fired one does, and other
    /// files, sessions and rules stay unsuppressed.
    #[test]
    fn already_warned_is_per_session_rule_and_file() {
        let dir = std::env::temp_dir().join(format!("ce-b4-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let line = |m: u64| serde_json::json!({"event": "probe", "file": "a.rs", "matches": m});
        observe_append(&dir, Some("s1"), line(0));
        assert!(!already_warned(&dir, "s1", "probe", "a.rs"), "clean probe");
        observe_append(&dir, Some("s1"), line(2));
        assert!(already_warned(&dir, "s1", "probe", "a.rs"), "fired probe");
        assert!(
            !already_warned(&dir, "s2", "probe", "a.rs"),
            "other session"
        );
        assert!(!already_warned(&dir, "s1", "probe", "b.rs"), "other file");
        assert!(!already_warned(&dir, "s1", "budget", "a.rs"), "other rule");
        observe_append(
            &dir,
            Some("s1"),
            serde_json::json!({"event": "budget", "file": "a.rs"}),
        );
        assert!(already_warned(&dir, "s1", "budget", "a.rs"), "budget fired");
        assert!(!already_warned(&dir, "", "probe", "a.rs"), "no session");
        std::fs::remove_dir_all(&dir).ok();
    }
}
