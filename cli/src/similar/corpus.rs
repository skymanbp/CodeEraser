//! The in-memory corpus (plan v2.33 W3): every bag with its posting
//! lists, seated in (path, key, nth) order, for the instruments and the
//! unit tests — the product reads the persisted tables through
//! reader.rs; both are `Postings` sources of one rank/1 request.

use super::bag::UnitBag;
use super::rank::{Postings, QueryTerm};
use super::terms::Channel;
use anyhow::{Result, ensure};
use std::collections::HashMap;

/// One indexed unit: its file and its bag.
pub struct Doc {
    pub path: String,
    pub bag: UnitBag,
}

/// The in-memory corpus: every bag with its posting lists, for the
/// instruments and the unit tests (the product reads the persisted
/// tables through reader.rs).
pub struct Corpus {
    pub docs: Vec<Doc>,
    postings: HashMap<u64, Vec<(usize, u32)>>,
    total_len: u64,
}

impl Corpus {
    /// Build the inverted index over `docs`, whose order is the seat
    /// order and must be strictly ascending by (path, key, nth) — the
    /// core breaks score ties by seat.
    pub fn build(docs: Vec<Doc>) -> Result<Corpus> {
        ensure!(
            docs.windows(2).all(|w| identity(&w[0]) < identity(&w[1])),
            "corpus seats not strictly ascending by (path, key, nth)"
        );
        let mut postings: HashMap<u64, Vec<(usize, u32)>> = HashMap::new();
        let mut total_len = 0u64;
        for (i, d) in docs.iter().enumerate() {
            total_len += u64::from(d.bag.len());
            for (term, (_, tf)) in &d.bag.terms {
                postings.entry(*term).or_default().push((i, *tf));
            }
        }
        Ok(Corpus {
            docs,
            postings,
            total_len,
        })
    }

    /// A unit's own bag as a query.
    pub fn query_of(&self, doc: usize) -> Vec<QueryTerm> {
        query_of(&self.docs[doc].bag)
    }
}

fn identity(d: &Doc) -> (&str, &str, i64) {
    (&d.path, &d.bag.key, d.bag.nth)
}

impl Postings for Corpus {
    fn n_docs(&self) -> usize {
        self.docs.len()
    }

    /// Average bag length, floored, never below one.
    fn avg_len(&self) -> i128 {
        (self.total_len / self.docs.len().max(1) as u64).max(1) as i128
    }

    fn df(&self, term: u64) -> Result<usize> {
        Ok(self.postings.get(&term).map_or(0, Vec::len))
    }

    fn posting(&self, term: u64) -> Result<Vec<(usize, u32)>> {
        Ok(self.postings.get(&term).cloned().unwrap_or_default())
    }

    fn len(&self, seat: usize) -> u32 {
        self.docs[seat].bag.len()
    }

    fn shape(&self, seat: usize) -> Result<Vec<u64>> {
        Ok(self.docs[seat].bag.channel(Channel::Shape))
    }
}

/// The query form of one bag: every term with its channel and tf, in
/// term order (the core weights them).
pub fn query_of(bag: &UnitBag) -> Vec<QueryTerm> {
    bag.terms
        .iter()
        .map(|(term, (channel, tf))| QueryTerm {
            term: *term,
            channel: *channel,
            tf: *tf,
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/similar/corpus.rs"]
mod tests;
