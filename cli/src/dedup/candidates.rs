//! T3 candidate generation (design vol.2 §4.2): the four-source
//! union, judged by nothing — this pass never consults TSED or TED,
//! so the candidate universe freezes before the judge exists (RM2:
//! no judge picks its own denominator). Two provably admissible
//! prunes (§4.3) shrink the future TED workload with zero false
//! negatives; every drop lands in the tally, never in silence.
//! Lives beside the future t3/ on purpose: the T-G13 ancestry gate
//! requires the frozen sample to PRECEDE any file under dedup/t3 or
//! CE/Clone, and the sample is drawn from what this module produces.
//! The four source walks live in sources.rs (300-line gate).

use super::index::Index;
use super::{struct_fp, unitcache};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

/// Admission floor in named nodes (§4.1): below this a "clone" is a
/// signature, not an implementation — the t3-universe below_floor
/// ledger, structurally invisible here.
pub const T3_MIN_NODES: i64 = 24;

/// The clone threshold as an integer ratio (§4.4, 0.85). The prunes
/// use the SAME constants the judgment will — that identity is what
/// makes them admissible: a pair pruned here provably cannot reach
/// the threshold, whatever TED later computes.
pub const TSED_NUM: i64 = 85;
pub const TSED_DEN: i64 = 100;

/// S4 MinHash/LSH shape as ONE fact — (permutations, bands, rows),
/// 128 = 32 × 4 (the docdup coarse-filter split, §5.3; band_keys
/// asserts the product covers the signature).
pub const LSH_SHAPE: (usize, usize, usize) = (128, 32, 4);

/// Groups above this size pair as adjacent chains, counted — the ONE
/// cap (pairs::HOT_CAP, attack-review D4: skipping hot groups zeroed
/// detection) re-exposed for the S3/S4 walks and the frozen docs.
pub const HOT_GROUP_CAP: usize = super::pairs::HOT_CAP;

/// Candidate source labels in bit order: bit i of a pair's `sources`
/// byte means SOURCES[i] found it. One table — the generators' bit
/// assignment and the sample's label lookup can never drift apart.
pub const SOURCES: [&str; 4] = ["s1", "s2", "s3", "s4"];

/// One admitted unit; its position in Candidates::units is its id.
pub struct Unit {
    pub path: String,
    pub key: String,
    pub nth: i64,
    pub lang: String,
    pub nodes: i64,
    pub start_line: i64,
    pub end_line: i64,
    pub sig: Vec<u64>,
    pub hist: Vec<(u64, u32)>,
}

pub struct PairRow {
    pub a: usize,
    pub b: usize,
    pub sources: u8,
}

/// Every count the pipeline sheds anywhere — published, never silent
/// (the low_diversity_suppressed discipline).
#[derive(Default)]
pub struct Tally {
    pub raw_by: BTreeMap<String, u64>,
    pub union_pairs: u64,
    pub cross_lang_dropped: u64,
    pub unowned_dropped: u64,
    pub self_pair_dropped: u64,
    pub pruned_size: u64,
    pub pruned_label: u64,
    pub survivors: u64,
    pub s3_hot_chained: u64,
    pub s4_hot_chained: u64,
    pub s4_band_groups: BTreeMap<u64, u64>,
}

pub struct Candidates {
    pub units: Vec<Unit>,
    pub pairs: Vec<PairRow>,
    pub tally: Tally,
}

/// The whole candidate pass over one refreshed index.
pub fn collect(root: &Path, idx: &mut Index) -> Result<Candidates> {
    let units = admitted(idx)?;
    let instances = idx.all_instances()?;
    let mut g = super::sources::Gen::new(&units);
    let union = g.union(root, &instances, idx)?;
    let mut tally = g.tally;
    let pairs = prune(&units, union, &mut tally);
    Ok(Candidates {
        units,
        pairs,
        tally,
    })
}

