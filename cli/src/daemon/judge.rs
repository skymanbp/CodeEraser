//! Daemon-owned judgment channel (ADR-003: ce-core is the DAEMON's
//! long-lived child, never the short-lived hook's). Holds the
//! corelink across requests with a restart budget: after
//! MAX_FAILURES consecutive open/request failures the daemon stops
//! retrying for its lifetime and reports degraded — visible,
//! never a storm (R-L2-8).

use crate::corelink::Link;
use crate::fourclass::batch::{PairInput, classify_batch};
use crate::fourclass::session;
use crate::scan::lang::Lang;
use std::path::Path;

#[derive(Default)]
pub struct Judge {
    link: Option<Link>,
    failures: u8,
}

const MAX_FAILURES: u8 = 3;

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
            .filter_map(|p| {
                load_pair(root, p.0.as_deref(), p.1.as_deref()).map(|t| (p.clone(), t))
            })
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
        } else if batch.degraded.is_none() {
            self.failures = 0;
        }
        session::report_json(&batch, &sent)
    }

    fn link_mut(&mut self) -> Option<&mut Link> {
        if self.link.is_none() && self.failures < MAX_FAILURES {
            match core_bin().and_then(|bin| Link::open(&bin).ok()) {
                Some((link, _reply)) => self.link = Some(link),
                None => self.note_failure(),
            }
        }
        self.link.as_mut()
    }

    fn note_failure(&mut self) {
        self.link = None; // a failed link is dead; retry within budget
        self.failures = self.failures.saturating_add(1);
        if self.failures == MAX_FAILURES {
            eprintln!(
                "ce daemon: ce-core unavailable after {MAX_FAILURES} attempts — L1 for this session"
            );
        }
    }
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
    let lang = Lang::from_path(Path::new(after.or(before)?))?;
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
