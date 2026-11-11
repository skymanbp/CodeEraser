//! The write as it would land, worked out ONCE per hook call (v2.35
//! guard lane, F3): the size classes, the flow leg and the tombstone
//! leg each read the target and re-applied the Edit on their own —
//! three reads of one file per Edit, and three chances to disagree.
//! Write is the payload itself; Edit is the payload applied to the
//! on-disk file under Edit's own semantics — CRLF-normalized, and a
//! single replacement requires a UNIQUE match exactly as the real tool
//! does (attack review F11: replacen on a non-unique old_string would
//! judge a write that will never land).
//!
//! An Edit that cannot apply says WHY (GD3): the target is absent (the
//! tool fails on its own), unreadable (an I/O failure the hook should
//! not have met — a degraded stamp), or `old_string` matches nothing or
//! more than once. The legs that need the text stay silent then; the
//! probe line carries the reason.

use super::envelope::Envelope;
use std::path::Path;

/// The text the write leaves, and — for an Edit — the line count the
/// file had before it, off the same read (a Write's before side is
/// counted only when a rule needs it: `before_lines`).
pub(super) struct Applied {
    pub text: String,
    /// Some for an Edit (the read it was applied to), None for a Write.
    pub before: Option<usize>,
}

/// Why an Edit has no resulting text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Unapplied {
    /// The target does not exist: the Edit fails on its own.
    Absent,
    /// Any other I/O failure reading the target (permission, a
    /// directory at the path, bytes that are not UTF-8).
    Unreadable(std::io::ErrorKind),
    /// `old_string` is empty or occurs nowhere.
    NoMatch,
    /// `old_string` occurs more than once without `replace_all`.
    Ambiguous,
}

impl Unapplied {
    /// The feed spelling (`unapplied` on the probe line).
    pub(super) fn code(self) -> &'static str {
        match self {
            Unapplied::Absent => "absent",
            Unapplied::Unreadable(_) => "unreadable",
            Unapplied::NoMatch => "no_match",
            Unapplied::Ambiguous => "ambiguous",
        }
    }
}

/// The write applied in memory.
pub(super) fn apply(env: &Envelope) -> Result<Applied, Unapplied> {
    let t = &env.tool_input;
    if env.tool_name == "Write" {
        return Ok(Applied {
            text: t.content.clone(),
            before: None,
        });
    }
    let raw = std::fs::read_to_string(&t.file_path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => Unapplied::Absent,
        kind => Unapplied::Unreadable(kind),
    })?;
    let before = raw.lines().count();
    let on_disk = raw.replace("\r\n", "\n");
    let (old, new) = (
        t.old_string.replace("\r\n", "\n"),
        t.new_string.replace("\r\n", "\n"),
    );
    if old.is_empty() {
        return Err(Unapplied::NoMatch);
    }
    let text = match (t.replace_all, on_disk.matches(&old).count()) {
        (_, 0) => return Err(Unapplied::NoMatch),
        (false, 1) => on_disk.replacen(&old, &new, 1),
        (false, _) => return Err(Unapplied::Ambiguous), // the real Edit rejects ambiguity
        (true, _) => on_disk.replace(&old, &new),
    };
    Ok(Applied {
        text,
        before: Some(before),
    })
}

/// The target's line count before this write: the Edit's own read, or
/// for a Write a streaming count of the file on disk — `None` when it
/// does not exist (absent is not empty: the sentence says "creates"),
/// the I/O error otherwise.
pub(super) fn before_lines(
    env: &Envelope,
    applied: &Applied,
) -> Result<Option<usize>, std::io::ErrorKind> {
    if let Some(n) = applied.before {
        return Ok(Some(n));
    }
    match count_lines(Path::new(&env.tool_input.file_path)) {
        Ok(n) => Ok(Some(n)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.kind()),
    }
}

/// A file's line count from its bytes, streamed (zone::count_lines_in).
fn count_lines(path: &Path) -> std::io::Result<usize> {
    super::zone::count_lines_in(std::fs::File::open(path)?)
}

#[cfg(test)]
#[path = "../../tests/unit/guard/applied.rs"]
mod tests;