/// unitsig facts joined to their symbols spans, floored at
/// T3_MIN_NODES. The span join is total by the identity-orphans
/// invariant `ce clone --units` asserts (3b).
fn admitted(idx: &Index) -> Result<Vec<Unit>> {
    let spans: BTreeMap<(String, String, i64), (i64, i64)> =
        crate::graph::symbols::symbol_rows(idx)?
            .into_iter()
            .map(|s| ((s.path, s.key, s.nth), (s.start_line, s.end_line)))
            .collect();
    Ok(unitcache::fact_rows(idx)?
        .into_iter()
        .filter(|f| f.nodes >= T3_MIN_NODES)
        .map(|f| {
            let key = (f.path.clone(), f.key.clone(), f.nth);
            let (start_line, end_line) = spans[&key];
            let lang = f.path.rsplit('.').next().unwrap_or("").to_string();
            Unit {
                path: f.path,
                key: f.key,
                nth: f.nth,
                lang,
                nodes: f.nodes,
                start_line,
                end_line,
                sig: f.sig,
                hist: f.hist,
            }
        })
        .collect())
}

/// The two admissible prunes, in cost order (§4.3): the O(1)
/// size-band corollary `min·tsedDen < tsedNum·max`, then the
/// O(#kinds) label-intersection bound `I·tsedDen < tsedNum·max`
/// (from `ted >= max(n1,n2) − I`). Both decide "provably below
/// threshold", never "probably" — an approximate prune here would be
/// a false negative polluting the frozen denominator (3c red
/// condition). The bounds themselves are asserted against brute-force
/// TED as exhaustive-family properties in 3e (R2).
enum Verdict {
    Size,
    Label,
    Keep,
}

/// The admissible verdict for one pair. Each bound quantity `q`
/// tests `q · tsedDen < tsedNum · max`: from `ted >= max − q` it
/// follows `TSED <= q/max`, so failing the cross-product means even
/// the best case cannot reach the threshold — "provably below",
/// never "probably". Array order = size bound first (its tally owns
/// pairs both bounds would cut).
fn verdict(a: &Unit, b: &Unit) -> Verdict {
    let mx = a.nodes.max(b.nodes);
    let bounds = [
        (a.nodes.min(b.nodes), Verdict::Size),
        (
            struct_fp::label_intersection(&a.hist, &b.hist),
            Verdict::Label,
        ),
    ];
    for (q, v) in bounds {
        if q * TSED_DEN < TSED_NUM * mx {
            return v;
        }
    }
    Verdict::Keep
}

fn prune(units: &[Unit], union: BTreeMap<(usize, usize), u8>, tally: &mut Tally) -> Vec<PairRow> {
    let mut out = Vec::new();
    for ((a, b), sources) in union {
        match verdict(&units[a], &units[b]) {
            Verdict::Size => tally.pruned_size += 1,
            Verdict::Label => tally.pruned_label += 1,
            Verdict::Keep => out.push(PairRow { a, b, sources }),
        }
    }
    tally.survivors = out.len() as u64;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(nodes: i64, hist: Vec<(u64, u32)>) -> Unit {
        Unit {
            path: "a.rs".into(),
            key: "k".into(),
            nth: 0,
            lang: "rs".into(),
            nodes,
            start_line: 1,
            end_line: 1,
            sig: vec![1],
            hist,
        }
    }

    /// The 0.85 boundary is admissible in BOTH directions: equality
    /// survives (best case ted = max−min gives TSED exactly
    /// min/max), provably-below is cut, and each bound owns its
    /// tally. Row 3/4: sizes equal but the label multisets share
    /// I = 84 (cut) vs 85 (kept) at max 100.
    #[test]
    fn prunes_sit_exactly_on_the_admissible_boundary() {
        let full = || vec![(1u64, 100u32)];
        let sharing = |c: u32| vec![(1, c), (2, 100 - c)];
        let table = [
            (85, full(), full(), (1, 0, 0)),
            (84, full(), full(), (0, 1, 0)),
            (100, sharing(84), full(), (0, 0, 1)),
            (100, sharing(85), full(), (1, 0, 0)),
        ];
        for (n1, h1, h2, want) in table {
            let units = vec![unit(n1, h1), unit(100, h2)];
            let union = BTreeMap::from([((0usize, 1usize), 2u8)]);
            let mut tally = Tally::default();
            let kept = prune(&units, union, &mut tally).len();
            assert_eq!(
                (kept, tally.pruned_size, tally.pruned_label),
                want,
                "n1 = {n1}"
            );
        }
    }
}
