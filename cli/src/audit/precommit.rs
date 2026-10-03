//! `ce precommit` / `ce commitmsg`: the git-hook faces of the audit —
//! STAGED changes only, printed for a person (they run in a terminal,
//! not a hook), and the commit refused (exit 1) when a deny-tier
//! verdict holds: `[guard] mode` deny over touched duplicates, or
//! `[tombstone] tier` deny over the class's budget. The two faces are
//! one body; commitmsg hands in the message as one more surface
//! (commitmsg.rs). The count is the JUDGED staged set (`changes::diff`'s
//! universe): a staged `.css` can never match a dedup block either.

use super::gather;
use super::speech::{self, Face, Said};
use crate::document::lines::{Mode, print};
use std::path::Path;
use std::process::ExitCode;

/// The pre-commit face: the staged set, no message.
pub fn run_precommit(root: &Path) -> ExitCode {
    run(root, "precommit", None)
}

/// One gather over `--cached`, then what the face says — the
/// tombstone summary and its reason when it blocks, the duplicate
/// verdict or the staged summary, or that this is no git repository —
/// and the exit code, both the core's (speech.rs). `face` names the
/// speaker and stamps the feed's `event`.
pub(super) fn run(root: &Path, face: &str, message: Option<&str>) -> ExitCode {
    // session = None honestly: the git hooks run in a terminal, no
    // session owns them — the M4 sampler excludes non-session events.
    // `--cached` needs no base rev: git compares the index against an
    // unborn HEAD without complaint, unlike the Stop leg.
    let mut gathered = gather(root, &["--cached"], face, None, message);
    let mut link = gathered.as_mut().and_then(|g| g.link.take());
    let said = gathered.as_ref().map_or_else(
        || Said::bare(Face::of(face)),
        |g| g.said(Face::of(face), None),
    );
    let (lines, fail) = speech::spoken(link.as_mut(), &said);
    print(&lines, Mode::Console);
    if fail {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
