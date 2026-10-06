//! The bags/1 leg (plan v2.33 W6; booklet 15 §1): the term road is the
//! core's (CE.Similar.Bags — identifier splitting, the stop list, Porter's
//! stemmer, the channel-tagged hashes, the shape spellings, the bag's
//! insertion order). This side reads the tree and sends what it read as
//! text — per unit the key, its kind word and return flag, the callee
//! spellings, the literal kinds, the structure histogram and the lines of
//! the comments it owns (bag.rs) — and a free-text query as it was typed
//! (query.rs); the reply's `[term, channel, tf]` rows are what the stored
//! tables hold. Index refreshes and queries share the process's link,
//! opened on first use. A core that cannot answer is a named refusal:
//! there is no copy of the road on this side to fall back on.

use super::terms::Channel;
use crate::corelink::{Link, judged};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

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

/// The process's link to the core: taken out for one request and put
/// back only when it answered, so a failed link is dropped and the next
/// request opens a fresh one.
static LINK: Mutex<Option<Link>> = Mutex::new(None);

fn request(body: Value) -> Result<Value, String> {
    let mut held = LINK.lock().unwrap_or_else(PoisonError::into_inner);
    let mut link = match held.take() {
        Some(link) => link,
        None => Link::open(crate::tables::core_flag())?.0,
    };
    let reply =
        judged::ask(&mut link, CAP, SINCE, KIND, body).map_err(|e| format!("{CAP}: {e}"))?;
    *held = Some(link);
    judged::degraded(&reply).map_err(|e| format!("{CAP}: {e}"))?;
    Ok(reply)
}

#[cfg(test)]
#[path = "../../tests/unit/similar/bags_diff.rs"]
mod tests;
