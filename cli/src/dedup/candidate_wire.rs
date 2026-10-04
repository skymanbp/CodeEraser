//! The candidates/1 leg of the T3 candidate pass (plan v2.33 W3): the
//! whole pass runs in the core over integers — every admitted unit as
//! its language, file, key, node count, line span and kind histogram,
//! each unit's structural shingle set, the fingerprint instances the
//! index holds and the near runs the T1/T2 extension verified — and the
//! core generates the sources S1 to S4, applies the two admissible
//! bounds (§4.3), adds the exhaustive source S5's new pairs when asked,
//! and answers the kept pairs and the tally. A pair's endpoints are
//! unit ids both ways; the languages come back by code and are named
//! here, the sources by bit.

use super::candidates::{PairRow, SOURCES, Tally, Unit};
use super::index::Instance;
use super::sources::NearRun;
use crate::corelink::{Link, judged};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// The capability the core must offer, and the request kind.
pub const CAP: &str = "candidates/1";
const KIND: &str = "candidates";

/// The proto that minted the family (named when a core lacks it).
const SINCE: &str = "8.2.0";

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

/// What the request carries besides the units.
pub(super) struct Facts<'f> {
    pub instances: &'f [Instance],
    pub near: &'f [NearRun],
    pub exhaustive: bool,
}

/// Every path the request names, ranked: code order is path order, so
/// the core's sort of a hot fingerprint group by (file, token offset)
/// is this side's sort by (path, token offset).
fn file_ranks<'p>(units: &'p [Unit], facts: &'p Facts<'_>) -> BTreeMap<&'p str, i64> {
    let paths: BTreeSet<&str> = units
        .iter()
        .map(|u| u.path.as_str())
        .chain(facts.instances.iter().map(|i| i.file.as_str()))
        .chain(
            facts
                .near
                .iter()
                .flat_map(|r| r.iter().map(|(f, _)| f.as_str())),
        )
        .collect();
    paths.into_iter().zip(0..).collect()
}

/// The request body and the language names by code: units as `[lang,
/// file, key, nodes, start, end, kind, count, …]` (language, key and
/// kind codes dense in first-seen order, file codes by path rank, each
/// histogram sorted by kind code), the units' sets, the instances as
/// `[hash, file, line, tok]` in index order, the near runs as `[fileA,
/// lineA, fileB, lineB]`.
fn body<'u>(units: &'u [Unit], facts: &Facts<'_>) -> (Value, Vec<&'u str>) {
    let files = file_ranks(units, facts);
    let (mut langs, mut keys) = (Codes::new(), Codes::new());
    let mut kinds: Codes<u64> = Codes::new();
    let rows: Vec<Vec<i64>> = units
        .iter()
        .map(|u| {
            let head = [
                langs.of(u.lang.as_str()),
                files[u.path.as_str()],
                keys.of(u.key.as_str()),
                u.nodes,
                u.start_line,
                u.end_line,
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
    let sigs: Vec<&[u64]> = units.iter().map(|u| u.sig.as_slice()).collect();
    let prints: Vec<[u64; 4]> = facts
        .instances
        .iter()
        .map(|i| {
            let file = files[i.file.as_str()] as u64;
            [i.hash, file, i.start_line as u64, i.start_tok as u64]
        })
        .collect();
    let near: Vec<[i64; 4]> = facts
        .near
        .iter()
        .map(|[(fa, la), (fb, lb)]| [files[fa.as_str()], *la, files[fb.as_str()], *lb])
        .collect();
    let mut names: Vec<(i64, &str)> = langs.0.into_iter().map(|(l, c)| (c, l)).collect();
    names.sort_unstable();
    (
        json!({"units": rows, "sigs": sigs, "prints": prints, "near": near,
               "exhaustive": facts.exhaustive}),
        names.into_iter().map(|(_, l)| l).collect(),
    )
}

/// The reply's counts, in the order `record` reads them.
const COUNTS: [&str; 16] = [
    "units",
    "prints",
    "near",
    "unowned",
    "selfPairs",
    "printHot",
    "bandHot",
    "union",
    "crossLanguage",
    "prunedSize",
    "prunedLabel",
    "survivors",
    "s5Windowed",
    "s5PrunedLabel",
    "s5Already",
    "s5New",
];

/// The pass: one request, the kept pairs back in (a, b) order, and
/// every source's, the bounds' and S5's counts written into the tally.
/// A degraded reply or a reply that does not add up is a named refusal
/// — the judgment never runs on a candidate set the core did not answer
/// whole.
pub(super) fn pass(
    link: &mut Link,
    units: &[Unit],
    facts: &Facts<'_>,
    tally: &mut Tally,
) -> Result<Vec<PairRow>> {
    let (request, lang_names) = body(units, facts);
    let fam = (CAP, SINCE, KIND);
    let (reply, rows, c): (_, Vec<[usize; 3]>, _) =
        judged::whole_pass(link, fam, request, ("T3 candidate pass", &COUNTS))?;
    let raw: Vec<[usize; 3]> = judged::table(&reply, "raw").map_err(anyhow::Error::msg)?;
    let groups: Vec<[u64; 2]> = judged::table(&reply, "bandGroups").map_err(anyhow::Error::msg)?;
    let (all, size, label, survivors, new) = (c[7], c[9], c[10], c[11], c[15]);
    ensure!(
        c[0] == units.len() as u64
            && c[1] == facts.instances.len() as u64
            && c[2] == facts.near.len() as u64
            && size + label + survivors == all
            && survivors + new == rows.len() as u64
            && raw
                .iter()
                .all(|&[s, l, _]| s < SOURCES.len() && l < lang_names.len())
            && rows
                .windows(2)
                .all(|w| (w[0][0], w[0][1]) < (w[1][0], w[1][1]))
            && rows.iter().all(|r| r[0] < r[1] && r[1] < units.len()),
        "candidates/1 reply does not add up: wire skew"
    );
    for [s, l, n] in raw {
        *tally
            .raw_by
            .entry(format!("{}/{}", SOURCES[s], lang_names[l]))
            .or_insert(0) += n as u64;
    }
    tally
        .s4_band_groups
        .extend(groups.into_iter().map(|[size, n]| (size, n)));
    record(tally, &c);
    Ok(rows
        .into_iter()
        .map(|[a, b, s]| PairRow {
            a,
            b,
            sources: s as u8,
        })
        .collect())
}

/// The reply's scalar counts (COUNTS order, from index 3) into the
/// tally, one field per count.
fn record(tally: &mut Tally, c: &[u64]) {
    let fields: [&mut u64; 13] = [
        &mut tally.unowned_dropped,
        &mut tally.self_pair_dropped,
        &mut tally.s3_hot_chained,
        &mut tally.s4_hot_chained,
        &mut tally.union_pairs,
        &mut tally.cross_lang_dropped,
        &mut tally.pruned_size,
        &mut tally.pruned_label,
        &mut tally.survivors,
        &mut tally.s5_windowed,
        &mut tally.s5_pruned_label,
        &mut tally.s5_already,
        &mut tally.s5_new,
    ];
    for (f, &v) in fields.into_iter().zip(&c[3..]) {
        *f = v;
    }
}
