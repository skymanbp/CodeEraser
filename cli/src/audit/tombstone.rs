//! The Stop / precommit leg of the tombstone measurement (plan v2.26
//! step 3): the session's changed pairs (working tree vs HEAD, plus
//! the audit's untracked files — a name that moved into a brand-new
//! file is alive, and a new CHANGELOG is a changelog) or the index's
//! (staged vs HEAD: what the commit will hold), their texts in one
//! git batch, the measurement over the whole changeset, and the
//! additive `tombstone` object the audit's feed line carries (feed
//! schema 0.8.0). Purely informational: no block reason ever reads
//! it — stage one has no FPR ledger yet, and this leg is what writes
//! that ledger's raw material. Precommit, which prints for a person,
//! adds one line when something fired.

use crate::fourclass::session;
use crate::tombstone::texts::{self, Side};
use crate::tombstone::{self, PairText};
use std::collections::BTreeSet;
use std::path::Path;

/// The changeset's measurement for one audit event; `untracked` = the
/// Stop audit's untracked leg (already scoped and judged), empty for
/// precommit. None = git could not pair the change (a Stop on an
/// unborn HEAD included: there is no before to erase from), and the
/// feed line then carries no `tombstone` key at all.
pub(super) fn report(root: &Path, event: &str, untracked: &[String]) -> Option<serde_json::Value> {
    let (mut pairs, after) = if event == "precommit" {
        (session::scoped_pairs(root, &["--cached"])?, Side::Index)
    } else {
        (session::scoped_pairs(root, &["HEAD"])?, Side::Worktree)
    };
    pairs.extend(untracked.iter().map(|f| (None, Some(f.clone()))));
    let (loaded, unread) = texts::load(root, &pairs, Side::Rev("HEAD"), after)?;
    let pairs: Vec<PairText> = loaded
        .iter()
        .map(|l| PairText {
            rel: &l.rel,
            before: &l.before,
            after: &l.after,
            lang: l.lang,
        })
        .collect();
    let mut line = tombstone::feed_json(&tombstone::measure(&pairs, &BTreeSet::new()), None);
    if unread > 0 {
        line["unread_pairs"] = serde_json::json!(unread);
    }
    Some(line)
}

/// The measurement of a changeset with no judged file in it — the
/// audit already knows that from its numstat, so this leg pairs
/// nothing and spawns nothing.
pub(super) fn nothing() -> serde_json::Value {
    tombstone::feed_json(&tombstone::measure(&[], &BTreeSet::new()), None)
}

/// The precommit terminal line, only when a site fired — the person
/// at the keyboard gets the counts and the first sites; the feed has
/// the rest.
pub(super) fn summary(report: &serde_json::Value) -> Option<String> {
    let count = |k: &str| report[k].as_u64().unwrap_or(0);
    let (label, prose) = (count("label"), count("prose"));
    if label + prose == 0 {
        return None;
    }
    let sites: Vec<&str> = report["sites"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect();
    Some(crate::i18n::line(
        "ce precommit: {} tombstone site(s) — {} label / {} prose over {} erased name(s): {} \
         (measured only; see .ce/observe.ndjson)",
        "ce precommit：{} 处墓碑残留 — 标签 {} / 散文 {}，涉及 {} 个被删名字：{}\
         （仅度量；详见 .ce/observe.ndjson）",
        &[
            &(label + prose),
            &label,
            &prose,
            &count("erased"),
            &sites.join("; "),
        ],
    ))
}
