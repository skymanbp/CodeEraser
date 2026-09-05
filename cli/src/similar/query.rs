//! What a similar query asks about (spec §六): the unit holding
//! `file:line`, a unit by key, or free text — resolved against the
//! index reader into the query bag every arm ranks with. Resolution
//! is measurement (which seat holds a line, which words a text
//! carries); nothing here orders or judges, and an ambiguous ask is
//! refused by name rather than answered for one of its readings.

use super::bag::UnitBag;
use super::bm25::{QueryTerm, query_of};
use super::reader::{Reader, Seat};
use super::terms::{self, Channel};
use anyhow::{Result, bail, ensure};

/// The three asks every face accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ask {
    At { path: String, line: usize },
    Unit(String),
    Text(String),
}

impl Ask {
    /// `path:line` — split at the LAST colon, so a drive letter or a
    /// path with colons still parses; lines count from 1.
    pub fn at(spec: &str) -> Result<Ask> {
        let parsed = spec
            .rsplit_once(':')
            .filter(|(p, _)| !p.is_empty())
            .and_then(|(p, l)| l.parse::<usize>().ok().map(|l| (p, l)));
        let Some((path, line)) = parsed else {
            bail!("`at` wants `file:line`, got `{spec}`");
        };
        ensure!(line >= 1, "`at` line numbers start at 1, got `{spec}`");
        Ok(Ask::At {
            path: path.replace('\\', "/"),
            line,
        })
    }

    /// Exactly one of the three, from the optional parts every face
    /// collects (clap's group, the MCP arguments, the GUI's inputs).
    pub fn from_parts(at: Option<&str>, text: Option<&str>, unit: Option<&str>) -> Result<Ask> {
        match (at, text, unit) {
            (Some(at), None, None) => Ask::at(at),
            (None, Some(t), None) if !t.trim().is_empty() => Ok(Ask::Text(t.to_string())),
            (None, None, Some(u)) if !u.is_empty() => Ok(Ask::Unit(u.to_string())),
            _ => bail!("similar wants exactly one of `at` (file:line), `text`, `unit`"),
        }
    }
}

/// The resolved query: the seat it excludes from its own candidates
/// (a unit is not similar to itself), how every face labels it, and
/// the terms.
pub struct Resolved {
    pub seat: Option<usize>,
    pub label: String,
    pub terms: Vec<QueryTerm>,
}

pub fn resolve(reader: &Reader<'_>, ask: &Ask) -> Result<Resolved> {
    match ask {
        Ask::At { path, line } => match innermost(reader.seats(), path, *line) {
            Some(seat) => seated(reader, seat),
            None => bail!(
                "no indexed unit at {path}:{line} (a judged language inside the own universe)"
            ),
        },
        Ask::Unit(key) => {
            let seats: Vec<usize> = (0..reader.seats().len())
                .filter(|&i| reader.seats()[i].key == *key)
                .collect();
            match seats.as_slice() {
                [one] => seated(reader, *one),
                [] => bail!("no indexed unit keyed `{key}`"),
                many => bail!(
                    "{} units keyed `{key}` — name one with `at` file:line: {}",
                    many.len(),
                    many.iter()
                        .take(5)
                        .map(|&i| place(&reader.seats()[i]))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            }
        }
        Ask::Text(text) => Ok(Resolved {
            seat: None,
            label: format!("text: {}", text.trim()),
            terms: text_terms(text),
        }),
    }
}

/// `path:start-end` — where a seat is, the way every face spells it.
pub fn place(s: &Seat) -> String {
    format!("{}:{}-{}", s.path, s.start_line, s.end_line)
}

/// The innermost seat of `path` whose span holds `line` (a method
/// inside a class is the method), if any.
fn innermost(seats: &[Seat], path: &str, line: usize) -> Option<usize> {
    let line = line as i64;
    seats
        .iter()
        .enumerate()
        .filter(|(_, s)| s.path == path && s.start_line <= line && line <= s.end_line)
        .min_by_key(|(_, s)| s.end_line - s.start_line)
        .map(|(i, _)| i)
}

fn seated(reader: &Reader<'_>, seat: usize) -> Result<Resolved> {
    let bag = reader.bag(seat)?;
    let s = &reader.seats()[seat];
    Ok(Resolved {
        seat: Some(seat),
        label: format!("{} {}", place(s), s.key),
        terms: query_of(&bag),
    })
}

/// Free text as a query: its words (the term road's prose split, stop
/// list and stemmer — one road for index and query) as NAME and DOC
/// evidence at those channels' weights. No shape, callee, structure or
/// literal term: a text carries nothing the conjunction's shape arm
/// reads and no callee for its first arm, so the role bits the core
/// answers for a text query are false by construction — reported as
/// the core's answer, never decided here.
pub fn text_terms(text: &str) -> Vec<QueryTerm> {
    let mut bag = UnitBag::empty(String::new(), 0);
    for w in terms::prose_words(text) {
        for ch in [Channel::Name, Channel::Doc] {
            bag.terms
                .entry(terms::word_term(ch, &w))
                .or_insert((ch, 0))
                .1 += 1;
        }
    }
    query_of(&bag)
}

#[cfg(test)]
#[path = "../../tests/unit/similar/query.rs"]
mod tests;
