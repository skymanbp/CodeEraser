//! In-repo association counts (spec §四): two WORD terms co-occurring
//! in one unit's bag are counted once per unit — the pair counts and
//! the per-word unit counts the PPMI widening reads. The widening itself
//! (PPMI in fixed point, the top-m neighbours, the scaled weights) is
//! the core's (CE.Similar.Rank, plan v2.33 W3); this side keeps the
//! counts and serves them through the `Cooc` trait: the in-memory
//! `Table` and the persisted reader (marginals in `df`, pairs derived
//! from the stored bags at query time — store.rs says why no pair
//! table) count the same capped words. Per the step-2 verdict the
//! widened arm is an opt-in association view, never the default and
//! never evidence.

use super::bag::UnitBag;
use super::corpus::Corpus;
use anyhow::Result;
use std::collections::{BTreeSet, HashMap};

/// Distinct word terms of one unit entering the pair count; a unit
/// past the cap contributes its first TERM_CAP terms in term order
/// (deterministic) and is ledgered in `capped_units`.
pub const TERM_CAP: usize = 96;

/// The word terms of one unit that enter the pair count: its distinct
/// word-channel terms in term order (the bag is a BTreeMap, so every
/// pair (a, b) drawn from the list has a < b), cut at TERM_CAP; the
/// flag says the cut happened. The ONE owner of the cap — the
/// in-memory table and the persisted writer count the same words.
pub fn capped_words(bag: &UnitBag) -> (Vec<u64>, bool) {
    let mut words: Vec<u64> = bag
        .terms
        .iter()
        .filter(|(_, (ch, _))| ch.is_words())
        .map(|(term, _)| *term)
        .collect();
    let capped = words.len() > TERM_CAP;
    words.truncate(TERM_CAP);
    (words, capped)
}

/// What association needs from a pair count: the unit total N, a
/// word's own unit count n_a, and every `(b, n_ab)` it co-occurred
/// with.
pub trait Cooc {
    fn n_units(&self) -> u32;
    fn n_term(&self, a: u64) -> Result<u32>;
    fn pairs(&self, a: u64) -> Result<Vec<(u64, u32)>>;
}

/// The in-memory pair count over a corpus (instruments and unit
/// tests; the product reads the persisted `cooc` rows).
pub struct Table {
    n_docs: u32,
    n_term: HashMap<u64, u32>,
    n_pair: HashMap<(u64, u64), u32>,
    adjacent: HashMap<u64, BTreeSet<u64>>,
    pub capped_units: u32,
}

impl Table {
    /// Count every unit's word-term pairs.
    pub fn build(corpus: &Corpus) -> Table {
        let mut t = Table {
            n_docs: corpus.docs.len() as u32,
            n_term: HashMap::new(),
            n_pair: HashMap::new(),
            adjacent: HashMap::new(),
            capped_units: 0,
        };
        for d in &corpus.docs {
            let (words, capped) = capped_words(&d.bag);
            t.capped_units += u32::from(capped);
            t.count(&words);
        }
        t
    }

    fn count(&mut self, words: &[u64]) {
        for (i, &a) in words.iter().enumerate() {
            *self.n_term.entry(a).or_insert(0) += 1;
            for &b in &words[i + 1..] {
                *self.n_pair.entry((a, b)).or_insert(0) += 1;
                self.adjacent.entry(a).or_default().insert(b);
                self.adjacent.entry(b).or_default().insert(a);
            }
        }
    }

    fn n_pair(&self, a: u64, b: u64) -> u32 {
        let key = if a < b { (a, b) } else { (b, a) };
        self.n_pair.get(&key).copied().unwrap_or(0)
    }
}

impl Cooc for Table {
    fn n_units(&self) -> u32 {
        self.n_docs
    }

    fn n_term(&self, a: u64) -> Result<u32> {
        Ok(self.n_term.get(&a).copied().unwrap_or(0))
    }

    fn pairs(&self, a: u64) -> Result<Vec<(u64, u32)>> {
        Ok(self
            .adjacent
            .get(&a)
            .map(|adj| adj.iter().map(|&b| (b, self.n_pair(a, b))).collect())
            .unwrap_or_default())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/similar/ppmi.rs"]
mod tests;
