//! In-repo association by positive PMI (spec §四): two WORD terms
//! co-occurring in one unit's bag are counted once per unit, and
//! `PPMI(a,b) = max(0, log2(n_ab · N / (n_a · n_b)))` in the same
//! 8-bit fixed point as the idf, from the same integer log2. A query
//! term is widened by its top-m neighbours at a fraction of its own
//! weight — enough to let this repository's `fetch / load / retrieve`
//! meet, never enough to outvote what the unit itself spells. No
//! corpus but this one is consulted, no float is touched. The
//! association math is written once against the `Cooc` trait: the
//! in-memory `Table` and the persisted reader (marginals in `df`,
//! pairs derived from the stored bags at query time — store.rs says
//! why no pair table) count the same capped words and answer the
//! same neighbours. Per the step-2 verdict the widened arm is an
//! opt-in association view, never the default and never evidence.

use super::bag::UnitBag;
use super::bm25::{Corpus, IDF_FRAC_BITS, QueryTerm, log2_fp};
use anyhow::Result;
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

/// PPMI(a, b) in fixed point from the four counts; zero below MIN_COOC
/// or at or below independence.
fn ppmi_fp(n_units: u32, n_ab: u32, n_a: u32, n_b: u32) -> i128 {
    if n_ab < MIN_COOC {
        return 0;
    }
    let num = u128::from(n_ab) * u128::from(n_units);
    let den = u128::from(n_a) * u128::from(n_b);
    if num <= den { 0 } else { log2_fp(num, den) }
}

/// The top-m neighbours of `a` at or above MIN_PPMI, ordered by PPMI
/// descending then term ascending. A word in more than a quarter of
/// the units has none: n_ab ≤ n_b bounds PPMI(a, b) by log2(N / n_a),
/// under MIN_PPMI's two bits as soon as 4·n_a > N — exact, so the
/// persisted reader never walks such a word's pair rows.
pub fn neighbours(c: &impl Cooc, a: u64) -> Result<Vec<(u64, i128)>> {
    let (n, n_a) = (c.n_units(), c.n_term(a)?);
    if n_a == 0 || 4 * n_a > n {
        return Ok(Vec::new());
    }
    let mut out: Vec<(u64, i128)> = Vec::new();
    for (b, n_ab) in c.pairs(a)? {
        if n_ab < MIN_COOC {
            continue;
        }
        let p = ppmi_fp(n, n_ab, n_a, c.n_term(b)?);
        if p >= MIN_PPMI {
            out.push((b, p));
        }
    }
    out.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
    out.truncate(TOP_M);
    Ok(out)
}

/// Widen a query in place: every spelled word term appends its
/// neighbours as unspelled terms at the scaled weight; a term the
/// query already spells is never appended.
pub fn expand(c: &impl Cooc, query: &mut Vec<QueryTerm>) -> Result<()> {
    let spelled: BTreeSet<u64> = query.iter().map(|q| q.term).collect();
    let mut added: Vec<QueryTerm> = Vec::new();
    for q in query.iter().filter(|q| q.spelled && q.channel.is_words()) {
        for (term, ppmi) in neighbours(c, q.term)? {
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
    Ok(())
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

    /// PPMI(a, b) as this table counts it.
    pub fn ppmi(&self, a: u64, b: u64) -> i128 {
        let n_of = |t| self.n_term.get(&t).copied().unwrap_or(0);
        ppmi_fp(self.n_docs, self.n_pair(a, b), n_of(a), n_of(b))
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
