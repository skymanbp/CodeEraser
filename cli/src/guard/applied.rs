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

/// The text the write leaves, and — for an Edit — the line count the
/// file had before it, off the same read (a Write's before side is
/// read only when a rule needs it, and then once: `prior`).
pub(super) struct Applied {
    pub text: String,
    /// Some for an Edit (the read it was applied to), None for a Write.
    pub before: Option<usize>,
    /// A Write's target as it is on disk, read on first need by the
    /// size classes (`before_lines`) or the duplicate class's
    /// subtraction (`prior_text`), never twice in one hook call.
    prior: std::cell::OnceCell<Result<Vec<u8>, std::io::ErrorKind>>,
}

impl Applied {
    /// A Write's target bytes as they are on disk (read once), or the
    /// I/O error kind (`NotFound` = the Write creates the file).
    fn prior(&self, env: &Envelope) -> Result<&[u8], std::io::ErrorKind> {
        let read = || std::fs::read(&env.tool_input.file_path).map_err(|e| e.kind());
        self.prior
            .get_or_init(read)
            .as_deref()
            .map_err(|kind| *kind)
    }

    /// The text a Write replaces, for the duplicate class's
    /// subtraction: None when there is none to read — absent, an I/O
    /// failure, bytes that are not UTF-8 — and an unreadable before
    /// side subtracts nothing (H14: the probe read the disk again and
    /// read an unreadable file as empty, the size classes as an error).
    pub(super) fn prior_text(&self, env: &Envelope) -> Option<&str> {
        self.prior(env)
            .ok()
            .and_then(|b| std::str::from_utf8(b).ok())
    }
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
            prior: std::cell::OnceCell::new(),
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
        prior: std::cell::OnceCell::new(),
    })
}

/// The target's line count before this write: the Edit's own read, or
/// for a Write the count of the file on disk off the one read `prior`
/// keeps — `None` when it does not exist (absent is not empty: the
/// sentence says "creates"), the I/O error otherwise.
pub(super) fn before_lines(
    env: &Envelope,
    applied: &Applied,
) -> Result<Option<usize>, std::io::ErrorKind> {
    if let Some(n) = applied.before {
        return Ok(Some(n));
    }
    match applied.prior(env) {
        Ok(bytes) => super::zone::count_lines_in(bytes)
            .map(Some)
            .map_err(|e| e.kind()),
        Err(std::io::ErrorKind::NotFound) => Ok(None),
        Err(kind) => Err(kind),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/guard/applied.rs"]
mod tests;
