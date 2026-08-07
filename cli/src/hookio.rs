//! Shared hook plumbing: stdin envelope intake and the observe-feed
//! writer. Extracted from three near-identical copies in
//! guard/audit/health after the self-ratchet flagged the envelope
//! family. Hooks are FAIL-OPEN — intake errors surface as None and
//! the caller exits 0.

use std::path::Path;

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
/// evaluation feed, plan D2-1), stamping `ts_ms`. Failures are
/// swallowed by design: the feed is telemetry, never worth failing a
/// hook over.
pub fn observe_append(root: &Path, mut line: serde_json::Value) {
    let dir = root.join(".ce");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let epoch_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
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
