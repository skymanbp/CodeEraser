//! `ce commitmsg <file>`: the commit-msg hook's face (plan v2.27 step
//! 5) — the pre-commit gate re-run with the message git hands the hook
//! as one more surface. A removed name argued away in the message ("X
//! is no longer needed") is a tombstone site like one in a README: the
//! message is Markdown prose named `COMMIT_EDITMSG` in the feed and in
//! the reason (tombstone::MESSAGE). Wire it as `.git/hooks/commit-msg`
//! running `ce commitmsg "$1"`; a PR body saved to a file is the same
//! surface — a CI recipe, not a leg. Only the message is read here;
//! what it measures lives in tombstone.rs, the body in precommit.rs.

use std::path::Path;
use std::process::ExitCode;

/// The face: the message file, its comment lines blanked, into the
/// git-hook body. An unreadable file is a usage error (2), never a
/// pass: the hook was handed a path, and a gate that cannot see its
/// input must say so.
pub fn run_commitmsg(root: &Path, file: &Path) -> ExitCode {
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "{}",
                crate::i18n::line(
                    "ce commitmsg: cannot read {}: {}",
                    "ce commitmsg：读不了 {}：{}",
                    &[&file.display(), &e],
                )
            );
            return ExitCode::from(2);
        }
    };
    let message = uncommented(&text, &comment_prefix(root));
    super::precommit::run(root, "commitmsg", Some(&message))
}

/// What a comment line of the message starts with: the repository's
/// `core.commentChar` / `core.commentString` — aliases of each other,
/// the last one set wins, as git itself reads them — and `#` when
/// neither is set. `auto` reads as `#`, which is what git picks unless
/// a message line already begins with `#`; its pick then is recorded
/// nowhere this hook can read, and those comment lines are measured
/// as the prose they look like.
fn comment_prefix(root: &Path) -> String {
    let out = crate::proc::git_output(root, &["config", "--get-regexp", "^core\\.comment"]);
    let text = out
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let last = text
        .lines()
        .rev()
        .find_map(|l| l.split_once(' ').map(|(_, v)| v.trim().to_string()));
    match last {
        Some(v) if !v.is_empty() && v != "auto" => v,
        _ => "#".into(),
    }
}

/// The message with every comment line blanked — blanked, not removed,
/// so a site's line is the file's own line (git strips them itself
/// only after this hook has run).
fn uncommented(text: &str, comment: &str) -> String {
    text.lines()
        .map(|l| if l.starts_with(comment) { "" } else { l })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
#[path = "../../tests/unit/audit/commitmsg.rs"]
mod tests;
