//! The observe feed's session readers (split from hookio.rs at the
//! 300-line wall, release 1.10.0): this session's lines in a bounded
//! window of the append-only feed, and whether a rule already fired.

use super::OBSERVE_REL;
use std::path::Path;

/// How much of the append-only feed a suppression query reads. The
/// question is per-SESSION and the session's own lines are always at
/// the end, so the whole file was never the answer — it was the cost:
/// `read_to_string` + a JSON parse per line, twice per Write/Edit, on
/// the PreToolUse path budgeted at p95 < 1 s, growing linearly with
/// project history forever. A window bounds that cost without
/// touching the ledger: the feed stays append-only (its promotion
/// record must not rotate away), and 1 MiB is ~4000 entries at this
/// repo's measured 245 B/line — several sessions deep.
const FEED_WINDOW: u64 = 1 << 20;

/// The last `cap` bytes of a file, starting at a line boundary (a
/// window that opens mid-line would hand a truncated fragment to the
/// JSON parser, which drops it — quietly losing one entry).
fn tail(path: &Path, cap: u64) -> std::io::Result<String> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path)?;
    let len = f.metadata()?.len();
    if len <= cap {
        let mut all = String::new();
        f.read_to_string(&mut all)?;
        return Ok(all);
    }
    f.seek(SeekFrom::Start(len - cap))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let start = buf.iter().position(|b| *b == b'\n').map_or(0, |i| i + 1);
    Ok(String::from_utf8_lossy(&buf[start..]).into_owned())
}

/// This session's lines in the feed window, parsed — the accumulator
/// every per-session reader asks (the warn suppression below, the
/// tombstone leg's erased-key union). A read or parse failure, or no
/// session, = no lines: fail open toward REPORTING, the opposite bias
/// from enforcement fail-open.
pub fn session_lines(root: &Path, session: &str) -> Vec<serde_json::Value> {
    if session.is_empty() {
        return Vec::new();
    }
    let Ok(feed) = tail(&root.join(OBSERVE_REL), FEED_WINDOW) else {
        return Vec::new();
    };
    feed.lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter(|v| v["session_id"] == session)
        .collect()
}

/// §4.4 B4 session-level suppression: has this (rule, file) already
/// FIRED for this session? The observe feed IS the accumulator the
/// clause's "silently accumulate" half names — probe lines count only
/// when matches > 0 (a clean probe never warned anyone), and zone
/// lines only when an ARMED map produced a real tier (an observe-leg
/// ledger line warned nobody either — v2.7 ①).
pub fn already_warned(root: &Path, session: &str, rule: &str, file: &str) -> bool {
    session_lines(root, session).iter().any(|v| {
        v["event"] == rule
            && v["file"] == file
            // a carried budget line (0.14.0) recorded a write the rule
            // let through: nothing was said, so nothing was warned
            && (rule != "budget" || v["carried"] != true)
            // a probe line warned when matches remained that were not
            // relocations (0.14.0: `relocated` counts those credited)
            && (rule != "probe"
                || v["matches"].as_u64().unwrap_or(0) > v["relocated"].as_u64().unwrap_or(0))
            && (rule != "zone" || v["zone_tier"].as_str().is_some_and(|t| t != "observe"))
    })
}
