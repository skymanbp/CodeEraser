//! Shared hook plumbing: stdin envelope intake and the observe-feed
//! writer. Extracted from three near-identical copies in
//! guard/audit/health after the self-ratchet flagged the envelope
//! family. Hooks are FAIL-OPEN — intake errors surface as None and
//! the caller exits 0.

// Root anchoring lives in crate::root (one ascent for the CLI, MCP,
// GUI and the hooks alike); this is the hooks' own door to it — the
// envelope cwd arrives as a &str, so the door keeps that shape.
// Private: gated_envelope below is its only caller (no caller outside
// this file remains, so there is no path to keep open).
fn project_root(cwd: &str) -> std::path::PathBuf {
    crate::root::project_root(Path::new(cwd))
}

use std::path::Path;

/// Observe-feed line schema id, stamped on every entry; bump on any
/// shape change (plan §7.1 discipline — the feed is the M4
/// evaluation-set raw material, so its shape is a contract, pinned
/// by contracts/fixtures/observe-feed/feed.golden.json).
///
/// 0.9.0 (plan v2.27): the `tombstone` object is judged over the wire
/// (tombstone/1) — its counts and sites live under `judged` (`sites` as
/// `file:line kind`, `label`, `prose`, `over`), or `judged.degraded`
/// names why there is none; `rows` counts the candidate surfaces sent;
/// an `exempt` entry may carry `line` (a segment) and `why: declared`
/// (a file `[tombstone] ledger` names); a `tombstone` event's `mode`
/// is the class's own `[tombstone] tier`; the `commitmsg` event (plan
/// v2.27 step 5) is `precommit`'s line written by `ce commitmsg` over
/// the staged set plus the message, session null like precommit's. A
/// `tombstone` event carries `applied` — true when the hook let the
/// write through, false when it denied it (that erasure never
/// happened, and the session union skips the line), null under `ask`
/// (the person decided, the hook cannot see what) — and, when any,
/// `revived_hashes`: session keys its after side declares again, which
/// leave the union. `degraded_pairs` (any producer) counts pairs whose
/// line diff was bounded and `unread_pairs` (audit lines) the pairs the
/// batch could not read: an incomplete measurement is recorded, never
/// enforced (codex review 2026-09-04).
/// Replaces 0.8.0's Rust-judged `label` / `prose` / `sites` — a named
/// break of a shape no release carried.
///
/// 0.8.0 adds the `tombstone` event (the per-edit leg of plan v2.26's
/// residue measurement, feed-only in every tier) and the OPTIONAL
/// `tombstone` object on `stop_audit` / `precommit` lines; every
/// prior key keeps its shape.
///
/// 0.7.0 re-defines `matches` on `probe` events (K step 11 novel-
/// duplication semantics): the count of matches the REPLACED content
/// did not already carry (guard::novel_matches), where it was the
/// raw probe-match count before. No key changes shape; the bump
/// marks the semantic break, because pre/post counts are not
/// comparable as FPR raw material — and that material is this
/// feed's whole job.
///
/// 0.6.0 adds the OPTIONAL `zone_tier` key on `zone` events (plan
/// v2.7 ①): present exactly when ce.toml arms the zone→tier map —
/// the tier the mapped position resolved to, so the feed records
/// what the armed rule DID, not only where the write landed.
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
pub const OBSERVE_SCHEMA: &str = "ce.observe/0.9.0";

/// How much envelope the hooks take from stdin. A bare `read_to_string`
/// bounds nothing: an oversized payload is materialized whole, and a
/// writer that never closes parks the PreToolUse path — budgeted at
/// p95 < 1 s — until the harness kills it. Same cap and same stance as
/// the bounded file reader (`tombstone::texts::READ_CAP`, "an unbounded
/// read is an availability hole"); 4 MiB clears the biggest thing an
/// envelope legitimately carries, a Write's whole file body.
const ENVELOPE_CAP: u64 = 4 << 20;

