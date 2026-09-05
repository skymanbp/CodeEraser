//! The Stop audit's similar leg (plan v2.29 step 6, spec §六): every
//! unit the session ADDED — a (key, nth) the working tree's file holds
//! and HEAD's did not — is asked of the index the way `ce similar`
//! asks, its top-K ridden over similar/1 on the audit's core link, and
//! a row written into the feed's `similar` object only when the
//! core's top-1 carries the role bit: an advisor's line for the
//! evaluation ledger, never a reason to block. No core = the object
//! names the degradation (A9f); no new unit, or no role hit and
//! nothing degraded = no `similar` key at all (a Stop that found
//! nothing to say says nothing, so the feed stays the size it was).

use crate::corelink::Link;
use crate::similar::{self, K, UnitBag, bm25, file_bags, query::place, reader::Reader, wire};
use crate::tombstone::texts::Loaded;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn leg(root: &Path, loaded: &[Loaded], link: Option<&mut Link>) -> Option<Value> {
    let fresh = new_units(loaded);
    if fresh.is_empty() {
        return None;
    }
    // the verdict leg just refreshed this index over the same tree, so
    // the open is a re-read, and the new units sit in it as seats
    let mut rows = Vec::new();
    let (queried, degraded) = match crate::dedup::refreshed_index(root, None) {
        Ok((idx, _db)) => ask_all(&idx, &fresh, link, &mut rows),
        Err(e) => (0, Some(format!("{e:#}"))),
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
/// the reader's, the wire's, a missing core — ends the loop by name.
fn ask_all(
    idx: &crate::dedup::index::Index,
    fresh: &[(&str, UnitBag)],
    mut link: Option<&mut Link>,
    rows: &mut Vec<Value>,
) -> (usize, Option<String>) {
    let reader = match Reader::open(idx) {
        Ok(r) => r,
        Err(e) => return (0, Some(format!("{e:#}"))),
    };
    let mut queried = 0;
    for (rel, bag) in fresh {
        let q = bm25::query_of(bag);
        let hits = match bm25::top_k(&reader, &q, K, reader.seat_of(rel, &bag.key, bag.nth)) {
            Ok(h) => h,
            Err(e) => return (queried, Some(format!("{e:#}"))),
        };
        if hits.is_empty() {
            continue;
        }
        queried += 1;
        let judged = match link.as_deref_mut() {
            Some(l) => wire::judge(l, &q, &hits),
            None => Err("core unavailable".into()),
        };
        match judged {
            Ok(j) => {
                let top = j.order[0];
                if j.roles[top] {
                    let twin = &hits[top];
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
/// seats them with.
fn new_units(loaded: &[Loaded]) -> Vec<(&str, UnitBag)> {
    let mut out = Vec::new();
    for l in loaded {
        let before: BTreeSet<(String, i64)> = file_bags(&l.before, l.lang)
            .into_iter()
            .map(|b| (b.key, b.nth))
            .collect();
        out.extend(
            file_bags(&l.after, l.lang)
                .into_iter()
                .filter(|b| !before.contains(&(b.key.clone(), b.nth)))
                .map(|b| (l.rel.as_str(), b)),
        );
    }
    out
}
