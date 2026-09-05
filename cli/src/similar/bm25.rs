//! Integer BM25 over unit bags (spec §三): k1 = 6/5 and b = 3/4 as
//! rationals folded into one integer fraction, idf in 8-bit fixed
//! point from an integer log2, every per-term contribution floored to
//! 16-bit fixed point — no float anywhere, so the same corpus ranks
//! the same on every platform and the frozen eval docs compare byte
//! for byte. The role conjunction computed here is the instrument's
//! declared MIRROR of what CE.Similar will own once the wire family
//! lands (step 5); the measurement side never decides alone after
//! that.

use super::bag::UnitBag;
use super::terms::Channel;
use std::collections::HashMap;

/// Okapi k1 and b (Robertson & Walker 1994's usual settings), as the
/// rationals the expanded fraction in `contribution` is derived from.
pub const K1: (i128, i128) = (6, 5);
pub const B: (i128, i128) = (3, 4);
pub const IDF_FRAC_BITS: u32 = 8;
pub const SCORE_FRAC_BITS: u32 = 16;
/// Query weights ride in 1/W_UNIT, so a PPMI-scaled expansion
/// (ppmi.rs) keeps a fraction of its parent's weight without floats.
pub const W_UNIT: i128 = 256;

/// One indexed unit: its file and its bag.
pub struct Doc {
    pub path: String,
    pub bag: UnitBag,
}

/// One query term: `spelled` = false marks an expansion, which adds
/// score but never counts as channel evidence.
#[derive(Debug, Clone)]
pub struct QueryTerm {
    pub term: u64,
    pub channel: Channel,
    pub weight: i128,
    pub spelled: bool,
}

/// One ranked candidate: fixed-point score, distinct spelled terms
/// shared per channel `[N,P,C,D,S,L]`, shape equality, and the role
/// bit — `nHit ≥ 1 ∧ cHit ≥ 1`, or `nHit ≥ 2 ∧ shape equal` (spec §五).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub doc: usize,
    pub score: i64,
    pub hits: [u32; 6],
    pub shape_equal: bool,
    pub role: bool,
}

pub struct Corpus {
    pub docs: Vec<Doc>,
    postings: HashMap<u64, Vec<(u32, u32)>>,
    total_len: u64,
}

impl Corpus {
    /// Build the inverted index over `docs` (their order is the
    /// candidate identity order — callers sort by (path, key, nth)).
    pub fn build(docs: Vec<Doc>) -> Corpus {
        let mut postings: HashMap<u64, Vec<(u32, u32)>> = HashMap::new();
        let mut total_len = 0u64;
        for (i, d) in docs.iter().enumerate() {
            total_len += u64::from(d.bag.len());
            for (term, (_, tf)) in &d.bag.terms {
                postings.entry(*term).or_default().push((i as u32, *tf));
            }
        }
        Corpus {
            docs,
            postings,
            total_len,
        }
    }

    pub fn df(&self, term: u64) -> usize {
        self.postings.get(&term).map_or(0, Vec::len)
    }

    /// Average bag length, floored, never below one.
    pub fn avg_len(&self) -> i128 {
        (self.total_len / self.docs.len().max(1) as u64).max(1) as i128
    }

    /// A unit's own bag as a query: `tf × channel weight × W_UNIT`.
    pub fn query_of(&self, doc: usize) -> Vec<QueryTerm> {
        query_of(&self.docs[doc].bag)
    }

