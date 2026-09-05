//! Daemon-owned judgment channel (ADR-003: ce-core is the DAEMON's
//! long-lived child, never the short-lived hook's). Holds the
//! corelink across requests behind a RETRY BUDGET that never closes
//! (7.0.0, O63): each consecutive open/request failure doubles the
//! wait before the next spawn attempt (1 s, 2 s, … capped at
//! BACKOFF_CAP), so a broken core costs at most one spawn a minute and
//! never a storm (R-L2-8) — and a core that comes back (an install that
//! finished, a repaired PATH) is picked up at the next attempt instead
//! of staying L1 for the daemon's lifetime, which is what the retired
//! three-strikes budget did. The recovery is VISIBLE: the first
//! classify report after it carries `recovered: <failed attempts>`,
//! which reaches the observe feed with the rest of the report. "Visible"
//! never means stderr: that handle is null whenever the daemon was
//! lazily spawned (client.rs gives it Stdio::null), so the lines below
//! are a courtesy for a foreground daemon, not the channel the A9f
//! promise rests on.

use crate::corelink::Link;
use crate::fourclass::batch::{PairInput, classify_batch};
use crate::fourclass::session;
use crate::scan::lang::Lang;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Judge {
    link: Option<Link>,
    budget: Budget,
    /// Failed attempts the last successful open recovered from, until
    /// the next classify report carries the number.
    recovered: Option<u32>,
}

/// First wait after a failure; each further consecutive failure
/// doubles it.
const BACKOFF_BASE: Duration = Duration::from_secs(1);
/// The longest wait between two spawn attempts.
const BACKOFF_CAP: Duration = Duration::from_secs(60);

/// The retry policy, pure over `Instant`s so a battery can drive it
/// without spawning anything.
#[derive(Default)]
pub(crate) struct Budget {
    failures: u32,
    retry_at: Option<Instant>,
}

impl Budget {
    /// May an attempt be made now? Always, until a failure has set a
    /// wait; then only once the wait has run out.
    pub(crate) fn open(&self, now: Instant) -> bool {
        self.retry_at.is_none_or(|at| now >= at)
    }

    /// Record one failure; returns the wait before the next attempt.
    pub(crate) fn failed(&mut self, now: Instant) -> Duration {
        self.failures = self.failures.saturating_add(1);
        let wait = Self::backoff(self.failures);
        self.retry_at = Some(now + wait);
        wait
    }

    /// Record a successful open: Some(failed attempts) when the link
    /// had been lost, None when nothing was ever wrong.
    pub(crate) fn recovered(&mut self) -> Option<u32> {
        self.retry_at = None;
        let n = std::mem::take(&mut self.failures);
        (n > 0).then_some(n)
    }

    /// BACKOFF_BASE · 2^(failures−1), capped — the exponent saturates
    /// so a daemon that has failed for a year still computes.
    pub(crate) fn backoff(failures: u32) -> Duration {
        let shift = failures.saturating_sub(1).min(16);
        BACKOFF_CAP.min(BACKOFF_BASE * (1u32 << shift))
    }
}

impl Judge {
    /// Classify the given (before, after) path pairs of `root`'s
    /// working tree vs HEAD. Always returns a report; every failure
    /// shape is a degraded report, never an error (fail-open, A9f).
    pub fn classify(
        &mut self,
        root: &Path,
        pairs: &[(Option<String>, Option<String>)],
    ) -> serde_json::Value {
        // ONE index space: reply indices point into what we SENT, and
        // load_pair DROPS pairs — the unfiltered list misnamed them.
        let kept: Vec<(session::PathPair, (String, String, Lang))> = pairs
            .iter()
            .filter_map(|p| load_pair(root, p.0.as_deref(), p.1.as_deref()).map(|t| (p.clone(), t)))
            .collect();
        let sent: Vec<session::PathPair> = kept.iter().map(|(p, _)| p.clone()).collect();
        let inputs: Vec<PairInput> = kept
            .iter()
            .map(|(_, (b, a, lang))| PairInput {
                before: b,
                after: a,
                lang: *lang,
            })
            .collect();
        let batch = classify_batch(&inputs, self.link_mut());
        // Keyed on the LINK, not on getting a verdict: a core refusing
        // via its own work budget is HEALTHY, killing it cost a retry.
        if batch.link_failed {
            self.note_failure();
        }
        let mut report = session::report_json(&batch, &sent);
        if let Some(n) = self.recovered.take() {
            report["recovered"] = serde_json::json!(n);
        }
        report
    }