/// The hooks' shared intake gate: envelope in, event name checked,
/// cwd non-empty, root resolved — and ANCHORED. The stanza grew a
/// copy per hook and the dedup gate refused the third — the drift
/// was a class, so the gate is a throat. `base` projects (event,
/// cwd) out of the hook's own envelope type — a closure, because a
/// trait needed one byte-identical impl block per hook and the gate
/// refused THOSE as clones too. None = not for this hook: exit 0.
///
/// The anchor rule is a class policy too (batch-8 review): it lived
/// as one local copy in guard, another in audit, and NO copy in
/// health — the one hook whose lazy daemon spawn plants `.ce/` and
/// a full winnowing index in whatever directory a session happened
/// to start in (a home dir, a drive root). No anchor anywhere up
/// the chain = no project here: every hook stays inert.
pub fn gated_envelope<T: serde::de::DeserializeOwned>(
    event: &str,
    base: impl Fn(&T) -> (&str, &str),
) -> Option<(T, std::path::PathBuf)> {
    let env = read_envelope::<T>()?;
    let (got, cwd) = base(&env);
    if got != event || cwd.is_empty() {
        return None;
    }
    let root = project_root(base(&env).1);
    if !crate::root::is_anchored(&root) {
        return None;
    }
    Some((env, root))
}

/// Read the hook envelope from stdin and deserialize it.
/// None = unreadable stdin, over-cap payload, or unparseable JSON —
/// the caller treats that as "not for me" and exits 0 (fail-open).
pub fn read_envelope<T: serde::de::DeserializeOwned>() -> Option<T> {
    parse_envelope(std::io::stdin().lock())
}

/// The bounded intake, split off only so a test can hand it an
/// over-cap reader. Over cap takes the existing fail-open None, never
/// a truncated parse: a hook that cannot see the whole envelope must
/// not judge on a prefix of it.
fn parse_envelope<T: serde::de::DeserializeOwned>(src: impl std::io::Read) -> Option<T> {
    use std::io::Read as _;
    let mut raw = Vec::new();
    // cap + 1: reading one byte past tells us it was truncated
    src.take(ENVELOPE_CAP + 1).read_to_end(&mut raw).ok()?;
    if raw.len() as u64 > ENVELOPE_CAP {
        return None;
    }
    serde_json::from_slice(&raw).ok()
}

/// Append one entry to `<root>/.ce/observe.ndjson` (the untainted M4
/// evaluation feed, plan D2-1), stamping `schema`, `session_id` and
/// `ts_ms`. Failures are swallowed by design: the feed is telemetry,
/// never worth failing a hook over.
///
/// `session` is `None` for producers that genuinely have no session —
/// `ce precommit` / `ce commitmsg` run in a terminal, not as a hook. An EMPTY string
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

/// The marker `clip` appends. A sentence a person reads, so it
/// answers CE_LANG like the sentence it truncates; its own byte
/// length is what the budget gives back, which is why the cut is
/// computed from it. Named rather than inline so a test can assert
/// the tail without picking a language.
pub fn clip_mark() -> &'static str {
    crate::i18n::t(
        "… (clipped; full report in .ce/observe.ndjson)",
        "…（已截断；完整记录见 .ce/observe.ndjson）",
    )
}

/// Clip an injected reason to its token budget, on a char boundary,
/// with a marker pointing at the on-disk full record.
pub fn clip(reason: &str, budget_tokens: usize) -> String {
    let cap = budget_tokens * CHARS_PER_TOKEN;
    if reason.len() <= cap {
        return reason.to_string();
    }
    let mark = clip_mark();
    let mut cut = cap.saturating_sub(mark.len());
    while cut > 0 && !reason.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}{mark}", &reason[..cut])
}

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
    let Ok(feed) = tail(&root.join(".ce/observe.ndjson"), FEED_WINDOW) else {
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
            && (rule != "probe" || v["matches"].as_u64().unwrap_or(0) > 0)
            && (rule != "zone" || v["zone_tier"].as_str().is_some_and(|t| t != "observe"))
    })
}

#[cfg(test)]
#[path = "../tests/unit/hookio.rs"]
mod tests;
