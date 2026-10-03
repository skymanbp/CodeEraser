//! The Stop / precommit / commitmsg leg of the tombstone class (plan
//! v2.26 step 3, judged since v2.27 step 4): the session's changed
//! pairs (working tree vs HEAD, plus the audit's untracked files — a
//! name that moved into a brand-new file is alive, and a new CHANGELOG
//! is a changelog) or the index's (staged vs HEAD: what the commit
//! will hold), their texts in one git batch, the measurement over the
//! whole changeset — `ce commitmsg` adds the message as one more
//! surface — its judgment over the audit's core link (tombstone/1),
//! and the `tombstone` object the audit's feed line carries (feed
//! schema 0.9.0). The decision reads two bits — the class's own tier
//! (`[tombstone] tier`) and the core's `over` — and blocks only when
//! both say so; no core = a degraded object, never a block and never
//! a silent pass (A9f). What the faces say about it is the core's
//! (speech.rs).

use crate::config::Config;
use crate::corelink::Link;
use crate::fourclass::session;
use crate::scan::lang::Lang;
use crate::scan::walk::Scope;
use crate::tombstone::texts::{self, Loaded, Side};
use crate::tombstone::{self, Judged, Judgment, PairText, Policy, Row, wire};
use std::collections::BTreeSet;
use std::path::Path;

/// The message surface's name in a site (`COMMIT_EDITMSG:line prose`):
/// git's own name for the file it hands the commit-msg hook.
pub const MESSAGE: &str = "COMMIT_EDITMSG";

/// What one audit event measures: the event (`stop_audit` pairs the
/// working tree, the git-hook faces pair the index), the diff's
/// changed files, the untracked files the Stop merges in, and — `ce
/// commitmsg` only — the message, measured as one Markdown
/// after-surface.
pub(super) struct Changeset<'a> {
    pub event: &'a str,
    pub changed: &'a [String],
    pub untracked: &'a [String],
    pub message: Option<&'a str>,
}

/// The leg's outcome for one audit event.
pub(super) struct Leg {
    /// The feed object (tombstone::feed_json without the per-edit keys).
    pub feed: serde_json::Value,
    pub judged: Judgment,
    /// The class's own tier, as declared or the route default.
    pub tier: String,
    pub budget: Option<u32>,
    /// The first judged sites.
    pub shown: Vec<Row>,
    pub erased: usize,
    /// Why the measurement was not whole: pairs the batch could not
    /// read, pairs whose line diff was bounded. An incomplete
    /// measurement is recorded, never enforced: the class cannot know
    /// what it did not read, or what a bounded diff read as written.
    pub unread: usize,
    pub bounded: usize,
}

impl Leg {
    /// The deny tier AND the core's condition AND a whole measurement —
    /// anything less is a feed entry, not a block.
    pub fn blocks(&self) -> bool {
        self.unread + self.bounded == 0
            && self.tier == "deny"
            && self.judged.as_ref().is_ok_and(|j| j.over)
    }
}

/// The leg as the audit runs it, in the clone verdict's stance: no
/// judged file changed = nothing to pair, measured zero without a
/// spawn (nothing changed erases nothing, so a message alone binds
/// nothing); a changeset with no candidate row is judged without a
/// request (the core would answer the same zeros). `[tombstone]`'s
/// four keys come from the audit's one config load (None = a broken
/// or absent ce.toml: nothing declared, observe); `link` = the audit's
/// core link, opened once for both verdicts (None = no core: the
/// judgment is degraded by name).
pub(super) fn leg(
    root: &Path,
    set: &Changeset,
    texts: Option<&(Vec<Loaded>, usize)>,
    cfg: Option<&Config>,
    link: Option<&mut Link>,
) -> Option<Leg> {
    let policy = cfg.map(|c| Policy::of(root, c)).unwrap_or_default();
    let (f, unread) = match (set.changed.is_empty(), texts) {
        (true, _) => (tombstone::measure(&[], &BTreeSet::new(), &policy), 0),
        (false, Some((loaded, unread))) => (measured(loaded, set, &policy), *unread),
        (false, None) => return None,
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
        .map(|j| f.judged_rows(j).take(10).cloned().collect())
        .unwrap_or_default();
    Some(Leg {
        feed,
        judged,
        tier: cfg.map_or("observe", |c| c.tombstone.tier()).to_string(),
        budget,
        shown,
        erased: f.erased.len(),
        unread,
        bounded: f.degraded_pairs,
    })
}

/// The changeset's texts, read ONCE for every leg that measures text:
/// git pairs the change, the config's walk scope keeps the pairs it
/// would measure (an excluded path is nobody's, as the guard leg reads
/// it), one batch reads both sides; `unread` = pairs the batch could
/// not read. None = git could not pair the change (a Stop on an unborn
/// HEAD included: there is no before to erase from), and the feed line
/// then carries no `tombstone` and no `similar` key at all.
pub(super) fn loaded(
    root: &Path,
    set: &Changeset,
    excludes: &[String],
) -> Option<(Vec<Loaded>, usize)> {
    let (mut pairs, after) = if set.event == "stop_audit" {
        (session::scoped_pairs(root, &["HEAD"])?, Side::Worktree)
    } else {
        (session::scoped_pairs(root, &["--cached"])?, Side::Index)
    };
    pairs.extend(set.untracked.iter().map(|f| (None, Some(f.clone()))));
    if let Ok(mut scope) = Scope::new(root, excludes) {
        pairs.retain(|(b, a)| {
            let path = a.as_deref().or(b.as_deref()).unwrap_or_default();
            scope.contains(Path::new(path))
        });
    }
    texts::load(root, &pairs, Side::Rev("HEAD"), after)
}

/// The measurement over every loaded pair — plus the message as an
/// after-only SURFACE (tombstone::measure_with: it offers rows, keeps
/// no name alive) when there is one.
fn measured(loaded: &[Loaded], set: &Changeset, policy: &Policy) -> tombstone::Findings {
    let pairs: Vec<PairText> = loaded
        .iter()
        .map(|l| PairText {
            rel: &l.rel,
            before: &l.before,
            after: &l.after,
            lang: l.lang,
        })
        .collect();
    // the message is all "after": it existed nowhere before the commit,
    // and it is no file — a surface, never a side that declares
    let message: Vec<PairText> = set
        .message
        .map(|after| PairText {
            rel: MESSAGE,
            before: "",
            after,
            lang: Lang::Markdown,
        })
        .into_iter()
        .collect();
    tombstone::measure_with(&pairs, &message, &BTreeSet::new(), policy)
}
