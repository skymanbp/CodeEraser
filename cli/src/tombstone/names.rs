//! The erased-name set R (spec §三 M1): which names a change removed
//! from its changeset. A NAME is a spelling a structural position of
//! the text declares (marked.rs: a code line's identifier, a declared
//! unit's name, a heading, a list item's lead) — never a word of prose
//! (deleting a comment that said `downtime` must not let `without
//! downtime` in a new heading fire), and never an inline code span,
//! which only mentions. A compound spells every window of up to
//! JOIN_MAX of its words: `braise_dongpo_pork` names `dongpo_pork`
//! too, which is what `(no Dongpo Pork)` binds.
//!
//! ERASED means: a name of some before side that SURVIVES on no after
//! side. A name survives in every marked text this change did not
//! touch (a code span included), and in a structural one it added —
//! outside the slots an absence frame binds there. A mention this
//! change wrote into prose or a code span is not survival: that is
//! where residue is written.
//!
//! Keys, not spellings, are what the hub compares and what the feed
//! stores: `key` hashes the canonical spelling (ASCII words lower-
//! cased and `_`-joined, wide runs verbatim), so `DongpoPork`,
//! `dongpo_pork` and `Dongpo Pork` in a heading are one name.

use super::frames::{Word, label_candidates, windows, words};
use super::marked::{Marked, marked};
use super::vocab::{JOIN_MAX, KEYWORDS, MIN_ASCII_NAME, MIN_WIDE_NAME, NEGATIONS, has, vocabulary};
use super::{PairText, Policy};
use crate::dedup::tokens::fnv1a;
use crate::mention::token::{fold, runs};
use crate::scan::lang::Lang;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    /// The canonical spelling.
    pub text: String,
    pub key: u64,
    /// Carries a non-ASCII char: also matched by substring in prose,
    /// since a Chinese sentence is one word.
    pub wide: bool,
}

/// The canonical spelling a key hashes: the text's words `_`-joined.
pub fn canon(s: &str) -> String {
    let w = words(s);
    let parts: Vec<&str> = w.iter().filter_map(Word::text).collect();
    parts.join("_")
}

pub fn key(s: &str) -> u64 {
    fnv1a(canon(s).as_bytes())
}

/// Every spelling a text offers a name under: each window of its
/// words (frames::windows) plus each whole run wider than a window.
fn spellings(text: &str) -> Vec<String> {
    let mut out: Vec<String> = windows(&words(text)).into_iter().map(|s| s.text).collect();
    out.extend(long_runs(text));
    out
}

/// The canonical spelling of each run wider than a window (a four-
/// word identifier is its own spelling beyond it).
fn long_runs(text: &str) -> impl Iterator<Item = String> + '_ {
    runs(text)
        .map(canon)
        .filter(|c| c.matches('_').count() >= JOIN_MAX)
}

/// The key of every spelling a text could mention a name under.
pub fn spelled_in(text: &str, known: impl Fn(u64) -> bool) -> Option<String> {
    spellings(text).into_iter().find(|s| known(key(s)))
}

/// The name floor: a letter, long enough for its script, no word of
/// the instrument's vocabulary among its words (a frame, an absence
/// word, a function word, a mark's word) nor of the repository's own
/// (`[tombstone] terms`), and not made of reserved words alone
/// (`user_data` is a name; `data` is not).
fn admitted(s: &str, policy: &Policy) -> bool {
    let wide = !s.is_ascii();
    let floor = if wide { MIN_WIDE_NAME } else { MIN_ASCII_NAME };
    let ws: Vec<&str> = s.split('_').collect();
    s.chars().any(char::is_alphabetic)
        && s.chars().filter(|c| *c != '_').count() >= floor
        && ws.iter().all(|w| !vocabulary(w) && !policy.term(w))
        && ws.iter().any(|w| !has(KEYWORDS, w))
}

