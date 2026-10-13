//! The chunked judgment wire machine every core-judgment family
//! drives (clone M5-3e, docdup M5-3g): family bindings, sorted-rank
//! request layout, mirrored-knob pinning, degradation refusal and
//! the lockstep score loop — extracted from the two family drivers
//! when the repo's own ratchet caught the second re-growing the
//! first verbatim (bite seventeen). corelink.rs keeps the Link
//! itself and corelink::judged the reply kit (open, ask, degraded,
//! table, field, count) this loop reads its replies with.

use crate::corelink::Link;
use crate::corelink::judged;
use serde_json::Value;

/// One judgment family's wire bindings: core path, capability name,
/// request kind and chunk ceiling.
pub struct Family<'a> {
    pub core: &'a str,
    pub cap: &'a str,
    pub kind: &'a str,
    pub chunk: usize,
}

impl<'a> Family<'a> {
    /// A family's bindings: core path, capability, request kind, chunk.
    pub fn new(core: &'a str, cap: &'a str, kind: &'a str, chunk: usize) -> Self {
        Family {
            core,
            cap,
            kind,
            chunk,
        }
    }
}

/// The knobs a family pins, as parallel lists: each wire name with the
/// value of the package this run read.
pub fn pins<const N: usize>(
    names: [&'static str; N],
    values: [i64; N],
) -> Vec<(&'static str, Value)> {
    names.into_iter().zip(values.map(Value::from)).collect()
}

/// One decoded reply: request-local `(i, j, metrics)` rows plus the
/// two family counters.
pub type Scored<E> = (Vec<(usize, usize, E)>, [u64; 2]);

/// One whole judgment: sorted global rows, the two accumulated family
/// counters, and the request count.
pub type Judged<E> = (Vec<(usize, usize, E)>, u64, u64, usize);

/// Chunked lockstep judging over ONE link the caller opened through
/// `judged::open` (plan v2.30 step 5b-9: the T3 family reads the
/// core's proto off the link before deciding what its verdict cache
/// may answer, then sends only the rest): chunk, build a
/// request-local body, one request per chunk, map each wire row's
/// endpoints through its chunk's rank order, accumulate the two
/// family counters, sort.
pub fn lockstep_scores<P, E: Ord>(
    link: &mut Link,
    fam: &Family<'_>,
    pairs: &[P],
    build: impl Fn(&[P]) -> (Vec<usize>, Value),
    parse: impl Fn(&Value) -> anyhow::Result<Scored<E>>,
) -> anyhow::Result<Judged<E>> {
    let (mut rows, mut c0, mut c1, mut requests) = (Vec::new(), 0, 0, 0);
    for c in pairs.chunks(fam.chunk) {
        let (order, body) = build(c);
        let reply = judged::ask(link, fam.cap, fam.kind, body)?;
        let (ws, counts) = parse(&reply)?;
        c0 += counts[0];
        c1 += counts[1];
        // `order[i]` on a CORE-supplied rank: request endpoints are
        // contract-checked (Clone.hs, Docdup.hs), the reply's were not.
        for (i, j, e) in ws {
            let (Some(&a), Some(&b)) = (order.get(i), order.get(j)) else {
                anyhow::bail!("{} reply: endpoint rank {i}/{j} out of range", fam.kind);
            };
            rows.push((a, b, e));
        }
        requests += 1;
    }
    rows.sort_unstable();
    Ok((rows, c0, c1, requests))
}

/// Request-local layout by sorted rank: the distinct endpoint ids in
/// ascending order plus the id→rank map — the monotone map keeps the
/// wire's strictly-ascending pair rows for free. ONE throat for every
/// family's chunk_request (chunk_layout below).
fn sorted_rank(
    ends: impl Iterator<Item = (usize, usize)>,
) -> (Vec<usize>, std::collections::BTreeMap<usize, usize>) {
    let order: Vec<usize> = ends
        .flat_map(|(a, b)| [a, b])
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let rank = order.iter().enumerate().map(|(r, &g)| (g, r)).collect();
    (order, rank)
}

/// Pin every mirrored knob in a reply to its Rust copy — a drift is
/// an error NAMING the owning module pair, never a silent score.
fn pin_knobs(reply: &Value, expect: &[(&str, Value)], owners: &str) -> anyhow::Result<()> {
    for (key, want) in expect {
        anyhow::ensure!(
            reply["knobs"][*key] == *want,
            "core {key} {} disagrees with the Rust mirror {want} — one number, two owners ({owners})",
            reply["knobs"][*key]
        );
    }
    Ok(())
}

/// A degraded reply to a client-sized request means the Rust cap
/// mirrors sit above the core's — refuse, naming the module pair. The
/// flag must be stated: a reply without it is no client-sized answer.
pub fn refuse_degraded(reply: &Value, mirrors: &str) -> anyhow::Result<()> {
    if reply["degraded"] == Value::Bool(false) {
        return Ok(());
    }
    let why = judged::degraded(reply)
        .err()
        .map_or("no degraded flag".into(), String::from);
    anyhow::bail!("core degraded a client-sized request ({why}) — cap mirror drift ({mirrors})")
}

/// A per-row column of the reply under `key` (the verdict bits of
/// ADR-008 P1, docdup's measured runs, clone/1's decided bits),
/// length-locked to the `rows` it qualifies — ONE decode throat for
/// every family.
pub fn rows_for<T: serde::de::DeserializeOwned>(
    reply: &Value,
    key: &str,
    rows: usize,
) -> anyhow::Result<Vec<T>> {
    let col: Vec<T> = judged::table(reply, key)?;
    anyhow::ensure!(
        col.len() == rows,
        "core sent {} {key} for {rows} score rows",
        col.len()
    );
    Ok(col)
}

/// One chunk's request-local layout over the shared sorted-rank throat:
/// the global ids in rank order, each one's item, and the pairs as
/// local `[i, j]` — the layout every pairwise family sends.
pub fn chunk_layout<'t, T: ?Sized>(
    pairs: &[(usize, usize)],
    item_of: impl Fn(usize) -> &'t T,
) -> (Vec<usize>, Vec<&'t T>, Vec<[usize; 2]>) {
    let (order, rank) = sorted_rank(pairs.iter().copied());
    let items = order.iter().map(|&g| item_of(g)).collect();
    let local = pairs.iter().map(|&(a, b)| [rank[&a], rank[&b]]).collect();
    (order, items, local)
}

