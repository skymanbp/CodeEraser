//! The similar/1 leg of the measurement (plan v2.29 step 5): the rows
//! the core judges, the request, and the reply consumed back onto the
//! candidates' seats. Rust ranks off its own tables and computes no
//! policy here — the order the candidates stand in and which of them
//! play the query's role come back on the wire (ADR-008 sixth
//! instalment); this side re-labels the indices it sent. Every failure
//! is a NAMED non-judgment, never conflated with "no candidates" (A9f).

use super::bm25::{Hit, QueryTerm, SCORE_FRAC_BITS};
use crate::corelink::{Link, judged};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The capability the core must offer, and the request kind.
pub const CAP: &str = "similar/1";
pub const KIND: &str = "similar";

/// The core's judgment of one query: the candidates (as indices into
/// the rows sent) in judged order, and the role bit per row in request
/// order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Judged {
    pub order: Vec<usize>,
    pub roles: Vec<bool>,
}

/// A judgment, or the named reason there is none: no core, a core
/// without the family, a degraded reply, wire skew.
pub type Judgment = Result<Judged, String>;

/// One candidate's wire row: `[nHit, pHit, cHit, dHit, sHit, lHit,
/// shapeEqual, bm25Num, bm25Den]` — the six channel hits, the shape
/// bit, and the fixed-point score over its unit (the core compares the
/// fraction and never learns the width).
pub fn row(h: &Hit) -> [i64; 9] {
    let [n, p, c, d, s, l] = h.hits.map(i64::from);
    let den = 1i64 << SCORE_FRAC_BITS;
    [n, p, c, d, s, l, i64::from(h.shape_equal), h.score_fp, den]
}

/// The wire rows, in the measurement's order (row index is identity on
/// the wire, so the core's tie order is this order).
pub fn rows(hits: &[Hit]) -> Vec<[i64; 9]> {
    hits.iter().map(row).collect()
}

/// The query bag on the wire: `[termHash, weight]` per distinct term,
/// hashes strictly ascending (a bag is a set — the core refuses a
/// repeat), a term's weight summed over its appearances.
pub fn query_terms(query: &[QueryTerm]) -> Vec<[u64; 2]> {
    let mut bag: BTreeMap<u64, u64> = BTreeMap::new();
    for q in query {
        let w = u64::try_from(q.weight).expect("query weight is positive and small");
        *bag.entry(q.term).or_default() += w;
    }
    bag.into_iter().map(|(term, w)| [term, w]).collect()
}

/// The request body: the query bag and the candidate rows.
pub fn body(query: &[[u64; 2]], rows: &[[i64; 9]]) -> Value {
    json!({ "query": query, "rows": rows })
}

/// One request over a link past its handshake, behind the capability
/// gate (a pre-6.7.0 core is healthy and answers nothing here).
pub fn ask(link: &mut Link, query: &[[u64; 2]], rows: &[[i64; 9]]) -> Result<Value, String> {
    judged::ask(link, CAP, "6.7.0", KIND, body(query, rows))
}

/// The whole leg over one link: the query and its ranked candidates
/// sent, the reply consumed (below).
pub fn judge(link: &mut Link, query: &[QueryTerm], hits: &[Hit]) -> Judgment {
    let rows = rows(hits);
    consume(&ask(link, &query_terms(query), &rows)?, rows.len())
}

/// The reply, consumed: a degraded reply is a named non-judgment; an
/// order that is not a permutation of the rows sent, a role table of
/// another length or with a non-boolean, or counts that disagree with
/// the tables are wire skew (a malformed reply is never a healthy one);
/// the rest is relayed as the core said.
pub fn consume(reply: &Value, sent: usize) -> Judgment {
    judged::degraded(reply)?;
    let (order, roles): (Vec<usize>, Vec<bool>) = (
        judged::table(reply, "order")?,
        judged::table(reply, "roles")?,
    );
    let mut seen = order.clone();
    seen.sort_unstable();
    if seen != (0..sent).collect::<Vec<_>>() {
        return Err("wire skew: order must be a permutation of the rows sent".into());
    }
    if roles.len() != sent {
        return Err("wire skew: one role bit per row sent".into());
    }
    let role = roles.iter().filter(|r| **r).count();
    if judged::count(reply, "rows")? != sent || judged::count(reply, "role")? != role {
        return Err("wire skew: counts disagree with the tables".into());
    }
    Ok(Judged { order, roles })
}

#[cfg(test)]
#[path = "../../tests/unit/similar/wire.rs"]
mod tests;
