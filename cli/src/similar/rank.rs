//! The rank/1 leg (plan v2.33 W3; booklet 15): what the ranking needs,
//! fetched off a `Postings` source and sent as integers — the query
//! bag `[term, channel, tf]`, each asked term's df, the postings of the
//! terms that score, the candidate seats' lengths and, for the widened
//! view, the words' unit counts and co-occurrence rows (`Cooc`). The
//! core weights, widens, scores and cuts (CE.Similar.Rank); this side
//! keeps the text → term road, the stored tables and the shape
//! comparison of the kept seats, and fetches by the package's two
//! ratios and co-occurrence floor so a term in half the corpus never
//! costs its posting list. The in-memory `Corpus` the instruments build
//! and the persisted reader over `.ce/index.db` (reader.rs) are two
//! sources of ONE request: the replay asserts they send the same bytes.

use super::ppmi::Cooc;
use super::terms::Channel;
use crate::corelink::{Link, judged};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// The capability the core must offer, the request kind, and the proto
/// that minted the family.
pub const CAP: &str = "rank/1";
const KIND: &str = "rank";
const SINCE: &str = "8.3.0";

/// One query term as spelled: hash, channel, term frequency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryTerm {
    pub term: u64,
    pub channel: Channel,
    pub tf: u32,
}

/// One kept candidate: the score's integer part (what the frozen eval
/// docs print), the fixed-point score over its denominator (what the
/// role judgment orders by), distinct spelled terms shared per channel
/// `[N,P,C,D,S,L]`, and whether its shape terms equal the query's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub doc: usize,
    pub score: i64,
    pub score_fp: i64,
    pub den: i64,
    pub hits: [u32; 6],
    pub shape_equal: bool,
}

/// One arm as the core answered it: the kept hits in rank order, the
/// expansion it found (when co-occurrence rows rode), and the weighted
/// bag it ranked — what similar/1 is asked about.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Arm {
    pub hits: Vec<Hit>,
    pub added: Vec<u64>,
    pub bag: Vec<[u64; 2]>,
}

/// What ranking needs from an index, by seat — a unit's position in
/// the corpus's (path, key, nth) order, which is the tie order: the
/// corpus size and average length, a term's df and posting list, a
/// seat's length and sorted shape terms. Fallible because the persisted
/// reader is; the in-memory corpus never fails.
pub trait Postings {
    fn n_docs(&self) -> usize;
    fn avg_len(&self) -> i128;
    fn df(&self, term: u64) -> Result<usize>;
    /// `(seat, tf)` of every unit carrying `term`.
    fn posting(&self, term: u64) -> Result<Vec<(usize, u32)>>;
    fn len(&self, seat: usize) -> u32;
    fn shape(&self, seat: usize) -> Result<Vec<u64>>;
}

/// The widened view's facts: `[term, marg]` per spelled word term and
/// `[a, b, nab, margB]` per co-occurrence above the package's floor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CoocRows {
    words: Vec<[u64; 2]>,
    pairs: Vec<[u64; 4]>,
}

/// One arm to rank: the query, how many to keep, the query's own seat.
pub struct Ask<'a> {
    pub query: &'a [QueryTerm],
    pub k: usize,
    pub exclude: Option<usize>,
}

/// A ranked arm, or the named reason the core gave none (A9f); the outer
/// error is the source's.
pub type Ranked = Result<Result<Arm, String>>;

/// The bare arm; with `cooc` the reply also names the expansion.
pub fn bare(link: &mut Link, p: &impl Postings, ask: &Ask, cooc: Option<&CoocRows>) -> Ranked {
    let body = request(p, ask, &[], cooc, false, &[])?;
    Ok(answer(link, p, ask, body))
}

/// The widened arm: the bare query plus `added` (the bare arm's
/// expansion), the seats of `seen` dropped after the cut.
pub fn widened(
    link: &mut Link,
    p: &impl Postings,
    ask: &Ask,
    cooc: &CoocRows,
    added: &[u64],
    seen: &[usize],
) -> Ranked {
    let body = request(p, ask, added, Some(cooc), true, seen)?;
    Ok(answer(link, p, ask, body))
}

fn answer(link: &mut Link, p: &impl Postings, ask: &Ask, body: Value) -> Result<Arm, String> {
    let reply = judged::ask(link, CAP, SINCE, KIND, body)?;
    let mut arm = consume(&reply, p.n_docs(), ask.k)?;
    let shape = shape_terms(ask.query);
    for h in &mut arm.hits {
        h.shape_equal = p.shape(h.doc).map_err(|e| format!("{e:#}"))? == shape;
    }
    Ok(arm)
}

