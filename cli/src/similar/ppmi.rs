//! In-repo association by positive PMI (spec §四): two WORD terms
//! co-occurring in one unit's bag are counted once per unit, and
//! `PPMI(a,b) = max(0, log2(n_ab · N / (n_a · n_b)))` in the same
//! 8-bit fixed point as the idf, from the same integer log2. A query
//! term is widened by its top-m neighbours at a fraction of its own
//! weight — enough to let this repository's `fetch / load / retrieve`
//! meet, never enough to outvote what the unit itself spells. No
//! corpus but this one is consulted, nothing is written to disk (a
//! step-3 persistence question), no float is touched.

use super::bm25::{Corpus, IDF_FRAC_BITS, QueryTerm, log2_fp};
use std::collections::{BTreeSet, HashMap};

/// Neighbours appended per spelled word term.
pub const TOP_M: usize = 3;
/// A neighbour counts only when it co-occurred in at least this many
/// units — one shared unit is coincidence, not association.
pub const MIN_COOC: u32 = 2;
/// PPMI floor: two bits of association, in fixed point.
pub const MIN_PPMI: i128 = 2 << IDF_FRAC_BITS;
/// Expansion weight = parent × min(ppmi, PPMI_CAP) / PPMI_SCALE, so an
/// expansion carries at most half its parent's weight.
pub const PPMI_CAP: i128 = 4 << IDF_FRAC_BITS;
pub const PPMI_SCALE: i128 = 8 << IDF_FRAC_BITS;
/// Distinct word terms of one unit entering the pair count; a unit
/// past the cap contributes its first TERM_CAP terms in term order
/// (deterministic) and is ledgered in `capped_units`.
pub const TERM_CAP: usize = 96;

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
            let mut words: Vec<u64> = d
                .bag
                .terms
                .iter()
                .filter(|(_, (ch, _))| ch.is_words())
                .map(|(term, _)| *term)
                .collect();
            if words.len() > TERM_CAP {
                words.truncate(TERM_CAP);
                t.capped_units += 1;
            }
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

    /// PPMI(a, b) in fixed point; zero below MIN_COOC or at or below
    /// independence.
    pub fn ppmi(&self, a: u64, b: u64) -> i128 {
        let key = if a < b { (a, b) } else { (b, a) };
        let n_ab = self.n_pair.get(&key).copied().unwrap_or(0);
        if n_ab < MIN_COOC {
            return 0;
        }
        let num = u128::from(n_ab) * u128::from(self.n_docs);
        let den = u128::from(self.n_term[&a]) * u128::from(self.n_term[&b]);
        if num <= den { 0 } else { log2_fp(num, den) }
    }

    /// The top-m neighbours of `a` at or above MIN_PPMI, ordered by
    /// PPMI descending then term ascending.
    pub fn neighbours(&self, a: u64) -> Vec<(u64, i128)> {
        let Some(adj) = self.adjacent.get(&a) else {
            return Vec::new();
        };
        let mut out: Vec<(u64, i128)> = adj
            .iter()
            .map(|&b| (b, self.ppmi(a, b)))
            .filter(|(_, p)| *p >= MIN_PPMI)
            .collect();
        out.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
        out.truncate(TOP_M);
        out
    }

    /// Widen a query in place: every spelled word term appends its
    /// neighbours as unspelled terms at the scaled weight; a term the
    /// query already spells is never appended.
    pub fn expand(&self, query: &mut Vec<QueryTerm>) {
        let spelled: BTreeSet<u64> = query.iter().map(|q| q.term).collect();
        let mut added: Vec<QueryTerm> = Vec::new();
        for q in query.iter().filter(|q| q.spelled && q.channel.is_words()) {
            for (term, ppmi) in self.neighbours(q.term) {
                if spelled.contains(&term) || added.iter().any(|a| a.term == term) {
                    continue;
                }
                added.push(QueryTerm {
                    term,
                    channel: q.channel,
                    weight: q.weight * ppmi.min(PPMI_CAP) / PPMI_SCALE,
                    spelled: false,
                });
            }
        }
        query.extend(added);
    }
}

#[cfg(test)]
#[path = "../../tests/unit/similar/ppmi.rs"]
mod tests;
