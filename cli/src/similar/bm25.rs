//! Integer BM25 over unit bags (spec §三): k1 = 6/5 and b = 3/4 as
//! rationals folded into one integer fraction, idf in 8-bit fixed
//! point from an integer log2, every per-term contribution floored to
//! 16-bit fixed point — no float anywhere, so the same corpus ranks
//! the same on every platform and the frozen eval docs compare byte
//! for byte. Ranking is written ONCE, against the `Postings` trait:
//! the in-memory `Corpus` the instruments build and the persisted
//! reader over `.ce/index.db` (reader.rs, the product's road) both
//! feed `top_k`, and the replay asserts they agree on every unit. The
//! role conjunction computed here is the instrument's declared MIRROR
//! of what CE.Similar will own once the wire family lands (step 5);
//! the measurement side never decides alone after that.

use super::bag::UnitBag;
use super::terms::Channel;
use anyhow::Result;
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryTerm {
    pub term: u64,
    pub channel: Channel,
    pub weight: i128,
    pub spelled: bool,
}

/// One ranked candidate: the score's integer part (what the frozen
/// eval docs print), the full fixed-point score (what ranks, and what
/// rides the wire as `bm25Num` over `1 << SCORE_FRAC_BITS`), distinct
/// spelled terms shared per channel `[N,P,C,D,S,L]`, shape equality,
/// and the role bit — `nHit ≥ 1 ∧ cHit ≥ 1`, or `nHit ≥ 2 ∧ shape
/// equal` (spec §五): the instrument's mirror of CE.Similar.Cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub doc: usize,
    pub score: i64,
    pub score_fp: i64,
    pub hits: [u32; 6],
    pub shape_equal: bool,
    pub role: bool,
}

/// What ranking needs from an index, by seat — a unit's position in
/// the corpus's (path, key, nth) order: the corpus size and average
/// length, a term's df and posting list, a seat's length, sorted
/// shape terms and identity. Fallible because the persisted reader
/// is; the in-memory corpus never fails.
pub trait Postings {
    fn n_docs(&self) -> usize;
    fn avg_len(&self) -> i128;
    fn df(&self, term: u64) -> Result<usize>;
    /// `(seat, tf)` of every unit carrying `term`.
    fn posting(&self, term: u64) -> Result<Vec<(usize, u32)>>;
    fn len(&self, seat: usize) -> u32;
    fn shape(&self, seat: usize) -> Result<Vec<u64>>;
    fn identity(&self, seat: usize) -> (&str, &str, i64);
}

/// The top `k` candidates for `query`, excluding `exclude` (the
/// query's own seat), ordered by score then identity. A term in more
/// than half the units (idf 0) is neither score nor evidence: sharing
/// what nearly everything shares says nothing, and walking its
/// posting list would cost the whole corpus per query — df is asked
/// first so the list is never fetched. Shape equality and the role
/// bit are read for the k survivors only; neither orders.
pub fn top_k(
    p: &impl Postings,
    query: &[QueryTerm],
    k: usize,
    exclude: Option<usize>,
) -> Result<Vec<Hit>> {
    let avg = p.avg_len();
    let mut acc: HashMap<usize, (i128, [u32; 6])> = HashMap::new();
    for q in query {
        let idf = idf_fp(p.n_docs(), p.df(q.term)?);
        if idf == 0 {
            continue;
        }
        for (seat, tf) in p.posting(q.term)? {
            let e = acc.entry(seat).or_insert((0, [0; 6]));
            let len = i128::from(p.len(seat));
            e.0 += contribution(q.weight, idf, i128::from(tf), len, avg);
            if q.spelled {
                e.1[q.channel.index()] += 1;
            }
        }
    }
    let mut ranked: Vec<(usize, i128, [u32; 6])> = acc
        .into_iter()
        .filter(|(seat, _)| Some(*seat) != exclude)
        .map(|(seat, (score, hits))| (seat, score, hits))
        .collect();
    ranked.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| p.identity(a.0).cmp(&p.identity(b.0)))
    });
    ranked.truncate(k);
    let shape = shape_terms(query);
    ranked
        .into_iter()
        .map(|(doc, score, hits)| {
            let shape_equal = p.shape(doc)? == shape;
            Ok(Hit {
                doc,
                score: i64::try_from(score >> SCORE_FRAC_BITS).expect("score fits i64"),
                // bounded by Σ w·idf·2.2 ≪ 2^63 for any bag the store admits
                score_fp: i64::try_from(score).expect("fixed-point score fits i64"),
                hits,
                shape_equal,
                role: role(&hits, shape_equal),
            })
        })
        .collect()
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
    /// Build the inverted index over `docs` (their order is the seat
    /// order — callers sort by (path, key, nth)).
    pub fn build(docs: Vec<Doc>) -> Corpus {
        let mut postings: HashMap<u64, Vec<(usize, u32)>> = HashMap::new();
        let mut total_len = 0u64;
        for (i, d) in docs.iter().enumerate() {
            total_len += u64::from(d.bag.len());
            for (term, (_, tf)) in &d.bag.terms {
                postings.entry(*term).or_default().push((i, *tf));
            }
        }
        Corpus {
            docs,
            postings,
            total_len,
        }
    }

    /// A unit's own bag as a query: `tf × channel weight × W_UNIT`.
    pub fn query_of(&self, doc: usize) -> Vec<QueryTerm> {
        query_of(&self.docs[doc].bag)
    }
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

    fn identity(&self, seat: usize) -> (&str, &str, i64) {
        let d = &self.docs[seat];
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