    /// The tombstone verdict over the daemon-owned link: the raw
    /// tombstone.result (the hook re-labels its indices), or a degraded
    /// object naming why there is none. The failure budget is
    /// classify's: a dead link counts, a core without the family does
    /// not (it is healthy, only older than this ce).
    pub fn tombstone(&mut self, rows: &[[u64; 3]], budget: Option<u32>) -> serde_json::Value {
        use crate::tombstone::wire;
        let Some(link) = self.link_mut() else {
            return degraded("core_unavailable");
        };
        if !link.has(wire::CAP) {
            return degraded("pre-6.6.0 core");
        }
        match link.request(wire::KIND, wire::body(rows, budget)) {
            Ok(reply) => reply,
            Err(why) => {
                self.note_failure();
                degraded(&why)
            }
        }
    }

    /// Drop the core link NOW (its Drop kills + reaps the ce-core
    /// child). The whole-daemon exits run `std::process::exit` from a
    /// connection thread (server/dispatch.rs), which skips destructors
    /// — without this, every shutdown/skew orphaned a ce-core process.
    pub fn retire_link(&mut self) {
        self.link = None;
    }

    fn link_mut(&mut self) -> Option<&mut Link> {
        if self.link.is_none() && self.budget.open(Instant::now()) {
            match core_bin().and_then(|bin| Link::open(&bin).ok()) {
                Some((link, _reply)) => {
                    self.link = Some(link);
                    if let Some(n) = self.budget.recovered() {
                        self.recovered = Some(n);
                        eprintln!("ce daemon: ce-core back after {n} failed attempts");
                    }
                }
                None => self.note_failure(),
            }
        }
        self.link.as_mut()
    }

    fn note_failure(&mut self) {
        self.link = None; // a failed link is dead; the budget times the retry
        let wait = self.budget.failed(Instant::now());
        eprintln!(
            "ce daemon: ce-core unavailable (attempt {}) — next try in {} s",
            self.budget.failures,
            wait.as_secs()
        );
    }
}

/// A verdict the daemon could not obtain, said in the reply's own
/// vocabulary (wire::consume reads it as a named non-judgment).
fn degraded(reason: &str) -> serde_json::Value {
    serde_json::json!({"degraded": true, "reason": reason})
}

/// CE_CORE_BIN, else a ce-core sibling of this binary, else PATH —
/// the daemon's resolver, reused by the MCP server (one authority).
pub(crate) fn core_bin() -> Option<String> {
    if let Ok(bin) = std::env::var("CE_CORE_BIN") {
        return Some(bin);
    }
    let exe = std::env::current_exe().ok()?;
    let sibling = exe.with_file_name(if cfg!(windows) {
        "ce-core.exe"
    } else {
        "ce-core"
    });
    if sibling.exists() {
        return Some(sibling.display().to_string());
    }
    Some("ce-core".into())
}

/// Before = HEAD content, after = working tree; a side with no path
/// (created/deleted) is empty. Pairs outside the supported languages
/// (or unreadable) are skipped — they carry no classifiable lines.
///
/// The after path comes off the unauthenticated DAEMON SOCKET, so it
/// is CONFINED before any read (fail-closed); git confines the before.
fn load_pair(
    root: &Path,
    before: Option<&str>,
    after: Option<&str>,
) -> Option<(String, String, Lang)> {
    // judged_path: the scan-only arm (plan v2.5) is never classified
    let lang = Lang::judged_path(Path::new(after.or(before)?))?;
    let before_text = match before {
        None => String::new(),
        Some(p) => head_content(root, p)?,
    };
    let after_text = match after {
        None => String::new(),
        Some(p) => std::fs::read_to_string(crate::scan::walk::contained(root, p)?).ok()?,
    };
    Some((before_text, after_text, lang))
}

fn head_content(root: &Path, path: &str) -> Option<String> {
    session::git_stdout(root, &["show", &format!("HEAD:{path}")])
}

#[cfg(test)]
#[path = "../../tests/unit/daemon/judge.rs"]
mod tests;
