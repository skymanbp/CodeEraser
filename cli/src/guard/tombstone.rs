//! The PreToolUse leg of the tombstone class (plan v2.26 step 4,
//! judged since v2.27 step 4): this edit's (on-disk, applied) pair,
//! measured against the names it erases AND the keys earlier edits of
//! the same session erased — the observe feed is the accumulator, as
//! it is for the warn suppression (hookio::session_lines) — then
//! judged over the daemon's core link (tombstone/1) and recorded. A
//! `tombstone` line lands only when there is something to record: this
//! edit erased a name, one of its surfaces bound a name the session
//! erased before, or it declared such a name again (a REVIVAL, which
//! the union subtracts). The line waits for the hook's decision and
//! carries it as `applied`: a denied write erased nothing. The class
//! speaks at its OWN tier (`[tombstone] tier`, default observe), only
//! when the core says the declared budget is exceeded, and only over a
//! WHOLE measurement (a bounded diff reads untouched lines as written);
//! no core = a degraded line, never a decision.

use super::envelope::Envelope;
use crate::config::{Config, TIERS, TOMBSTONE_DEFAULT};
use crate::daemon::client;
use crate::daemon::proto::{Request, Response};
use crate::tombstone::{self, HASH_CAP, Judgment, PairText, Policy, Row, wire};
use std::collections::BTreeSet;
use std::path::Path;

/// What the leg leaves for the hook: the feed line to append once the
/// decision is known, and the class's reason at its own tier when the
/// condition holds.
pub(super) struct Pending {
    pub line: serde_json::Value,
    pub speak: Option<(&'static str, String)>,
}

/// Measure and judge one Write/Edit. Scope is the budget rule's (a
/// judged language, inside the config's walk) and the applied text is
/// the budget rule's too (budget::resulting_text) — a tool call that
/// is failing on its own measures nothing. `fence` = the drift note
/// when the hook judges with fenced budgets (budget::fenced).
pub(super) fn observe(
    root: &Path,
    env: &Envelope,
    cfg: Option<&Config>,
    fence: Option<&str>,
) -> Option<Pending> {
    let path = Path::new(&env.tool_input.file_path);
    let lang = crate::scan::lang::Lang::judged_path(path)?;
    if cfg.is_some_and(|c| !crate::scan::walk::in_scope(root, path, &c.exclude)) {
        return None;
    }
    let after = super::budget::resulting_text(env)?;
    let before =
        tombstone::texts::read_capped(path).or_else(|| (!path.exists()).then(String::new))?;
    let rel = crate::scan::walk::rel_str(root, path);
    let session = session_keys(root, &env.session_id);
    let pair = PairText {
        rel: &rel,
        before: &before,
        after: &after,
        lang,
    };
    let policy = cfg.map(|c| Policy::of(root, c)).unwrap_or_default();
    let f = tombstone::measure(&[pair], &session, &policy);
    // the session keys this edit declares again are alive after all
    let declared = tombstone::declared_keys(&after, lang, &policy);
    let revived: Vec<u64> = session
        .intersection(&declared)
        .copied()
        .take(HASH_CAP)
        .collect();
    if f.erased.is_empty() && revived.is_empty() && !f.rows.iter().any(|r| r.names > 0) {
        return None;
    }
    let (tier, budget) = knobs(cfg);
    let judged = judge(root, &f, budget);
    let mut line = tombstone::feed_json(&f, Some(session.len()), &judged);
    line["event"] = serde_json::json!("tombstone");
    line["file"] = serde_json::json!(env.tool_input.file_path);
    line["mode"] = serde_json::json!(tier);
    if !revived.is_empty() {
        line["revived_hashes"] = serde_json::json!(revived);
    }
    let speak = spoken(&f, &judged, tier, budget, fence);
    Some(Pending { line, speak })
}

/// The class's sentence, or nothing: its tier is not observe, a budget
/// is declared (`over` is only ever true under one, knob 0), the core
/// said `over`, and the measurement was whole.
fn spoken(
    f: &tombstone::Findings,
    judged: &Judgment,
    tier: &'static str,
    budget: Option<u32>,
    fence: Option<&str>,
) -> Option<(&'static str, String)> {
    let j = judged.as_ref().ok()?;
    let armed = j.over && tier != TOMBSTONE_DEFAULT && f.degraded_pairs == 0;
    let budget = budget.filter(|_| armed)?;
    let shown: Vec<String> = f.judged_rows(j).take(3).map(Row::place).collect();
    let mut why = super::say::tombstone_over(j.sites.len(), budget, &shown.join("; "));
    if let Some(note) = fence {
        why.push(' ');
        why.push_str(note);
    }
    Some((tier, why))
}

/// The feed line, once the hook has decided at `decided`: `applied` is
/// true when the write goes through, false under deny (that erasure
/// never happened, and session_keys skips the line), null under ask —
/// the person decides, and the hook cannot see what.
pub(super) fn record(root: &Path, env: &Envelope, pending: Option<Pending>, decided: &str) {
    let Some(mut p) = pending else {
        return;
    };
    p.line["applied"] = match decided {
        "deny" => serde_json::json!(false),
        "ask" => serde_json::Value::Null,
        _ => serde_json::json!(true),
    };
    crate::hookio::observe_append(root, Some(&env.session_id), p.line);
}

/// The class's tier and budget as declared (valid by load), or the
/// route defaults when there is no config to read.
fn knobs(cfg: Option<&Config>) -> (&'static str, Option<u32>) {
    let declared = cfg.map_or(TOMBSTONE_DEFAULT, |c| c.tombstone.tier());
    let tier = TIERS
        .iter()
        .find(|t| **t == declared)
        .copied()
        .unwrap_or(TOMBSTONE_DEFAULT);
    (tier, cfg.and_then(|c| c.tombstone.budget))
}

/// The judgment over the daemon-owned core link: only the rows and the
/// budget cross the socket, and every failure is named.
fn judge(root: &Path, f: &tombstone::Findings, budget: Option<u32>) -> Judgment {
    let rows = wire::rows(f);
    match client::request(
        root,
        &Request::Tombstone {
            rows: rows.clone(),
            budget,
        },
    ) {
        Ok(Response::TombstoneReport { reply }) => wire::consume(&reply, rows.len()),
        Ok(other) => Err(format!("daemon answered {other:?}")),
        Err(e) => Err(format!("daemon: {e}")),
    }
}

/// The session's erased keys as its earlier `tombstone` lines left
/// them, folded in feed order: a line's erased keys join the union,
/// the keys it revived (declared again on its after side) leave it,
/// and a line whose write the hook denied (`applied` false) did
/// neither — that erasure never happened. Each list is capped at
/// tombstone::HASH_CAP; the union is as wide as the feed window.
fn session_keys(root: &Path, session: &str) -> BTreeSet<u64> {
    let mut keys = BTreeSet::new();
    for v in crate::hookio::session_lines(root, session) {
        if v["event"] != "tombstone" || v["applied"] == false {
            continue;
        }
        for k in hashes(&v["revived_hashes"]) {
            keys.remove(&k);
        }
        keys.extend(hashes(&v["erased_hashes"]));
    }
    keys
}

/// The u64s of a JSON array (anything else = none).
fn hashes(v: &serde_json::Value) -> impl Iterator<Item = u64> + '_ {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_u64)
}
