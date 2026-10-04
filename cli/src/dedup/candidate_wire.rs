//! The candidates/1 leg of the T3 candidate pass (plan v2.33 W3): the
//! three index-bound sources' union and every admitted unit go to the
//! core as integers — each unit its language, file, key, node count and
//! kind histogram, every one a request-local code — and the core adds
//! the same-key source S2, answers the pairs the two admissible bounds
//! keep (§4.3), with the exhaustive source S5's new pairs when asked,
//! and the tally. A pair's endpoints are unit ids both ways; the
//! languages come back by code and are named here.

use super::candidates::{PairRow, Tally, Unit};
use crate::corelink::{Link, judged};
use anyhow::{Result, bail, ensure};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The capability the core must offer, and the request kind.
pub const CAP: &str = "candidates/1";
const KIND: &str = "candidates";

/// The proto that minted the family (named when a core lacks it).
const SINCE: &str = "7.12.0";

/// Request-local codes in first-seen order.
struct Codes<K: Ord>(BTreeMap<K, i64>);

impl<K: Ord> Codes<K> {
    fn new() -> Self {
        Codes(BTreeMap::new())
    }

    fn of(&mut self, k: K) -> i64 {
        let next = self.0.len() as i64;
        *self.0.entry(k).or_insert(next)
    }
}

/// The request body and the language names by code: units as `[lang,
/// file, key, nodes, kind, count, …]` (every code dense in first-seen
/// order across the units, each histogram sorted by kind code), the
/// union as `[a, b, sources]` ascending.
fn body<'u>(
    units: &'u [Unit],
    union: &BTreeMap<(usize, usize), u8>,
    exhaustive: bool,
) -> (Value, Vec<&'u str>) {
    let (mut langs, mut files, mut keys) = (Codes::new(), Codes::new(), Codes::new());
    let mut kinds: Codes<u64> = Codes::new();
    let rows: Vec<Vec<i64>> = units
        .iter()
        .map(|u| {
            let head = [
                langs.of(u.lang.as_str()),
                files.of(u.path.as_str()),
                keys.of(u.key.as_str()),
                u.nodes,
            ];
            let mut hist: Vec<(i64, i64)> = u
                .hist
                .iter()
                .map(|&(k, c)| (kinds.of(k), i64::from(c)))
                .collect();
            hist.sort_unstable();
            let mut row = head.to_vec();
            row.extend(hist.into_iter().flat_map(|(k, c)| [k, c]));
            row
        })
        .collect();
    let pairs: Vec<[usize; 3]> = union
        .iter()
        .map(|(&(a, b), &s)| [a, b, usize::from(s)])
        .collect();
    let mut names: Vec<(i64, &str)> = langs.0.into_iter().map(|(l, c)| (c, l)).collect();
    names.sort_unstable();
    (
        json!({"units": rows, "pairs": pairs, "exhaustive": exhaustive}),
        names.into_iter().map(|(_, l)| l).collect(),
    )
}

/// The pass: one request, the kept pairs back in (a, b) order, and the
/// union's, S2's, the bounds' and S5's counts written into the tally. A
/// degraded reply or a reply that does not add up is a named refusal —
/// the judgment never runs on a candidate set the core did not answer
/// whole.
pub(super) fn pass(
    link: &mut Link,
    units: &[Unit],
    union: &BTreeMap<(usize, usize), u8>,
    exhaustive: bool,
    tally: &mut Tally,
) -> Result<Vec<PairRow>> {
    let (request, lang_names) = body(units, union, exhaustive);
    let reply = judged::ask(link, CAP, SINCE, KIND, request).map_err(anyhow::Error::msg)?;
    if let Err(why) = judged::degraded(&reply) {
        bail!("candidates/1 degraded the T3 candidate pass ({why})");
    }
    let rows: Vec<[usize; 3]> = judged::table(&reply, "pairs").map_err(anyhow::Error::msg)?;
    let by_lang: Vec<[usize; 2]> =
        judged::table(&reply, "keyPairs").map_err(anyhow::Error::msg)?;
    let count = |k: &str| judged::count(&reply, k).map(|n| n as u64);
    let read = (|| -> Result<[u64; 11], String> {
        Ok([
            count("units")?,
            count("pairs")?,
            count("union")?,
            count("crossLanguage")?,
            count("prunedSize")?,
            count("prunedLabel")?,
            count("survivors")?,
            count("s5Windowed")?,
            count("s5PrunedLabel")?,
            count("s5Already")?,
            count("s5New")?,
        ])
    })()
    .map_err(anyhow::Error::msg)?;
    let [n_units, n_pairs, all, cross, size, label, survivors, windowed, cut, already, new] = read;
    ensure!(
        n_units == units.len() as u64
            && n_pairs == union.len() as u64
            && size + label + survivors == all
            && survivors + new == rows.len() as u64
            && by_lang.iter().all(|&[l, _]| l < lang_names.len())
            && rows.windows(2).all(|w| (w[0][0], w[0][1]) < (w[1][0], w[1][1]))
            && rows.iter().all(|r| r[0] < r[1] && r[1] < units.len()),
        "candidates/1 reply does not add up: wire skew"
    );
    for [l, n] in by_lang {
        *tally.raw_by.entry(format!("s2/{}", lang_names[l])).or_insert(0) += n as u64;
    }
    tally.cross_lang_dropped += cross;
    (
        tally.union_pairs,
        tally.pruned_size,
        tally.pruned_label,
        tally.survivors,
        tally.s5_windowed,
        tally.s5_pruned_label,
        tally.s5_already,
        tally.s5_new,
    ) = (all, size, label, survivors, windowed, cut, already, new);
    Ok(rows
        .into_iter()
        .map(|[a, b, s]| PairRow {
            a,
            b,
            sources: s as u8,
        })
        .collect())
}
