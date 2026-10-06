//! The Stop audit's similar leg (plan v2.29 step 6, spec §六): every
//! unit the session ADDED — a (key, nth) the working tree's file holds
//! and HEAD's did not — is asked of the index the way `ce similar`
//! asks, ranked over rank/1 and its top-K ridden over similar/1 on the
//! audit's core link, and
//! a row written into the feed's `similar` object only when the
//! core's top-1 carries the role bit: an advisor's line for the
//! evaluation ledger, never a reason to block. No core = the object
//! names the degradation (A9f); no new unit, or no role hit and
//! nothing degraded = no `similar` key at all (a Stop that found
//! nothing to say says nothing, so the feed stays the size it was).

use crate::corelink::Link;
use crate::similar::{self, K, UnitBag, bag, corpus, query::place, rank, reader::Reader, wire};
use crate::tombstone::texts::Loaded;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn leg(root: &Path, loaded: &[Loaded], link: Option<&mut Link>) -> Option<Value> {
    let (fresh, bagging) = new_units(loaded);
    if fresh.is_empty() {
        return None;
    }
    // the verdict leg just refreshed this index over the same tree, so
    // the open is a re-read, and the new units sit in it as seats
    let mut rows = Vec::new();
    let (queried, degraded) = match (bagging, crate::dedup::refreshed_index(root, None)) {
        (Some(why), _) => (0, Some(why)),
        (None, Ok((idx, _db))) => ask_all(&idx, &fresh, link, &mut rows),
        (None, Err(e)) => (0, Some(format!("{e:#}"))),
    };
    if rows.is_empty() && degraded.is_none() {
        return None;
    }
    let mut v = json!({
        "rev": similar::SIMILAR_REV,
        "new_units": fresh.len(),
        "queried": queried,
        "rows": rows,
    });
    if let Some(why) = degraded {
        v["degraded"] = json!(why);
    }
    Some(v)
}

/// Every new unit asked in turn: `(units asked, first failure)`. A
/// unit with no candidate at all is not asked; the first refusal —
/// the reader's, the wire's, a missing core (which now ranks, so none
/// is asked without one) — ends the loop by name.
fn ask_all(
    idx: &crate::dedup::index::Index,
    fresh: &[(&str, UnitBag)],
    link: Option<&mut Link>,
    rows: &mut Vec<Value>,
) -> (usize, Option<String>) {
    let reader = match Reader::open(idx) {
        Ok(r) => r,
        Err(e) => return (0, Some(format!("{e:#}"))),
    };
    let Some(link) = link else {
        return (0, Some("core unavailable".into()));
    };
    let mut queried = 0;
    for (rel, bag) in fresh {
        let q = corpus::query_of(bag);
        let ask = rank::Ask {
            query: &q,
            k: K,
            exclude: reader.seat_of(rel, &bag.key, bag.nth),
        };
        let arm = match rank::bare(link, &reader, &ask, None) {
            Ok(Ok(arm)) => arm,
            Ok(Err(why)) => return (queried, Some(why)),
            Err(e) => return (queried, Some(format!("{e:#}"))),
        };
        if arm.hits.is_empty() {
            continue;
        }
        queried += 1;
        match wire::judge(link, &arm.bag, &arm.hits) {
            Ok(j) => {
                let top = j.order[0];
                if j.roles[top] {
                    let twin = &arm.hits[top];
                    rows.push(json!({
                        "unit": format!("{rel}:{}", bag.start_line),
                        "twin": place(&reader.seats()[twin.doc]),
                        "score": twin.score,
                    }));
                }
            }
            Err(why) => return (queried, Some(why)),
        }
    }
    (queried, None)
}

/// The after side's bags whose (key, nth) the before side lacks — the
/// units this change brought into being, by the same throat the index
/// seats them with — bagged by the core in one ask; its refusal is the
/// second half, named.
fn new_units(loaded: &[Loaded]) -> (Vec<(&str, UnitBag)>, Option<String>) {
    let (mut rels, mut rows) = (Vec::new(), Vec::new());
    for l in loaded {
        let before: BTreeSet<(String, i64)> = bag::file_rows(&l.before, l.lang)
            .into_iter()
            .map(|(b, _)| (b.key, b.nth))
            .collect();
        for row in bag::file_rows(&l.after, l.lang) {
            if !before.contains(&(row.0.key.clone(), row.0.nth)) {
                rels.push(l.rel.as_str());
                rows.push(row);
            }
        }
    }
    let unbagged: Vec<UnitBag> = rows.iter().map(|(b, _)| b.clone()).collect();
    match bag::bagged(rows) {
        Ok(bags) => (rels.into_iter().zip(bags).collect(), None),
        Err(why) => (rels.into_iter().zip(unbagged).collect(), Some(why)),
    }
}