    /// The top `k` candidates for `query`, excluding `exclude` (the
    /// query's own seat), ordered by score then identity. A term in
    /// more than half the units (idf 0) is neither score nor evidence:
    /// sharing what nearly everything shares says nothing, and walking
    /// its posting list would cost the whole corpus per query.
    pub fn top_k(&self, query: &[QueryTerm], k: usize, exclude: Option<usize>) -> Vec<Hit> {
        let avg = self.avg_len();
        let mut acc: HashMap<usize, (i128, [u32; 6])> = HashMap::new();
        for q in query {
            let Some(posting) = self.postings.get(&q.term) else {
                continue;
            };
            let idf = idf_fp(self.docs.len(), posting.len());
            if idf == 0 {
                continue;
            }
            for &(doc, tf) in posting {
                let (doc, len) = (doc as usize, i128::from(self.docs[doc as usize].bag.len()));
                let e = acc.entry(doc).or_insert((0, [0; 6]));
                e.0 += contribution(q.weight, idf, i128::from(tf), len, avg);
                if q.spelled {
                    e.1[q.channel.index()] += 1;
                }
            }
        }
        let shape: Vec<u64> = shape_terms(query);
        let mut hits: Vec<Hit> = acc
            .into_iter()
            .filter(|(d, _)| Some(*d) != exclude)
            .map(|(doc, (score, hits))| {
                let shape_equal = self.docs[doc].bag.channel(Channel::Shape) == shape;
                Hit {
                    doc,
                    score: i64::try_from(score >> SCORE_FRAC_BITS).expect("score fits i64"),
                    hits,
                    shape_equal,
                    role: role(&hits, shape_equal),
                }
            })
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| self.identity(a.doc).cmp(&self.identity(b.doc)))
        });
        hits.truncate(k);
        hits
    }

    fn identity(&self, doc: usize) -> (&str, &str, i64) {
        let d = &self.docs[doc];
        (&d.path, &d.bag.key, d.bag.nth)
    }
}

/// The query form of one bag.
pub fn query_of(bag: &UnitBag) -> Vec<QueryTerm> {
    bag.terms
        .iter()
        .map(|(term, (channel, tf))| QueryTerm {
            term: *term,
            channel: *channel,
            weight: i128::from(*tf) * i128::from(channel.weight()) * W_UNIT,
            spelled: true,
        })
        .collect()
}

/// The sorted spelled shape terms of a query.
fn shape_terms(query: &[QueryTerm]) -> Vec<u64> {
    let mut v: Vec<u64> = query
        .iter()
        .filter(|q| q.spelled && q.channel == Channel::Shape)
        .map(|q| q.term)
        .collect();
    v.sort_unstable();
    v
}

/// The same-role conjunction over an evidence row (spec §五).
pub fn role(hits: &[u32; 6], shape_equal: bool) -> bool {
    let (n, c) = (hits[Channel::Name.index()], hits[Channel::Callee.index()]);
    (n >= 1 && c >= 1) || (n >= 2 && shape_equal)
}

/// idf(t) = log2((N − df + ½) / (df + ½)) in IDF_FRAC_BITS fixed
/// point, floored at zero (a term in more than half the units carries
/// no discrimination and no penalty).
pub fn idf_fp(n_docs: usize, df: usize) -> i128 {
    let num = 2 * n_docs as u128 + 1 - 2 * df as u128;
    let den = 2 * df as u128 + 1;
    if num <= den { 0 } else { log2_fp(num, den) }
}

/// One term's contribution, floored to SCORE_FRAC_BITS fixed point:
/// `w · idf · (k1+1)·tf / (tf + k1·(1 − b + b·len/avg))` with k1 = 6/5
/// and b = 3/4 expanded to `22·tf·avg / (10·tf·avg + 3·avg + 9·len)`
/// (the unit test re-derives this fraction from K1 and B).
pub fn contribution(w: i128, idf: i128, tf: i128, len: i128, avg: i128) -> i128 {
    let num = (w * idf * 22 * tf * avg) << SCORE_FRAC_BITS;
    let den = 10 * tf * avg + 3 * avg + 9 * len;
    num / den
}

/// floor(2^IDF_FRAC_BITS · log2(num/den)) for num ≥ den > 0, by
/// integer squaring only — the fraction bits fall out of whether the
/// squared ratio passes two. Operands are kept below 2^62 by equal
/// right shifts, a truncation that is the same on every platform.
pub fn log2_fp(num: u128, den: u128) -> i128 {
    debug_assert!(den > 0 && num >= den);
    let (mut n, mut d, mut int_part) = (num, den, 0i128);
    while n >= 2 * d {
        d <<= 1;
        int_part += 1;
    }
    let mut frac = 0i128;
    for _ in 0..IDF_FRAC_BITS {
        while n >= 1 << 62 || d >= 1 << 62 {
            n >>= 1;
            d >>= 1;
        }
        n *= n;
        d *= d;
        frac <<= 1;
        if n >= 2 * d {
            d <<= 1;
            frac |= 1;
        }
    }
    (int_part << IDF_FRAC_BITS) | frac
}

#[cfg(test)]
#[path = "../../tests/unit/similar/bm25.rs"]
mod tests;
