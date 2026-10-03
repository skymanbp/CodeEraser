//! What the Stop audit and the git-hook faces say, said by the core
//! (plan v2.32 step 5, ruling R7; design booklet
//! docs/reference/authority-track.md §6): one `audit` document over the
//! audit's own link (verdict::open) — the face, the counts and the
//! verdicts as facts and rows, every path a reference — answered with
//! the lines the face prints and `exit.fail`, the block / refuse bit.
//! The sentences and both languages are the core's (`CE.Text.Audit`).
//! A core the face cannot reach, or a reply it cannot bind, still
//! decides by the local rule (`blocked`); the line then is the one
//! English fallback, which names the rules and why the core did not
//! phrase them.

use super::tombstone::Leg;
use super::verdict::Verdict;
use crate::corelink::Link;
use crate::document::lines::{Line, Stream, bind_lines};
use crate::document::{self, Resolve, at};
use serde_json::Value;

/// The faces, by their code in the request.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Face {
    Stop,
    Precommit,
    Commitmsg,
}

impl Face {
    pub(super) fn of(event: &str) -> Self {
        match event {
            "precommit" => Face::Precommit,
            "commitmsg" => Face::Commitmsg,
            _ => Face::Stop,
        }
    }

    /// The face's word in the fallback.
    fn word(self) -> &'static str {
        match self {
            Face::Stop => "audit",
            Face::Precommit => "precommit",
            Face::Commitmsg => "commitmsg",
        }
    }
}

/// One audit's outcome, as the face would say it.
pub(super) struct Said<'a> {
    pub face: Face,
    /// The change was gathered (git answered).
    pub git: bool,
    /// commitmsg's message file, when it could not be read.
    pub unreadable: Option<String>,
    /// A gated submodule's mount, when the audit is its.
    pub mount: Option<&'a str>,
    /// The guard's tier.
    pub mode: &'a str,
    pub changed: usize,
    pub net: i64,
    /// The duplicate verdict (None = it degraded).
    pub dups: Option<&'a Verdict>,
    /// The tombstone leg (None = no leg).
    pub tomb: Option<&'a Leg>,
}

impl Said<'_> {
    /// A face whose change git could not gather.
    pub(super) fn bare(face: Face) -> Said<'static> {
        Said {
            face,
            git: false,
            unreadable: None,
            mount: None,
            mode: "observe",
            changed: 0,
            net: 0,
            dups: None,
            tomb: None,
        }
    }

    fn dup_blocks(&self) -> bool {
        self.mode == "deny" && self.dups.is_some_and(|v| v.fail)
    }

    fn tomb_blocks(&self) -> bool {
        self.tomb.is_some_and(Leg::blocks)
    }

    /// The local rule, the core's `blocked` read the same way: a whole
    /// gathered change, and a deny-tier verdict that holds.
    pub(super) fn blocked(&self) -> bool {
        self.git && self.unreadable.is_none() && (self.dup_blocks() || self.tomb_blocks())
    }
}

impl super::Gathered {
    /// What the face says, over this audit's change.
    pub(super) fn said<'a>(&'a self, face: Face, mount: Option<&'a str>) -> Said<'a> {
        Said {
            face,
            git: true,
            unreadable: None,
            mount,
            mode: &self.mode,
            changed: self.changed.len(),
            net: self.net_loc,
            dups: self.dups.as_ref(),
            tomb: self.tomb.as_ref(),
        }
    }
}

/// The face's lines and its veto: the core's, over `link` (a fresh one
/// when the audit opened none), or the fallback and the local rule.
pub(super) fn spoken(link: Option<&mut Link>, said: &Said) -> (Vec<Line>, bool) {
    let mut fresh = None;
    let link = link.or_else(|| fresh.insert(super::verdict::open()).as_mut());
    asked(link, said).unwrap_or_else(|why| (vec![fallback(said, &why)], said.blocked()))
}

fn asked(link: Option<&mut Link>, said: &Said) -> Result<(Vec<Line>, bool), String> {
    let link = link.ok_or("no core")?;
    if !link.has(document::CAP) {
        return Err(format!("the core offers no {} (pre-7.8.0)", document::CAP));
    }
    let strings = Strings::of(said);
    let reply = link.request(document::KIND, strings.body(said))?;
    crate::corelink::judged::degraded(&reply)?;
    bind_lines(&reply, &strings).map_err(|e| e.to_string())
}

