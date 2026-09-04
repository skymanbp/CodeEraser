//! The PreToolUse leg of the tombstone measurement (plan v2.26 step
//! 4), feed-only in every tier: this edit's (on-disk, applied) pair,
//! measured against the names it erases AND the keys earlier edits of
//! the same session erased — the observe feed is the accumulator, as
//! it is for the warn suppression (hookio::session_lines). A
//! `tombstone` line lands only when there is something to say: this
//! edit erased a name, or one of its frames or marks bound a name the
//! session erased before. Never a decision: stage one has no FPR
//! ledger, and this leg is what writes its raw material.

use super::envelope::Envelope;
use crate::config::Config;
use crate::tombstone::{self, PairText, Policy};
use std::collections::BTreeSet;
use std::path::Path;

/// Measure and record one Write/Edit. Scope is the budget rule's
/// (a judged language, inside the config's walk) and the applied text
/// is the budget rule's too (budget::resulting_text) — a tool call
/// that is failing on its own measures nothing.
pub(super) fn observe(root: &Path, env: &Envelope, mode: &str, cfg: Option<&Config>) {
    let path = Path::new(&env.tool_input.file_path);
    let Some(lang) = crate::scan::lang::Lang::judged_path(path) else {
        return;
    };
    if cfg.is_some_and(|c| !crate::scan::walk::in_scope(root, path, &c.exclude)) {
        return;
    }
    let Some(after) = super::budget::resulting_text(env) else {
        return;
    };
    let Some(before) =
        tombstone::texts::read_capped(path).or_else(|| (!path.exists()).then(String::new))
    else {
        return;
    };
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
    if f.erased.is_empty() && f.label + f.prose == 0 {
        return;
    }
    let mut line = tombstone::feed_json(&f, Some(session.len()));
    line["event"] = serde_json::json!("tombstone");
    line["file"] = serde_json::json!(env.tool_input.file_path);
    line["mode"] = serde_json::json!(mode);
    crate::hookio::observe_append(root, Some(&env.session_id), line);
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