/// The reply's score rows plus the named counters, decoded once.
fn scores_and_counts<R: serde::de::DeserializeOwned>(
    reply: &Value,
    keys: &[&str],
) -> anyhow::Result<(Vec<R>, Vec<u64>)> {
    let rows: Vec<R> = judged::table(reply, "scores")?;
    let counts = keys
        .iter()
        .map(|k| judged::count(reply, k).map(|n| n as u64))
        .collect::<Result<_, _>>()?;
    Ok((rows, counts))
}

/// The whole family reply decode in ONE throat: pin the mirrored
/// knobs, refuse degradation, decode the score rows plus the two
/// named counters, zip each row with the core's verdict bit and shape
/// it with `row`. A family's parse_result is one call with its own
/// data — the zip-and-shape tail was the two families' last clone
/// pair (v2.18 subtraction batch).
pub fn parse_scores<R: serde::de::DeserializeOwned, E>(
    reply: &Value,
    knobs: &[(&str, Value)],
    owners: &str,
    count_keys: &[&str],
    row: impl Fn(R, bool) -> (usize, usize, E),
) -> anyhow::Result<Scored<E>> {
    pin_knobs(reply, knobs, owners)?;
    refuse_degraded(reply, owners)?;
    let (rows, c) = scores_and_counts::<R>(reply, count_keys)?;
    let bits: Vec<bool> = rows_for(reply, "verdicts", rows.len())?;
    let local = rows.into_iter().zip(bits).map(|(r, v)| row(r, v)).collect();
    Ok((local, [c[0], c[1]]))
}

#[cfg(test)]
#[path = "../tests/unit/lockstep.rs"]
mod tests;
