//! The chunked judgment wire machine every core-judgment family
//! drives (clone M5-3e, docdup M5-3g): family bindings, sorted-rank
//! request layout, mirrored-knob pinning, degradation refusal and
//! the lockstep score loop — extracted from the two family drivers
//! when the repo's own ratchet caught the second re-growing the
//! first verbatim (bite seventeen). corelink.rs keeps the Link
//! itself; this module is everything the families say OVER it.

use crate::corelink::Link;
use serde_json::Value;

/// One judgment family's wire bindings: core path, capability name,
/// request kind and chunk ceiling.
pub struct Family<'a> {
    pub core: &'a str,
    pub cap: &'a str,
    pub kind: &'a str,
    pub chunk: usize,
}

/// One decoded reply: request-local `(i, j, metrics)` rows plus the
/// two family counters.
pub type Scored<E> = (Vec<(usize, usize, E)>, [u64; 2]);

/// One whole judgment: sorted global rows, the two accumulated family
/// counters, and the request count.
pub type Judged<E> = (Vec<(usize, usize, E)>, u64, u64, usize);

/// Open the link and demand one capability — the shared head every
/// judgment surface speaks (the lockstep families here, the score
/// wire, the scan wire): the repo's own ratchet caught the third
/// hand-rolled copy when the scan family landed (ADR-008 P3).
pub fn open_family(core: &str, cap: &str) -> anyhow::Result<Link> {
    let (link, _hello) = Link::open(core).map_err(anyhow::Error::msg)?;
    anyhow::ensure!(
        link.has(cap),
        "ce-core offers no {cap} capability — upgrade the core"
    );
    Ok(link)
}

/// Chunked lockstep judging over ONE link: capability gate, chunk,
/// build a request-local body, one request per chunk, map each wire
/// row's endpoints through its chunk's rank order, accumulate the two
/// family counters, sort.
pub fn lockstep_scores<P, E: Ord>(
    fam: &Family<'_>,
    pairs: &[P],
    build: impl Fn(&[P]) -> (Vec<usize>, Value),
    parse: impl Fn(&Value) -> anyhow::Result<Scored<E>>,
) -> anyhow::Result<Judged<E>> {
    let mut link = open_family(fam.core, fam.cap)?;
    let (mut rows, mut c0, mut c1, mut requests) = (Vec::new(), 0, 0, 0);
    for c in pairs.chunks(fam.chunk) {
        let (order, body) = build(c);
        let reply = link.request(fam.kind, body).map_err(anyhow::Error::msg)?;
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
/// family's chunk_request.
pub fn sorted_rank(
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
pub fn pin_knobs(reply: &Value, expect: &[(&str, Value)], owners: &str) -> anyhow::Result<()> {
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
/// mirrors sit above the core's — refuse, naming the module pair.
pub fn refuse_degraded(reply: &Value, mirrors: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        reply["degraded"] == Value::Bool(false),
        "core degraded a client-sized request ({}) — cap mirror drift ({mirrors})",
        reply["reason"]
    );
    Ok(())
}

/// The reply's per-row verdict bits (ADR-008 P1: the core owns the
/// reported-set decision, one bit per score row in row order; scores
/// stay raw for the instruments' cut tables), length-locked to the
/// score rows they qualify — ONE decode throat for every family.
pub fn verdict_bits(reply: &Value, rows: usize) -> anyhow::Result<Vec<bool>> {
    use anyhow::Context;
    let bits: Vec<bool> = serde_json::from_value(reply["verdicts"].clone()).context("verdicts")?;
    anyhow::ensure!(
        bits.len() == rows,
        "core sent {} verdicts for {} score rows",
        bits.len(),
        rows
    );
    Ok(bits)
}

/// A required reply field, its absence named — the third
/// hand-rolled closure of this shape (score, then structure) made
/// it family infrastructure.
pub fn reply_field(reply: &Value, key: &str) -> anyhow::Result<Value> {
    use anyhow::Context;
    reply
        .get(key)
        .cloned()
        .with_context(|| format!("reply missing {key}"))
}

/// The typed sibling: fetch AND decode in one throat. The
/// from_value+context ladder recloned across the wire parsers when
/// the A-layer keys landed (thirteenth ratchet bite) — every table
/// row a family reads now comes through here.
pub fn reply_rows<T: serde::de::DeserializeOwned>(reply: &Value, key: &str) -> anyhow::Result<T> {
    use anyhow::Context;
    serde_json::from_value(reply_field(reply, key)?).with_context(|| format!("decode {key}"))
}

/// The reply's score rows plus the named u64 counters, decoded once.
pub fn scores_and_counts<R: serde::de::DeserializeOwned>(
    reply: &Value,
    keys: &[&str],
) -> anyhow::Result<(Vec<R>, Vec<u64>)> {
    use anyhow::Context;
    let rows: Vec<R> = serde_json::from_value(reply["scores"].clone()).context("scores")?;
    let counts = keys
        .iter()
        .map(|k| {
            reply["counts"][*k]
                .as_u64()
                .with_context(|| format!("counts.{k}"))
        })
        .collect::<anyhow::Result<_>>()?;
    Ok((rows, counts))
}

/// The whole family reply decode in ONE throat: pin the mirrored
/// knobs, refuse degradation, decode rows plus the two named
/// counters. A family's parse_result is one call with its own data.
pub fn parse_scores<R: serde::de::DeserializeOwned>(
    reply: &Value,
    knobs: &[(&str, Value)],
    owners: &str,
    count_keys: &[&str],
) -> anyhow::Result<(Vec<R>, [u64; 2])> {
    pin_knobs(reply, knobs, owners)?;
    refuse_degraded(reply, owners)?;
    let (rows, c) = scores_and_counts(reply, count_keys)?;
    Ok((rows, [c[0], c[1]]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The three reply pins fire generically ONCE here; family wire
    /// tests keep only their own knob lists and bespoke cases.
    #[test]
    fn reply_pins_refuse_drift_and_degradation() {
        let ok = json!({"knobs": {"x": 7}, "degraded": false,
            "scores": [[1, 2]], "counts": {"n": 3}});
        assert!(pin_knobs(&ok, &[("x", json!(7))], "pair").is_ok());
        assert!(pin_knobs(&ok, &[("x", json!(8))], "pair").is_err());
        assert!(refuse_degraded(&ok, "pair").is_ok());
        let mut bad = ok.clone();
        bad["degraded"] = json!(true);
        assert!(refuse_degraded(&bad, "pair").is_err());
        let (rows, counts) = scores_and_counts::<[u64; 2]>(&ok, &["n"]).expect("decode");
        assert_eq!((rows, counts), (vec![[1, 2]], vec![3]));
    }

    /// The verdict-bit throat (ADR-008 P1) fires generically ONCE
    /// here: bits decode in row order, a truncated or missing array
    /// refuses — an unqualified score row must never default.
    #[test]
    fn verdict_bits_decode_and_length_lock() {
        let ok = json!({"verdicts": [true, false]});
        assert_eq!(verdict_bits(&ok, 2).expect("bits"), vec![true, false]);
        assert!(verdict_bits(&ok, 3).is_err(), "truncated bits refuse");
        assert!(
            verdict_bits(&json!({}), 0).is_err(),
            "missing verdicts refuses even for zero rows"
        );
    }
}
