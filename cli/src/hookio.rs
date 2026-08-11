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
/// 0.2.0 adds `session_id`. 0.1.0 carried no session identity at all,
/// which left two M3/M4 acceptance criteria unmeasurable: "dogfood
/// sessions >= 10, of which observe-mode >= 5" (D2-2) cannot be
/// counted from a feed that does not say which session a line came
/// from, and D2-1 sample purity — excluding edits a guard intervened
/// in — needs the same partition. Measured before the bump: 49
/// entries, all from one hour, with no way to tell whether that was
/// one session or ten.
pub const OBSERVE_SCHEMA: &str = "ce.observe/0.3.0";

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
    if let Ok(mut fh) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("observe.ndjson"))
    {
        let _ = writeln!(fh, "{line}");
    }
}
