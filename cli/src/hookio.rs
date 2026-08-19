//! Shared hook plumbing: stdin envelope intake and the observe-feed
//! writer. Extracted from three near-identical copies in
//! guard/audit/health after the self-ratchet flagged the envelope
//! family. Hooks are FAIL-OPEN — intake errors surface as None and
//! the caller exits 0.

use std::path::Path;

/// Observe-feed line schema id, stamped on every entry; bump on any
/// shape change (plan §7.1 discipline — the feed is the M4
/// evaluation-set raw material, so its shape is a contract, pinned
/// by contracts/fixtures/observe-feed/feed.golden.json).
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
pub const OBSERVE_SCHEMA: &str = "ce.observe/0.4.0";

/// The project root for a hook envelope's cwd: the NEAREST ancestor
/// (cwd itself first) holding a `ce.toml` or a `.git` — cross-session
/// field report 2026-08-18: the raw cwd made the same edit judge
/// differently depending on where the shell had cd'd, and fragmented
/// the index/daemon per directory. A tree with neither anchor keeps
/// the cwd verbatim (the old behavior as the honest fallback). One
/// throat for all three hooks — the drift was a class, not a site.
pub fn project_root(cwd: &str) -> std::path::PathBuf {
    let start = std::path::PathBuf::from(cwd);
    let mut probe = start.as_path();
    loop {
        if probe.join("ce.toml").is_file() || is_git_anchor(&probe.join(".git")) {
            return probe.to_path_buf();
        }
        match probe.parent() {
            Some(p) if !p.as_os_str().is_empty() => probe = p,
            _ => return start,
        }
    }
}

/// A REAL git anchor: the `.git` dir or a worktree gitfile. `.exists()`
/// took any FILE of that name, so one Write re-rooted a subtree's hooks.
fn is_git_anchor(p: &Path) -> bool {
    p.is_dir() || std::fs::read_to_string(p).is_ok_and(|s| s.starts_with("gitdir:"))
}

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

    /// The hook root ascends to the nearest anchor (ce.toml or
    /// .git), cwd itself first; an anchorless tree keeps the cwd —
    /// the field-report counterexample was `cd background/` flipping
    /// the same write's verdict.
    #[test]
    fn project_root_ascends_to_the_nearest_anchor() {
        let dir = std::env::temp_dir().join(format!("ce-root-{}", std::process::id()));
        let deep = dir.join("repo/sub/deep");
        std::fs::create_dir_all(&deep).expect("mkdir");
        std::fs::write(dir.join("repo/ce.toml"), "\n").expect("anchor");
        let cwd = deep.to_string_lossy().to_string();
        assert_eq!(project_root(&cwd), dir.join("repo"), "ascends to ce.toml");
        assert_eq!(
            project_root(&dir.join("repo").to_string_lossy()),
            dir.join("repo"),
            "cwd itself first"
        );
        let loose = dir.join("loose");
        std::fs::create_dir_all(&loose).expect("mkdir");
        let lc = loose.to_string_lossy().to_string();
        // the walk above `loose` may cross REAL anchors on the host
        // (temp dirs live under a user profile) — assert the honest
        // property instead: the answer is `loose` itself or one of
        // its ancestors carrying a real anchor, never a sibling
        let got = project_root(&lc);
        assert!(loose.starts_with(&got), "never leaves the ancestry line");
        std::fs::remove_dir_all(&dir).ok();
    }

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
