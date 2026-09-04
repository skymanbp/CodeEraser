//! The texts a changeset's pairs hold: every git side in ONE `cat-file
//! --batch` process and the working tree through one bounded read.
//! The Stop leg must not pay a `git show` per changed file (the Stop
//! row of PERF-BUDGET.md is a whole-session budget), and the FPR
//! replay walks thousands of commits through this same door. A blob
//! that is missing, binary or past READ_CAP drops its pair, and pairs
//! past PAIR_CAP are never read — both are counted back to the
//! caller, never silent.

use crate::fourclass::session::PathPair;
use crate::scan::lang::Lang;
use std::path::Path;

/// Files past this many bytes read as absent. The audit's untracked
/// leg set the stance (an unbounded read is an availability hole: a
/// stray 2 GB csv measured 1.96 GB RSS on a Stop) and reads through
/// the same door.
pub const READ_CAP: usize = 4 << 20;

/// Pairs one measurement reads at most; the rest are counted, not
/// read — a whole-tree rewrite must not turn a Stop into a read of
/// everything.
pub const PAIR_CAP: usize = 256;

/// Where one side's text comes from.
#[derive(Clone, Copy)]
pub enum Side<'a> {
    /// `<rev>:<path>` through git.
    Rev(&'a str),
    /// `:<path>` — the index.
    Index,
    /// The file under the root, read bounded.
    Worktree,
}

/// One pair with both texts (an absent side is empty).
pub struct Loaded {
    pub rel: String,
    pub before: String,
    pub after: String,
    pub lang: Lang,
}

/// The judged pairs' texts (PAIR_CAP of them at most) and how many
/// judged pairs were left unread. None = git could not answer.
pub fn load(
    root: &Path,
    pairs: &[PathPair],
    before: Side,
    after: Side,
) -> Option<(Vec<Loaded>, usize)> {
    let judged: Vec<(&PathPair, Lang)> = pairs
        .iter()
        .filter_map(|p| {
            Some((
                p,
                Lang::judged_path(Path::new(p.1.as_deref().or(p.0.as_deref())?))?,
            ))
        })
        .collect();
    let dropped = judged.len().saturating_sub(PAIR_CAP);
    let judged = &judged[..judged.len().min(PAIR_CAP)];
    let mut specs = Vec::new();
    for ((b, a), _) in judged {
        spec(&mut specs, before, b.as_deref());
        spec(&mut specs, after, a.as_deref());
    }
    let mut blobs = batch(root, &specs)?.into_iter();
    let mut out = Vec::new();
    for ((b, a), lang) in judged {
        let before_text = text(root, before, b.as_deref(), &mut blobs);
        let after_text = text(root, after, a.as_deref(), &mut blobs);
        if let (Some(before), Some(after)) = (before_text, after_text) {
            let rel = a
                .as_deref()
                .or(b.as_deref())
                .unwrap_or_default()
                .to_string();
            out.push(Loaded {
                rel,
                before,
                after,
                lang: *lang,
            });
        }
    }
    Some((out, dropped))
}

/// The batch line one git-side path needs (none for an absent side or
/// the working tree). Pairs are ce-root-relative (session::scoped_pairs)
/// while a bare `<rev>:<path>` is repo-root-relative; `./` makes git
/// resolve the path against its own cwd — the root every spawn here
/// runs in (`git -C root`) — so a nested root spends no `rev-parse
/// --show-prefix` spawn on the Stop leg (a spawn is ~60 ms here, and
/// the information legs' stance is to pay none they can avoid).
fn spec(specs: &mut Vec<String>, side: Side, path: Option<&str>) {
    match (side, path) {
        (Side::Rev(rev), Some(p)) => specs.push(format!("{rev}:./{p}")),
        (Side::Index, Some(p)) => specs.push(format!(":./{p}")),
        _ => {}
    }
}

/// One side's text: empty for an absent path, the next batch answer
/// for a git side, a bounded read for the working tree.
fn text(
    root: &Path,
    side: Side,
    path: Option<&str>,
    blobs: &mut impl Iterator<Item = Option<String>>,
) -> Option<String> {
    let Some(p) = path else {
        return Some(String::new());
    };
    match side {
        Side::Worktree => read_capped(&crate::scan::walk::contained(root, p)?),
        Side::Rev(_) | Side::Index => blobs.next()?,
    }
}

/// `git cat-file --batch` over every spec, answers in spec order:
/// None for a missing object, a non-UTF-8 blob, or one past READ_CAP.
/// The reply grammar is `<sha> <type> <size>\n<bytes>\n` per object
/// and `<spec> missing\n` for an absent one.
fn batch(root: &Path, specs: &[String]) -> Option<Vec<Option<String>>> {
    if specs.is_empty() {
        return Some(Vec::new());
    }
    let input = specs.join("\n") + "\n";
    let out = crate::proc::git_feed(root, &["cat-file", "--batch"], input.as_bytes()).ok()?;
    if !out.status.success() {
        return None;
    }
    let bytes = &out.stdout;
    let (mut at, mut blobs) = (0, Vec::new());
    while blobs.len() < specs.len() {
        let nl = bytes[at..].iter().position(|b| *b == b'\n')?;
        let header = std::str::from_utf8(&bytes[at..at + nl]).ok()?;
        at += nl + 1;
        if header.ends_with(" missing") {
            blobs.push(None);
            continue;
        }
        let size: usize = header.rsplit(' ').next()?.parse().ok()?;
        let body = bytes.get(at..at + size)?;
        at += size + 1;
        let text = (size <= READ_CAP).then(|| String::from_utf8(body.to_vec()).ok());
        blobs.push(text.flatten());
    }
    Some(blobs)
}

/// A file's text, or None when unreadable, binary, or past READ_CAP.
/// The cap is enforced by the READ, not by a metadata check
/// beforehand: a stat-then-read pair is a race a growing file wins
/// (and a symlink or /dev/zero has no useful size at all), so the
/// reader itself is bounded and an over-cap file reads as absent
/// rather than being trusted.
pub fn read_capped(path: &Path) -> Option<String> {
    use std::io::Read as _;
    let mut buf = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(READ_CAP as u64 + 1)
        .read_to_end(&mut buf)
        .ok()?;
    (buf.len() <= READ_CAP)
        .then(|| String::from_utf8(buf).ok())
        .flatten()
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone/texts.rs"]
mod tests;
