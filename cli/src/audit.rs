//! Stop audit (M3 shape, judged over audit/1 since 2.24.0): net LOC
//! of the working tree vs HEAD plus duplicate blocks touching the
//! changed files — deliberately NOT the four-way classification
//! (that is M4); the conviction and the zero-tolerance threshold are
//! the core's (verdict.rs). PreToolUse-gate discipline:
//! fail-open on any internal failure, every run appended to
//! .ce/observe.ndjson. Stop hooks know exactly one enforcement shape
//! (proven by the locally installed cc-enforcer): top-level
//! {"decision":"block","reason":...}; only deny mode uses it. WHAT
//! changed lives in changes.rs (git's paths in ce's own vocabulary).

mod changes;
mod commitmsg;
mod flow;
mod observe;
mod precommit;
mod similar;
mod speech;
mod tombstone;
mod verdict;

use crate::config::Config;
use crate::corelink::Link;
pub use commitmsg::run_commitmsg;
use observe::{AuditEvent, observe_log};
pub use precommit::run_precommit;
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;
use std::process::ExitCode;

#[derive(Deserialize)]
struct Envelope {
    #[serde(default)]
    hook_event_name: String,
    #[serde(default)]
    cwd: String,
    /// Stamped on every hook event; reaches the observe feed so the
    /// M4 evaluation set can be partitioned by session (D2-1/D2-2).
    #[serde(default)]
    session_id: String,
    /// Loop guard: true when a previous Stop hook already blocked.
    #[serde(default)]
    stop_hook_active: bool,
}

/// Entry point for `ce audit --hook`. Never fails outward.
pub fn run_hook() -> ExitCode {
    let Some((env, root)) =
        crate::hookio::gated_envelope("Stop", |e: &Envelope| (&e.hook_event_name, &e.cwd))
    else {
        return ExitCode::SUCCESS;
    };
    // The loop guard skips the AUDIT, not the RECORD: a session whose
    // only Stop landed here used to leave no line at all, which reads
    // to the ledger exactly like a session the hook never ran in.
    if env.stop_hook_active {
        unmeasured_stop(&root, &env.session_id, Some("loop_guard"));
        return ExitCode::SUCCESS;
    }
    // the session root's own files, then every nested project with a
    // gate of its own (plan v2.18 step #12): a seated submodule
    // carrying a ce.toml is audited THERE — its git, its index, its
    // budget — and its verdict rides this Stop under its mount name;
    // a submodule without one is a reader here, measured by nobody
    let mut reasons: Vec<String> = audit(&root, &env.session_id, None).into_iter().collect();
    for mount in crate::gitmodules::gated(&root) {
        reasons.extend(audit(&root.join(&mount), &env.session_id, Some(&mount)));
    }
    if !reasons.is_empty() {
        let payload = serde_json::json!({
            "decision": "block",
            // §4.4 B4: the Stop summary rides its own 400-token budget
            "reason": crate::hookio::clip(
                &reasons.join(" "),
                crate::hookio::STOP_BUDGET_TOKENS,
            ),
        });
        println!("{payload}");
    }
    ExitCode::SUCCESS
}

/// Shared head of the Stop audit and the git-hook faces: guard mode,
/// numstat over `diff`, the core's touched-duplicate verdict (None =
/// degraded, A9f), the informational reports (four-class and the
/// similar advisor are Stop concerns; tombstone rides every leg, and
/// `ce commitmsg`'s `message` rides the tombstone leg as one more
/// surface — the two text legs read ONE git batch of the changed
/// pairs) and the `event`-stamped observe entry (a git hook must not
/// masquerade as a Stop audit). None = git could not answer: fail
/// open.
struct Gathered {
    mode: String,
    net_loc: i64,
    changed: Vec<String>,
    dups: Option<verdict::Verdict>,
    tomb: Option<tombstone::Leg>,
    /// The audit's core link, for what the face then says (speech.rs).
    link: Option<Link>,
}

fn gather(
    root: &Path,
    diff_tail: &[&str],
    event: &str,
    session: Option<&str>,
    message: Option<&str>,
) -> Option<Gathered> {
    // ONE load: tier rendering and the exclusion list both need it,
    // and a broken ce.toml must name itself (config::tier_of).
    let loaded = Config::load(root);
    // The audit class is not §4.2-promoted: unset mode stays observe.
    let mode = crate::config::tier_of(&loaded, "observe");
    let cfg = loaded.as_ref().ok();
    let (mut net_loc, mut changed) = changes::diff(root, diff_tail)?;
    let untracked = stop_untracked(root, event, cfg, &mut net_loc);
    changed.extend_from_slice(&untracked);
    // nothing changed = judged clean without a spawn (INFORMATION
    // never pays; the enforcement leg pays only when there is
    // something to enforce) — one core link for both verdicts
    let mut link = (!changed.is_empty()).then(verdict::open).flatten();
    let dups = if changed.is_empty() {
        Some(verdict::Verdict {
            fail: false,
            dups: 0,
            shown: Vec::new(),
        })
    } else {
        verdict::judge(root, &changed, link.as_mut())
    };
    let fourclass = (event == "stop_audit").then(|| fourclass_report(root));
    let set = tombstone::Changeset {
        event,
        changed: &changed,
        untracked: &untracked,
        message,
    };
    let (tombstone, similar, flow) = text_legs(root, &set, cfg, link.as_mut());
    observe_log(
        root,
        AuditEvent {
            event,
            mode: &mode,
            session,
            net_loc,
            changed: changed.len(),
            dups: dups.as_ref().map(|v| v.dups),
            fourclass,
            tombstone: tombstone.as_ref().map(|t| t.feed.clone()),
            similar,
            flow,
            skipped: None, // this leg MEASURED — see AuditEvent::skipped
            unmeasured: crate::gitmodules::unseated(root),
        },
    );
    Some(Gathered {
        mode,
        net_loc,
        changed,
        dups,
        tomb: tombstone,
        link,
    })
}

