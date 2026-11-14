//! Shared hook plumbing: stdin envelope intake and the observe-feed
//! writer. Extracted from three near-identical copies in
//! guard/audit/health after the self-ratchet flagged the envelope
//! family. Hooks are FAIL-OPEN — intake errors surface as None and
//! the caller exits 0.

// Root anchoring lives in crate::root (one ascent for the CLI, MCP,
// GUI and the hooks alike); this is the hooks' own door to it — the
// envelope cwd arrives as a &str, so the door keeps that shape.
// Private: the gate (hookio/inert.rs) is its only caller (no caller
// outside this module remains, so there is no path to keep open).
fn project_root(cwd: &str) -> std::path::PathBuf {
    crate::root::project_root(Path::new(cwd))
}

mod feed;
mod inert;
pub use feed::{already_warned, session_lines};
pub(crate) use inert::degraded;
pub use inert::{Gate, gate};
use std::path::Path;

/// Observe-feed line schema id, stamped on every entry; bump on any
/// shape change (plan §7.1 discipline — the feed is the M4
/// evaluation-set raw material, so its shape is a contract, pinned
/// by contracts/fixtures/observe-feed/feed.golden.json).
///
/// 0.14.0 (v2.35 guard lane): the PreToolUse classes judge what a
/// write ADDS. A `budget` line (written for every write past the
/// line) carries `before_lines` and exactly one of `kind` (`crossing`
/// | `growth`, a refused write), `carried: true` (a write that keeps or
/// shrinks a file already past the line: recorded, not refused) or
/// `before_unread` (the size before could not be read: refused as
/// before, with a top-level `degraded` string); a `zone` line carries
/// `before_lines` and `carried: true` on a write that does not grow
/// the file (its tier then does not fire, `zone_tier` reads
/// `observe`); a `probe` line whose Edit the hook could not apply
/// carries `unapplied` (`absent` | `unreadable` | `no_match` |
/// `ambiguous`) — and for `unreadable` a nested `apply.degraded`
/// stamp; a `probe` line whose novel matches went to the relocation
/// stage carries `relocated` (how many the core judged relocations —
/// all of them, and the class did not refuse) or `relocation` (why the
/// stage could not credit: no git, a cap, an unread blob, the core);
/// the PostToolUse leg re-measures the file that landed in the shared
/// index (GD7) and, when that fails, writes a `settled` line carrying
/// `reindex.degraded` (a settled line without an ask behind it is that
/// record alone). An envelope the hook cannot read leaves the same
/// `degraded` / `reason` line the inert hook writes, in the feed of the
/// project the hook runs in: over the cap, not JSON, a cwd that is not
/// a native path, or no cwd at all. The `fourclass` object on a
/// `stop_audit` line may carry `unread_pairs`: the changed pairs the
/// daemon's loader could not read (a side missing, binary or past the
/// read cap, a pair past the pair cap), never read as clean. Every
/// prior key keeps its shape.
///
/// 0.13.0 (plan v2.32 step 2 follow-up): a hook left inert by an
/// unreadable definition package writes one line — `probe` /
/// `settle` / `stop_audit` / `health` with `degraded` true and
/// `reason` (hookio/inert.rs); since 1.9.2 the git-hook faces too
/// (`precommit` / `commitmsg`, which then skip with exit 0). Every
/// prior key keeps its shape.
///
/// 0.12.0 (plan v2.31 step 5): the `flow` event (`mode` = `[flow]
/// tier`; a `flow` object: `units`, `before` / `after` finding counts,
/// `novel`, `kinds` by name, `judged`, or `degraded`) and the OPTIONAL
/// `flow` object on audit lines (`files`, `units`, `findings`, `kinds`,
/// `judged`, or `degraded`). Every prior key keeps its shape.
///
/// 0.11.0 (plan v2.30 step 5b): every line the PreToolUse hook writes
/// carries `tool_use_id` (Claude Code's identity for the tool call, when
/// the envelope has one); the `probe` line carries `decision`, the tier
/// the hook decided the event at (`observe` when nothing was emitted,
/// else `warn` / `ask` / `deny`); the `settled` event (`file`,
/// `tool_use_id`) is the PostToolUse leg's record that the tool ran after
/// an `ask` — the person let the write through — and a `tombstone` line's
/// `applied` null is read through it. Every prior key keeps its shape.
///
/// 0.10.0 (plan v2.29 step 6): the OPTIONAL `similar` object on
/// `stop_audit` lines — the same-role advisor asked about every unit
/// the session added (`new_units`, `queried`), `rows` of `{unit, twin,
/// score}` for those whose top-1 the core judged same-role over
/// similar/1, `degraded` naming why the core did not judge; absent
/// when there is nothing to say. The `fourclass` object may carry
/// `recovered: <n>` on the first report after the daemon's core link
/// came back from n failed spawn attempts (7.0.0, O63 — the retry
/// budget backs off and never closes). Every prior key keeps its shape.
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
/// 0.4.0 adds the `budget` event (§4.2 step-2 per-rule firing records).
///
/// 0.2.0 adds `session_id`: without it the D2-2 session count and
/// the D2-1 sample purity could not be measured (49 entries, one hour,
/// no way to tell one session from ten).
pub const OBSERVE_SCHEMA: &str = "ce.observe/0.14.0";

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
    match gate(event, base) {
        Gate::Open(env, root) => Some((env, root)),
        Gate::Shut | Gate::Inert(_) => None,
    }
}

/// Why an envelope did not reach its hook. Unreadable stdin is not a
/// hook call at all; an envelope over the cap IS one — the largest
/// Writes — and so is input that is not JSON (Claude Code sent
/// something the hook cannot read): neither passes untraced (v2.35
/// G4, R3-13; release 1.10.0 for the second): the refusal is named and
/// the gate writes it to the feed (inert.rs). All keep the fail-open
/// exit 0.
pub(crate) enum Intake {
    Ignored,
    Refused(String),
}

/// Read the hook envelope from stdin and deserialize it.
pub(crate) fn read_envelope<T: serde::de::DeserializeOwned>() -> Result<T, Intake> {
    parse_envelope(std::io::stdin().lock())
}

/// The bounded intake, split off only so a test can hand it an
/// over-cap reader. Over cap is a named refusal, never a truncated
/// parse: a hook that cannot see the whole envelope must not judge on
/// a prefix of it.
fn parse_envelope<T: serde::de::DeserializeOwned>(src: impl std::io::Read) -> Result<T, Intake> {
    use std::io::Read as _;
    let mut raw = Vec::new();
    // cap + 1: reading one byte past tells us it was truncated
    src.take(ENVELOPE_CAP + 1)
        .read_to_end(&mut raw)
        .map_err(|_| Intake::Ignored)?;
    if raw.len() as u64 > ENVELOPE_CAP {
        let why = format!("hook envelope over the {ENVELOPE_CAP}-byte cap — not judged");
        return Err(Intake::Refused(why));
    }
    serde_json::from_slice(&raw)
        .map_err(|_| Intake::Refused("hook envelope is not JSON — not judged".to_string()))
}

/// The observe feed, root-relative: the one spelling every reader and
/// the writer below join.
pub const OBSERVE_REL: &str = ".ce/observe.ndjson";

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
    let path = root.join(OBSERVE_REL);
    if path
        .parent()
        .is_some_and(|dir| crate::root::make_state_dir(dir).is_err())
    {
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
        .open(&path)
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

#[cfg(test)]
#[path = "../tests/unit/hookio.rs"]
mod tests;