/// The one English line a face says when the core did not phrase it:
/// the rules whose deny-tier verdict holds, and no decision word — the
/// decision is the exit code / the Stop's block channel.
fn fallback(said: &Said, why: &str) -> Line {
    let codes: Vec<&str> = [
        ("duplicate", said.dup_blocks()),
        ("tombstone", said.tomb_blocks()),
    ]
    .into_iter()
    .filter(|(_, held)| *held)
    .map(|(code, _)| code)
    .collect();
    let act = match said.blocked() {
        false => String::new(),
        true => format!("rule {} fired; ", codes.join(", ")),
    };
    let stream = if said.git && said.unreadable.is_none() {
        Stream::Out
    } else {
        Stream::Err
    };
    let face = said.face.word();
    let text = format!("ce {face}: {act}the core could not phrase the verdict: {why}");
    let text = match said.mount {
        Some(mount) => format!("{mount}: {text}"),
        None => text,
    };
    Line { stream, text }
}

/// The strings the request's references name: each shown block's two
/// paths, each shown site's path, and the three single strings.
struct Strings {
    blocks: Vec<(String, String)>,
    places: Vec<String>,
    error: Option<String>,
    mount: Option<String>,
    message: Option<String>,
}

impl Strings {
    fn of(said: &Said) -> Self {
        let shown = said.dups.map_or(&[][..], |v| v.shown.as_slice());
        Strings {
            blocks: shown
                .iter()
                .map(|b| (b.a_file.clone(), b.b_file.clone()))
                .collect(),
            places: said
                .tomb
                .map(|t| t.shown.iter().map(|p| p.file.clone()).collect())
                .unwrap_or_default(),
            error: said.tomb.and_then(|t| t.judged.as_ref().err().cloned()),
            mount: said.mount.map(str::to_string),
            message: said.unreadable.clone(),
        }
    }

    /// The `audit` request (CE.Audit.Document's statement).
    fn body(&self, said: &Said) -> Value {
        let tier = |t: &str| crate::config::TIERS.iter().position(|x| *x == t);
        let shown = said.dups.map_or(&[][..], |v| v.shown.as_slice());
        let blocks: Vec<[usize; 6]> = shown
            .iter()
            .enumerate()
            .map(|(k, b)| [k, b.a_start, b.a_end, b.b_start, b.b_end, b.tokens])
            .collect();
        let places: Vec<[usize; 3]> = said.tomb.map_or_else(Vec::new, |t| {
            let rows = t.shown.iter().enumerate();
            rows.map(|(i, p)| [i, p.line, p.kind as usize]).collect()
        });
        let dups: Vec<[usize; 2]> = said
            .dups
            .map(|v| [v.dups, usize::from(v.fail)])
            .into_iter()
            .collect();
        let tomb: Vec<Vec<usize>> = said
            .tomb
            .map(|t| tomb_row(t, tier(&t.tier)))
            .into_iter()
            .collect();
        document::Request::new("audit")
            .range("blocks", blocks.len())
            .range("places", places.len())
            .range("errors", usize::from(self.error.is_some()))
            .rows("net", [[said.net]])
            .rows("dups", dups)
            .rows("blocks", blocks)
            .rows("tomb", tomb)
            .rows("places", places)
            .fact("face", said.face as u8)
            .fact("git", u8::from(said.git))
            .fact("unreadable", u8::from(said.unreadable.is_some()))
            .fact("mounted", u8::from(said.mount.is_some()))
            .fact("mode", tier(said.mode).unwrap_or(0))
            .fact("changed", said.changed)
            .body()
    }
}

/// The leg's row: [state, sites, label, prose, erased, budget, tier,
/// over, unread, bounded] — state 1 judged, 2 degraded.
fn tomb_row(t: &Leg, tier: Option<usize>) -> Vec<usize> {
    let tail = [
        t.erased,
        t.budget.map_or(0, |b| b as usize),
        tier.unwrap_or(0),
    ];
    let (state, counts, over) = match &t.judged {
        Ok(j) => (1, [j.sites.len(), j.label, j.prose], j.over),
        Err(_) => (2, [0; 3], false),
    };
    let mut row = vec![state];
    row.extend(counts);
    row.extend(tail);
    row.extend([usize::from(over), t.unread, t.bounded]);
    row
}

impl Resolve for Strings {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        let one = |s: &Option<String>| s.clone().filter(|_| ints.is_empty());
        match class {
            "block_a" | "block_b" => {
                let [k] = ints else { return None };
                let (a, b) = self.blocks.get(usize::try_from(*k).ok()?)?;
                Some(if class == "block_a" { a } else { b }.clone())
            }
            "place_file" => at(&self.places, ints),
            "error" => self.error.clone().filter(|_| ints == [0]),
            "mount" => one(&self.mount),
            "message" => one(&self.message),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/audit/speech.rs"]
mod tests;