/// The spellings of one marked text outside the slots an absence
/// frame binds in it — every window overlapping a candidate of
/// frames::label_candidates, the same reading the label side makes —
/// plus its long runs. `fn no_return_chars` declares nothing named
/// `return_chars`, nor `chars`, on either side of a change: the name
/// side and the survival side read a text through this one door, so
/// a framed word can be neither erased nor alive.
fn free_spellings(text: &str) -> Vec<String> {
    let w = words(text);
    let slots: Vec<(usize, usize)> = label_candidates(&w)
        .iter()
        .map(|c| (c.span.at, c.span.at + c.span.len))
        .collect();
    let mut out: Vec<String> = windows(&w)
        .into_iter()
        .filter(|s| !slots.iter().any(|(a, b)| s.at < *b && *a < s.at + s.len))
        .map(|s| s.text)
        .collect();
    out.extend(long_runs(text));
    out
}

/// The names one text declares (its structural positions), keyed and
/// de-duplicated. A text that is an absence word WHOLE (`NotFound`,
/// `no_std`: its fold is in V₀) spells no name at all — its `found`
/// half must not enter R just because the word is compound.
pub fn names_of(text: &str, lang: Lang, policy: &Policy) -> Vec<Name> {
    let mut seen = BTreeSet::new();
    let positions: Vec<Marked> = marked(text, lang);
    positions
        .iter()
        .filter(|m| m.structural && !has(NEGATIONS, &fold(&m.text)))
        .flat_map(|m| free_spellings(&m.text))
        .filter(|s| admitted(s, policy))
        .filter_map(|s| {
            let key = key(&s);
            seen.insert(key).then(|| Name {
                wide: !s.is_ascii(),
                text: s,
                key,
            })
        })
        .collect()
}

/// The keys under which a name survives on one after side (module
/// header): every free spelling of a marked text this change did not
/// add, and of a structural one it did add — so `(no dongpo)` keeps
/// nothing alive and `dongpo pork` keeps `dongpo_pork`.
fn surviving(after: &str, lang: Lang, added: &BTreeSet<usize>) -> BTreeSet<u64> {
    marked(after, lang)
        .iter()
        .filter(|m| m.structural || !added.contains(&m.line))
        .flat_map(|m| free_spellings(&m.text))
        .map(|s| key(&s))
        .collect()
}

/// The erased set of a changeset and its membership tests.
#[derive(Debug, Default)]
pub struct Erased {
    pub names: Vec<Name>,
    keys: BTreeSet<u64>,
}

impl Erased {
    pub fn has(&self, key: u64) -> bool {
        self.keys.contains(&key)
    }

    /// The first wide erased name occurring in `text` as a substring
    /// — the read a key cannot make, since a Chinese sentence is one
    /// word.
    pub fn wide_in(&self, text: &str) -> Option<&str> {
        self.names
            .iter()
            .find(|n| n.wide && text.contains(&n.text))
            .map(|n| n.text.as_str())
    }
}

/// R over a changeset (`added[i]` = the after-side lines pair `i`
/// added): every before-side name that survives on no after side. A
/// name moved to another changed file still survives; a name that
/// only recurs inside `(no X)` does not.
pub fn erased(pairs: &[PairText], added: &[BTreeSet<usize>], policy: &Policy) -> Erased {
    let mut seen = BTreeSet::new();
    let before: Vec<Name> = pairs
        .iter()
        .flat_map(|p| names_of(p.before, p.lang, policy))
        .filter(|n| seen.insert(n.key))
        .collect();
    let alive: BTreeSet<u64> = pairs
        .iter()
        .zip(added)
        .flat_map(|(p, a)| surviving(p.after, p.lang, a))
        .collect();
    let names: Vec<Name> = before
        .into_iter()
        .filter(|n| !alive.contains(&n.key))
        .collect();
    let keys = names.iter().map(|n| n.key).collect();
    Erased { names, keys }
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone/names.rs"]
mod tests;
