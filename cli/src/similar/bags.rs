//! The bags/1 leg (plan v2.33 W6; booklet 15 §1): the term road is the
//! core's (CE.Similar.Bags — identifier splitting, the stop list, Porter's
//! stemmer, the channel-tagged hashes, the shape spellings, the bag's
//! insertion order). This side reads the tree and sends what it read as
//! text — per unit the key, its kind word and return flag, the callee
//! spellings, the literal kinds, the structure histogram and the lines of
//! the comments it owns (bag.rs) — and a free-text query as it was typed
//! (query.rs); the reply's `[term, channel, tf]` rows are what the stored
//! tables hold. Index refreshes and queries ride the process's core
//! session (corelink::judged::session_ask). A core that cannot answer is
//! a named refusal: there is no copy of the road on this side to fall
//! back on.

use super::terms::Channel;
use crate::corelink::judged;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The capability the core must offer, the request kind, the proto that
/// minted the family.
pub const CAP: &str = "bags/1";
const KIND: &str = "bags";
const SINCE: &str = "9.0.0";

/// term → (channel, tf): one bag as the stored tables hold it.
pub type Terms = BTreeMap<u64, (Channel, u32)>;

/// The bag of every unit row and of every text, in the order sent.
pub fn ask(units: Vec<Value>, texts: &[&str]) -> Result<(Vec<Terms>, Vec<Terms>), String> {
    let sent = (units.len(), texts.len());
    if sent == (0, 0) {
        return Ok((Vec::new(), Vec::new()));
    }
    let reply = request(json!({ "units": units, "texts": texts }))?;
    let bags: Vec<Vec<(u64, usize, u32)>> = judged::table(&reply, "bags")?;
    let text_bags: Vec<Vec<(u64, usize, u32)>> = judged::table(&reply, "texts")?;
    if (bags.len(), text_bags.len()) != sent {
        return Err(format!(
            "{CAP}: wire skew: one bag per unit and per text sent"
        ));
    }
    Ok((terms(bags)?, terms(text_bags)?))
}

/// The rows of each bag as terms; a channel past the evidence row is
/// wire skew, named.
fn terms(bags: Vec<Vec<(u64, usize, u32)>>) -> Result<Vec<Terms>, String> {
    bags.into_iter()
        .map(|rows| {
            rows.into_iter()
                .map(|(term, c, tf)| match Channel::ALL.get(c) {
                    Some(ch) => Ok((term, (*ch, tf))),
                    None => Err(format!("{CAP}: wire skew: channel {c}")),
                })
                .collect()
        })
        .collect()
}

/// One request over the process's core session (the reader legs of the
/// differential gate ask `inspect` through it too).
fn request(body: Value) -> Result<Value, String> {
    judged::session_ask((CAP, SINCE, KIND), body)
}

#[cfg(test)]
#[path = "../../tests/unit/similar/bags_diff.rs"]
mod tests;
