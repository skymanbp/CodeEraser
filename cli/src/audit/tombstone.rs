//! The Stop / precommit leg of the tombstone class (plan v2.26 step 3,
//! judged since v2.27 step 4): the session's changed pairs (working
//! tree vs HEAD, plus the audit's untracked files — a name that moved
//! into a brand-new file is alive, and a new CHANGELOG is a changelog)
//! or the index's (staged vs HEAD: what the commit will hold), their
//! texts in one git batch, the measurement over the whole changeset,
//! its judgment over the audit's core link (tombstone/1), and the
//! `tombstone` object the audit's feed line carries (feed schema
//! 0.9.0). The decision reads two bits — the class's own tier
//! (`[tombstone] tier`) and the core's `over` — and blocks only when
//! both say so; no core = a degraded object, never a block and never
//! a silent pass (A9f). Precommit prints one line for the person.

use crate::config::Config;
use crate::corelink::Link;
use crate::fourclass::session;
use crate::tombstone::texts::{self, Side};
use crate::tombstone::{self, Judged, Judgment, PairText, Policy, Row, wire};
use std::collections::BTreeSet;
use std::path::Path;

/// The leg's outcome for one audit event.
pub(super) struct Leg {
    /// The feed object (tombstone::feed_json without the per-edit keys).
    pub feed: serde_json::Value,
    pub judged: Judgment,
    /// The class's own tier, as declared or the route default.
    pub tier: String,
    pub budget: Option<u32>,
    /// The first judged sites as `file:line kind`.
    pub shown: Vec<String>,
    pub erased: usize,
}

impl Leg {
    /// The deny tier AND the core's condition — one without the other
    /// is a feed entry, not a block.
    pub fn blocks(&self) -> bool {
        self.tier == "deny" && self.judged.as_ref().is_ok_and(|j| j.over)
    }
}

/// The leg as the audit runs it, in the clone verdict's stance: no
/// judged file changed = nothing to pair, measured zero without a
/// spawn; a changeset with no candidate row is judged without a
/// request (the core would answer the same zeros). `[tombstone]`'s
/// four keys come from the audit's one config load (None = a broken
/// or absent ce.toml: nothing declared, observe); `link` = the audit's
/// core link, opened once for both verdicts (None = no core: the
/// judgment is degraded by name).
pub(super) fn leg(
    root: &Path,
    event: &str,
    changed: &[String],
    untracked: &[String],
    cfg: Option<&Config>,
    link: Option<&mut Link>,
) -> Option<Leg> {
    let policy = cfg.map(|c| Policy::of(root, c)).unwrap_or_default();
    let (f, unread) = if changed.is_empty() {
        (tombstone::measure(&[], &BTreeSet::new(), &policy), 0)
    } else {
        measured(root, event, untracked, &policy)?
    };
    let budget = cfg.and_then(|c| c.tombstone.budget);
    let judged = match (f.rows.is_empty(), link) {
        (true, _) => Ok(Judged::default()),
        (false, Some(l)) => wire::judge(l, &f, budget),
        (false, None) => Err("core unavailable".into()),
    };
    let mut feed = tombstone::feed_json(&f, None, &judged);
    if unread > 0 {
        feed["unread_pairs"] = serde_json::json!(unread);
    }
    let shown = judged
        .as_ref()
        .map(|j| f.judged_rows(j).take(10).map(Row::place).collect())
        .unwrap_or_default();
    Some(Leg {
        feed,
        judged,
        tier: cfg.map_or("observe", |c| c.tombstone.tier()).to_string(),
        budget,
        shown,
        erased: f.erased.len(),
    })
}

/// The changeset measured: git pairs the change, one batch reads both
/// sides, the measurement runs over every pair; `unread` = pairs the
/// batch could not read. None = git could not pair the change (a Stop
/// on an unborn HEAD included: there is no before to erase from), and
/// the feed line then carries no `tombstone` key at all.
fn measured(
    root: &Path,
    event: &str,
    untracked: &[String],
    policy: &Policy,
) -> Option<(tombstone::Findings, usize)> {
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
    Some((tombstone::measure(&pairs, &BTreeSet::new(), policy), unread))
}

/// The block reason (deny tier, condition held): the count, the
/// budget it passed, the first sites, and what to do instead.
pub(super) fn reason(leg: &Leg) -> String {
    let sites = leg.judged.as_ref().map_or(0, |j| j.sites.len());
    crate::i18n::line(
        "ce audit: this session's edits leave {} tombstone site(s), past the \
         `[tombstone] budget` of {}: {} — a removed name must not survive as an \
         absence label or an argument from absence; drop the label, or say what \
         replaced it.",
        "ce audit：本会话的编辑留下 {} 处墓碑残留，越过 `[tombstone] budget` 的 {}：{} — \
         被删的名字不该以「无 X」标签或缺席论证留下；去掉标签，或写清替代物。",
        &[&sites, &leg.budget.unwrap_or(0), &leg.shown.join("; ")],
    )
}

/// The precommit terminal line: the judged sites when there are any,
/// the degradation when there is no verdict, nothing when the
/// changeset is clean — the feed has the rest.
pub(super) fn summary(leg: &Leg) -> Option<String> {
    match &leg.judged {
        Err(why) => Some(crate::i18n::line(
            "ce precommit: tombstone verdict unavailable (DEGRADED: {})",
            "ce precommit：墓碑残留判决不可用（已降级：{}）",
            &[why],
        )),
        Ok(j) if j.sites.is_empty() => None,
        Ok(j) => Some(crate::i18n::line(
            "ce precommit: {} tombstone site(s) — {} label / {} prose over {} erased name(s): {} \
             (tier {}; see .ce/observe.ndjson)",
            "ce precommit：{} 处墓碑残留 — 标签 {} / 散文 {}，涉及 {} 个被删名字：{}\
             （档位 {}；详见 .ce/observe.ndjson）",
            &[
                &j.sites.len(),
                &j.label,
                &j.prose,
                &leg.erased,
                &leg.shown.join("; "),
                &leg.tier,
            ],
        )),
    }
}