/// The co-occurrence facts of a query: every spelled word term's unit
/// count and, for a word that can have a neighbour, its pairs at or
/// above the floor with the partner's count.
pub fn cooc_rows(c: &impl Cooc, query: &[QueryTerm]) -> Result<CoocRows> {
    let lim = &crate::tables::get().limits.similar;
    let n = u64::from(c.n_units());
    let mut rows = CoocRows::default();
    for q in query.iter().filter(|q| q.channel.is_words()) {
        let marg = u64::from(c.n_term(q.term)?);
        rows.words.push([q.term, marg]);
        if marg == 0 || lim.neighbour_df_ratio * marg > n {
            continue;
        }
        for (b, nab) in c.pairs(q.term)? {
            if u64::from(nab) >= lim.min_cooc {
                rows.pairs
                    .push([q.term, b, u64::from(nab), u64::from(c.n_term(b)?)]);
            }
        }
    }
    rows.words.sort_unstable();
    rows.pairs.sort_unstable();
    Ok(rows)
}

/// The request body: the asked terms are the query's and `extra`; a
/// term that scores (n > ratio · df) carries its postings, sorted by
/// seat, and every seat they name its length.
pub fn request(
    p: &impl Postings,
    ask: &Ask,
    extra: &[u64],
    cooc: Option<&CoocRows>,
    widen: bool,
    seen: &[usize],
) -> Result<Value> {
    let ratio = crate::tables::get().limits.similar.scored_df_ratio;
    let n = p.n_docs();
    let asked: BTreeSet<u64> = ask
        .query
        .iter()
        .map(|q| q.term)
        .chain(extra.iter().copied())
        .collect();
    let (mut terms, mut postings) = (Vec::new(), Vec::new());
    let mut lens: BTreeMap<usize, u32> = BTreeMap::new();
    for t in asked {
        let df = p.df(t)?;
        terms.push(json!([t, df]));
        if n as u64 > ratio * df as u64 {
            let mut list = p.posting(t)?;
            list.sort_unstable();
            for (seat, tf) in list {
                lens.insert(seat, p.len(seat));
                postings.push(json!([t, seat, tf]));
            }
        }
    }
    let query: Vec<Value> = ask
        .query
        .iter()
        .map(|q| json!([q.term, q.channel.index(), q.tf]))
        .collect();
    let mut seen = seen.to_vec();
    seen.sort_unstable();
    let mut body = json!({
        "n": n, "avg": p.avg_len() as i64, "k": ask.k, "seen": seen, "query": query, "widen": widen,
        "terms": terms, "postings": postings,
        "lens": lens.into_iter().map(|(s, l)| [s as u64, u64::from(l)]).collect::<Vec<_>>(),
    });
    if let Some(seat) = ask.exclude {
        body["exclude"] = json!(seat);
    }
    if let Some(c) = cooc {
        (body["words"], body["pairs"]) = (json!(c.words), json!(c.pairs));
    }
    Ok(body)
}

/// The reply, consumed: a degraded reply is a named non-judgment; more
/// hits than asked, a seat out of range or twice, a channel out of range
/// or a non-positive denominator is wire skew.
fn consume(reply: &Value, n: usize, k: usize) -> Result<Arm, String> {
    judged::degraded(reply)?;
    let rows: Vec<Vec<i64>> = judged::table(reply, "hits")?;
    let den: i64 = judged::table(reply, "scoreDen")?;
    let added: Vec<[u64; 2]> = judged::table(reply, "added")?;
    let bag: Vec<[u64; 2]> = judged::table(reply, "query")?;
    let mut seats = BTreeSet::new();
    let mut hits = Vec::new();
    for row in &rows {
        let [seat, score, fp, h @ ..] = row.as_slice() else {
            return Err("wire skew: a hit is [seat,score,scoreNum,N,P,C,D,S,L]".into());
        };
        let (Ok(seat), Ok(h)) = (usize::try_from(*seat), <[i64; 6]>::try_from(h)) else {
            return Err("wire skew: a hit is [seat,score,scoreNum,N,P,C,D,S,L]".into());
        };
        if seat >= n || !seats.insert(seat) {
            return Err("wire skew: a hit seat out of range or repeated".into());
        }
        hits.push(Hit {
            doc: seat,
            score: *score,
            score_fp: *fp,
            den,
            hits: h.map(|x| x as u32),
            shape_equal: false,
        });
    }
    if hits.len() > k || den < 1 || added.iter().any(|a| a[1] >= Channel::ALL.len() as u64) {
        return Err("wire skew: more hits than asked, a denominator or a channel".into());
    }
    Ok(Arm {
        hits,
        added: added.into_iter().map(|[t, _]| t).collect(),
        bag,
    })
}

/// The sorted spelled shape terms of a query.
fn shape_terms(query: &[QueryTerm]) -> Vec<u64> {
    let mut v: Vec<u64> = query
        .iter()
        .filter(|q| q.channel == Channel::Shape)
        .map(|q| q.term)
        .collect();
    v.sort_unstable();
    v
}

#[cfg(test)]
#[path = "../../tests/unit/similar/rank.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/similar/rank_differential.rs"]
mod differential;