/// The legs that measure the changeset's text, read in ONE git batch
/// (none when nothing changed): tombstone on every event, the similar
/// advisor on the Stop alone, flow on every event.
fn text_legs(
    root: &Path,
    set: &tombstone::Changeset,
    cfg: Option<&Config>,
    mut link: Option<&mut Link>,
) -> (Option<tombstone::Leg>, Option<Value>, Option<Value>) {
    let excludes = cfg.map_or(&[][..], |c| c.exclude.as_slice());
    let texts = (!set.changed.is_empty())
        .then(|| tombstone::loaded(root, set, excludes))
        .flatten();
    let tomb = tombstone::leg(root, set, texts.as_ref(), cfg, link.as_deref_mut());
    let similar = texts
        .as_ref()
        .filter(|_| set.event == "stop_audit")
        .and_then(|(loaded, _)| similar::leg(root, loaded, link.as_deref_mut()));
    let flow = texts
        .as_ref()
        .and_then(|(loaded, _)| flow::leg(loaded, link));
    (tomb, similar, flow)
}

/// §4.2 promises the Stop audit covers Bash writes, but `git diff`
/// cannot see a brand-new untracked file — the stop path merges them
/// in, their lines into `net_loc`; the git-hook faces stay staged-only
/// by design (nothing for any other event).
fn stop_untracked(
    root: &Path,
    event: &str,
    cfg: Option<&Config>,
    net_loc: &mut i64,
) -> Vec<String> {
    if event != "stop_audit" {
        return Vec::new();
    }
    let excludes = cfg.map(|c| c.exclude.clone()).unwrap_or_default();
    match changes::untracked(root, &excludes) {
        Some((n, files)) => {
            *net_loc += n;
            files
        }
        None => Vec::new(),
    }
}

/// One project's Stop audit: its observe line always, and the block
/// reasons when a deny tier holds a failing verdict — the duplicate
/// verdict at `[guard] mode`, the tombstone verdict at `[tombstone]
/// tier` — said by the core (speech.rs) under the gated submodule's
/// `mount` when it is one. The caller prints, so a gated submodule's
/// verdict can ride the session's one Stop. A Stop that blocks nothing
/// asks nothing: INFORMATION never pays a spawn.
fn audit(root: &Path, session: &str, mount: Option<&str>) -> Option<String> {
    // Not a git repo: nothing to audit, but the skip is RECORDED.
    let Some(base) = changes::base_rev(root) else {
        unmeasured_stop(root, session, Some("no_git"));
        return None;
    };
    let Some(mut g) = gather(root, &[base.as_str()], "stop_audit", Some(session), None) else {
        unmeasured_stop(root, session, None);
        return None;
    };
    let mut link = g.link.take();
    let said = g.said(speech::Face::Stop, mount);
    if !said.blocked() {
        return None;
    }
    let (lines, fail) = speech::spoken(link.as_mut(), &said);
    let texts: Vec<String> = lines.into_iter().map(|l| l.text).collect();
    fail.then(|| texts.join(" "))
}

/// Every Stop that produces NO measurement, said in the feed instead
/// of passing silently: a Stop that writes nothing reads to the M4
/// ledger exactly like a clean one, and that ledger — counted per
/// SESSION (D2-2) — is the stated gate for promoting this audit to
/// deny. `skipped` names which kind: Some(why) = an early return, the
/// audit never ran; None = git resolved the base but not the diff, a
/// real degradation (A9f). Every root here came through the throat
/// (hookio::gated_envelope), which anchors — an anchorless cwd never
/// reaches this function since batch-8 moved the gate there.
fn unmeasured_stop(root: &Path, session: &str, skipped: Option<&str>) {
    let mode = crate::config::tier_of(&Config::load(root), "observe");
    observe_log(
        root,
        AuditEvent {
            event: "stop_audit",
            mode: &mode,
            session: Some(session),
            net_loc: 0,
            changed: 0,
            // measuring nothing != failing to measure: only the latter
            dups: skipped.map(|_| 0),
            fourclass: None,
            tombstone: None,
            similar: None,
            flow: None,
            skipped,
            unmeasured: Vec::new(),
        },
    );
}

/// M4 four-class summary of the session's working-tree diff via the
/// daemon-owned ce-core link. INFORMATIONAL: no tier reads it, and a
/// promotion is argued from the changeset ledger (R-L2-4, replayed
/// through Judge::judge_changeset — docs/FPR-L2.md), never assumed;
/// `request_if_running` because a Stop must not pay a daemon spawn —
/// cold = a visible degraded field, not latency.
fn fourclass_report(root: &Path) -> serde_json::Value {
    use crate::daemon::{client, proto::Request, proto::Response};
    let Some(pairs) = crate::fourclass::session::head_pairs(root) else {
        return serde_json::json!({"degraded": "no_git"});
    };
    if pairs.is_empty() {
        return serde_json::json!({"degraded": null, "relocations": []});
    }
    match client::request_if_running(root, &Request::FourClass { pairs }) {
        Ok(Response::FourClassReport { report }) => report,
        _ => serde_json::json!({"degraded": "daemon_unavailable"}),
    }
}
