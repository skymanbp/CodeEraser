//! The PreToolUse leg of the tombstone class (plan v2.26 step 4,
//! judged since v2.27 step 4): this edit's (on-disk, applied) pair,
//! measured against the names it erases AND the keys earlier edits of
//! the same session erased — the observe feed is the accumulator, as
//! it is for the warn suppression (hookio::session_lines) — then
//! judged over the daemon's core link (tombstone/1) and recorded. A
//! `tombstone` line lands only when there is something to judge: this
//! edit erased a name, or one of its surfaces bound a name the session
//! erased before. The class speaks at its OWN tier (`[tombstone]
//! tier`, default observe) and only when the core says the declared
//! budget is exceeded; no core = a degraded line, never a decision.

use super::envelope::Envelope;
use crate::config::{Config, TIERS, TOMBSTONE_DEFAULT};
use crate::daemon::client;
use crate::daemon::proto::{Request, Response};
use crate::tombstone::{self, Judgment, PairText, Policy, Row, wire};
use std::collections::BTreeSet;
use std::path::Path;

/// Measure, judge and record one Write/Edit; the class's reason at its
/// own tier when the condition holds. Scope is the budget rule's (a
/// judged language, inside the config's walk) and the applied text is
/// the budget rule's too (budget::resulting_text) — a tool call that
/// is failing on its own measures nothing.
pub(super) fn observe(
    root: &Path,
    env: &Envelope,
    cfg: Option<&Config>,
) -> Option<(&'static str, String)> {
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
    if f.erased.is_empty() && !f.rows.iter().any(|r| r.names > 0) {
        return None;
    }
    let (tier, budget) = knobs(cfg);
    let judged = judge(root, &f, budget);
    let mut line = tombstone::feed_json(&f, Some(session.len()), &judged);
    line["event"] = serde_json::json!("tombstone");
    line["file"] = serde_json::json!(env.tool_input.file_path);
    line["mode"] = serde_json::json!(tier);
    crate::hookio::observe_append(root, Some(&env.session_id), line);
    let j = judged.ok()?;
    let shown: Vec<String> = f.judged_rows(&j).take(3).map(Row::place).collect();
    // `over` is only ever true under a declared budget (knob 0)
    let budget = budget.filter(|_| j.over && tier != TOMBSTONE_DEFAULT)?;
    Some((
        tier,
        super::say::tombstone_over(j.sites.len(), budget, &shown.join("; ")),
    ))
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

/// The union of every erased key this session's earlier `tombstone`
/// lines recorded (each capped at tombstone::HASH_CAP; the union is
/// as wide as the feed window).
fn session_keys(root: &Path, session: &str) -> BTreeSet<u64> {
    crate::hookio::session_lines(root, session)
        .iter()
        .filter(|v| v["event"] == "tombstone")
        .flat_map(|v| {
            let hashes = v["erased_hashes"].as_array().into_iter().flatten();
            hashes
                .filter_map(serde_json::Value::as_u64)
                .collect::<Vec<_>>()
        })
        .collect()
}
