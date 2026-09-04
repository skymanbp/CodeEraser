//! The tombstone/1 leg of the measurement (plan v2.27 step 4): the
//! rows the core judges, the request, and the reply consumed back
//! into places. Rust computes no policy here — which rows are sites,
//! the label / prose split and the budget condition all come back on
//! the wire (ADR-008 fifth instalment); this side re-labels the
//! indices it sent. Every failure is a NAMED non-judgment, never
//! conflated with "no sites" (A9f).

use super::Findings;
use crate::corelink::Link;
use serde_json::{Value, json};

/// The capability the core must offer, and the request kind.
pub const CAP: &str = "tombstone/1";
pub const KIND: &str = "tombstone";

/// The core's judgment of one measurement: the site rows by index,
/// their split, and whether the declared budget is exceeded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Judged {
    pub sites: Vec<usize>,
    pub label: usize,
    pub prose: usize,
    pub over: bool,
}

/// A judgment, or the named reason there is none: no core, a core
/// without the family, a degraded reply, wire skew.
pub type Judgment = Result<Judged, String>;

/// The wire rows: `[kind, marks, erasedNames]` per candidate, in the
/// measurement's order (row index is identity on the wire).
pub fn rows(f: &Findings) -> Vec<[u64; 3]> {
    f.rows
        .iter()
        .map(|r| [r.kind as u64, r.marks as u64, r.names as u64])
        .collect()
}

/// The request body: the rows, and the budget as knob 0 when declared
/// (absent = the core evaluates no condition).
pub fn body(rows: &[[u64; 3]], budget: Option<u32>) -> Value {
    let knobs: Vec<[u64; 2]> = budget.map(|b| [0, u64::from(b)]).into_iter().collect();
    json!({ "rows": rows, "knobs": knobs })
}

/// One request over a link past its handshake: the capability gate
/// first — a pre-6.6.0 core is healthy and answers nothing here.
pub fn ask(link: &mut Link, rows: &[[u64; 3]], budget: Option<u32>) -> Result<Value, String> {
    if !link.has(CAP) {
        return Err(format!("core offers no {CAP} (pre-6.6.0)"));
    }
    link.request(KIND, body(rows, budget))
}

/// The reply, consumed: a degraded reply is a named non-judgment, a
/// site index beyond the rows sent is wire skew, everything else is
/// relayed as the core said it.
pub fn consume(reply: &Value, sent: usize) -> Judgment {
    if reply["degraded"] == json!(true) {
        let why = reply["reason"].as_str().unwrap_or("degraded");
        return Err(why.to_string());
    }
    let sites: Vec<usize> =
        serde_json::from_value(reply["sites"].clone()).map_err(|e| format!("sites: {e}"))?;
    if sites.iter().any(|&i| i >= sent) {
        return Err("wire skew: a site index beyond the rows sent".into());
    }
    let count = |k: &str| {
        reply["counts"][k]
            .as_u64()
            .map(|n| n as usize)
            .ok_or_else(|| format!("counts.{k} missing"))
    };
    Ok(Judged {
        sites,
        label: count("label")?,
        prose: count("prose")?,
        over: reply["over"] == json!(true),
    })
}

/// The whole leg over one link: rows sent, reply consumed.
pub fn judge(link: &mut Link, f: &Findings, budget: Option<u32>) -> Judgment {
    let rows = rows(f);
    consume(&ask(link, &rows, budget)?, rows.len())
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone/wire.rs"]
mod tests;
