//! The similar/1 leg of the measurement (plan v2.29 step 5): the rows
//! the core judges, the request, and the reply consumed back onto the
//! candidates' seats. The core ranked the candidates (rank/1, rank.rs)
//! and this side measured their shape; the order the candidates stand
//! in and which of them play the query's role come back on the wire
//! (ADR-008 sixth instalment); this side re-labels the indices it sent.
//! Every failure is a NAMED non-judgment, never conflated with "no
//! candidates" (A9f).

use super::rank::Hit;
use crate::corelink::{Link, judged};
use serde_json::{Value, json};

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
/// bit, and the fixed-point score over the denominator rank/1 answered.
pub fn row(h: &Hit) -> [i64; 9] {
    let [n, p, c, d, s, l] = h.hits.map(i64::from);
    [
        n,
        p,
        c,
        d,
        s,
        l,
        i64::from(h.shape_equal),
        h.score_fp,
        h.den,
    ]
}

/// The wire rows, in the measurement's order (row index is identity on
/// the wire, so the core's tie order is this order).
pub fn rows(hits: &[Hit]) -> Vec<[i64; 9]> {
    hits.iter().map(row).collect()
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

/// The whole leg over one link: the weighted bag rank/1 ranked
/// (`[termHash, weight]`, hashes ascending) and its kept candidates
/// sent, the reply consumed (below).
pub fn judge(link: &mut Link, bag: &[[u64; 2]], hits: &[Hit]) -> Judgment {
    let rows = rows(hits);
    consume(&ask(link, bag, &rows)?, rows.len())
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
