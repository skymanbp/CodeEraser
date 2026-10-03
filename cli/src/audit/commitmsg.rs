//! `ce commitmsg <file>`: the commit-msg hook's face (plan v2.27 step
//! 5) — the pre-commit gate re-run with the message git hands the hook
//! as one more surface. A removed name argued away in the message ("X
//! is no longer needed") is a tombstone site like one in a README: the
//! message is Markdown prose named `COMMIT_EDITMSG` in the feed and in
//! the reason (tombstone::MESSAGE). Wire it as `.git/hooks/commit-msg`
//! running `ce commitmsg "$1"`; a PR body saved to a file is the same
//! surface — a CI recipe, not a leg. Only the message is read here;
//! what it measures lives in tombstone.rs, the body in precommit.rs.

use super::speech::{self, Face, Said};
use crate::document::lines::{Mode, print};
use std::path::Path;
use std::process::ExitCode;

/// The face: the message file, its comment lines blanked, into the
/// git-hook body. An unreadable file — absent, binary, or past
/// texts::READ_CAP: the hook's input is read bounded, as every other
/// side is — is a usage error (2), never a pass: the hook was handed a
/// path, and a gate that cannot see its input must say so (the
/// sentence is the core's, speech.rs; the exit code is this side's).
pub fn run_commitmsg(root: &Path, file: &Path) -> ExitCode {
    let Some(text) = crate::tombstone::texts::read_capped(file) else {
        let said = Said {
            unreadable: Some(file.display().to_string()),
            ..Said::bare(Face::Commitmsg)
        };
        print(&speech::spoken(None, &said).0, Mode::Console);
        return ExitCode::from(2);
    };
    let message = uncommented(&text, &comment_prefix(root));
    super::precommit::run(root, "commitmsg", Some(&message))
}

/// What blanks a comment line of the message: the repository's
/// `core.commentChar` / `core.commentString` value byte for byte —
/// aliases of each other, the last one set wins, as git itself reads
/// them — `#` when neither is set, and `Auto` for the deprecated
/// `auto` (git 2.52 warns it goes with 3.0), whose pick lives in no
/// config the hook can read (`auto_reading`). The two keys are matched
/// by their exact names (`core.commentary` is somebody else's key): a
/// `## ` with its space is the prefix git strips, and trimming it
/// would have blanked no line.
enum Prefix {
    Fixed(String),
    Auto,
}

fn comment_prefix(root: &Path) -> Prefix {
    let regexp = "^core\\.comment(char|string)$";
    let out = crate::proc::git_output(root, &["config", "--get-regexp", regexp]);
    let text = out
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let last = text
        .lines()
        .rev()
        .find_map(|l| l.split_once(' ').map(|(_, v)| v.to_string()));
    match last {
        Some(v) if v == "auto" => Prefix::Auto,
        Some(v) if !v.is_empty() => Prefix::Fixed(v),
        _ => Prefix::Fixed("#".into()),
    }
}

/// The characters git tries for `auto`, in its order (builtin/commit.c
/// `adjust_comment_line_char`: `char candidates[] = "#;@!$%^&|:"`).
const AUTO_CANDIDATES: &str = "#;@!$%^&|:";

/// The scissors line git writes under `--cleanup=scissors` and
/// `--verbose` after its comment character and a space (wt-status.c
/// `cut_line`); everything from it on is git's.
const CUT_LINE: &str = "------------------------ >8 ------------------------";

/// git picks `auto`'s character before the editor and before its own
/// status block — the first of AUTO_CANDIDATES that begins no line of
/// the message it was handed (`-m`, `-F`, a template, a merge message)
/// — and records the pick nowhere; the hook reads it back from what
/// git wrote into the file: the scissors line, else the status block,
/// the trailing run of two or more lines beginning with one candidate
/// (git's block is never one line; a user's lone `# footnote` is
/// prose). A file with neither had no editor run (`git commit -F`,
/// measured on git 2.52): git's cleanup then strips nothing, `#`-led
/// lines included, so no line is a comment. Answers (character, the
/// index everything from which is git's).
fn auto_reading(lines: &[&str]) -> Option<(char, usize)> {
    let cut = |l: &&str| {
        let b = l.as_bytes();
        b.len() == CUT_LINE.len() + 2
            && AUTO_CANDIDATES.as_bytes().contains(&b[0])
            && b[1] == b' '
            && b[2..] == *CUT_LINE.as_bytes()
    };
    if let Some(at) = lines.iter().position(cut) {
        return Some((lines[at].as_bytes()[0] as char, at));
    }
    let end = lines.iter().rposition(|l| !l.is_empty())? + 1;
    let c = lines[end - 1].chars().next()?;
    let run = lines[..end]
        .iter()
        .rev()
        .take_while(|l| l.starts_with(c))
        .count();
    (AUTO_CANDIDATES.contains(c) && run >= 2).then_some((c, lines.len()))
}

/// The message with every comment line blanked — blanked, not removed,
/// so a site's line is the file's own line (git strips them itself
/// only after this hook has run); under `Auto`, everything from the
/// scissors line on is blanked too.
fn uncommented(text: &str, prefix: &Prefix) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let (comment, cut) = match prefix {
        Prefix::Fixed(p) => (p.clone(), lines.len()),
        Prefix::Auto => match auto_reading(&lines) {
            Some((c, cut)) => (c.to_string(), cut),
            None => return lines.join("\n"),
        },
    };
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            if i >= cut || l.starts_with(comment.as_str()) {
                ""
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
#[path = "../../tests/unit/audit/commitmsg.rs"]
mod tests;
