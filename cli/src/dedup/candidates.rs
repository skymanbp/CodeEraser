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
/// s5 (bit 4) is the M5-close exhaustive source and PRODUCT-ONLY:
/// the frozen t3-candidates epoch re-derives collect() by digest, so
/// s5 lives in extend_exhaustive at the `ce clone` seam — frozen
/// docs can never carry bit 4.
pub const SOURCES: [&str; 5] = ["s1", "s2", "s3", "s4", "s5"];

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
    /// S5 ledger (extend_exhaustive): pairs the size window admitted,
    /// how many the label bound cut, how many were already four-source
    /// candidates, and how many are NEW to the judgment.
    pub s5_windowed: u64,
    pub s5_pruned_label: u64,
    pub s5_already: u64,
    pub s5_new: u64,
}

pub struct Candidates {
    pub units: Vec<Unit>,
    pub pairs: Vec<PairRow>,
    pub tally: Tally,
}

/// The whole candidate pass over one refreshed index — READ-ONLY on
/// the index (review HIGH-1: a writing candidate pass orphaned edges).
pub fn collect(root: &Path, idx: &Index) -> Result<Candidates> {
    let units = admitted(idx)?;
    let instances = idx.all_instances()?;
    let mut g = super::sources::Gen::new(&units);
    let union = g.union(root, &instances)?;
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

/// S5, the exhaustive in-domain source (M5 close, repaying the recall
/// instrument's finding: the four identity-signal sources cap recall
/// at their own yield — requests produced 128 candidate pairs against
/// a 425-pair comparator denominator, so even a perfect judgment
/// could not clear 0.30). Same-language admitted units, sorted by
/// node count; only pairs inside the §4.3 size bound are GENERATED
/// (`min·den >= num·max` — the identical predicate the prune
/// applies, evaluated at generation so the pair space stays near-
/// linear), and the label bound then cuts like everywhere else.
/// PRODUCT-ONLY by design: collect() stays the frozen four-source
/// pass whose digest the 3c/3f instruments re-derive on CI, and the
/// audited precision sample was drawn from that universe — S5 pairs
/// are candidates the frozen epoch never claimed.
pub fn extend_exhaustive(c: &mut Candidates) {
    let have: std::collections::BTreeSet<(usize, usize)> =
        c.pairs.iter().map(|p| (p.a, p.b)).collect();
    let mut by_lang: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (id, u) in c.units.iter().enumerate() {
        by_lang.entry(&u.lang).or_default().push(id);
    }
    let mut fresh = Vec::new();
    for ids in by_lang.values_mut() {
        ids.sort_by_key(|&id| c.units[id].nodes);
        for (i, &a) in ids.iter().enumerate() {
            for &b in &ids[i + 1..] {
                // ascending node order: a holds the min side, so the
                // window closes exactly where the size prune would cut
                if c.units[a].nodes * TSED_DEN < TSED_NUM * c.units[b].nodes {
                    break;
                }
                c.tally.s5_windowed += 1;
                let pair = (a.min(b), a.max(b));
                if have.contains(&pair) {
                    c.tally.s5_already += 1;
                } else if matches!(verdict(&c.units[a], &c.units[b]), Verdict::Label) {
                    c.tally.s5_pruned_label += 1;
                } else {
                    fresh.push(pair);
                }
            }
        }
    }
    fresh.sort_unstable();
    fresh.dedup();
    c.tally.s5_new = fresh.len() as u64;
    c.pairs.extend(fresh.into_iter().map(|(a, b)| PairRow {
        a,
        b,
        sources: 1 << 4,
    }));
    // the clone wire refuses non-ascending pair rows (the frozen
    // boundary contract) — appended S5 rows must fold back into the
    // canonical order
    c.pairs.sort_by_key(|p| (p.a, p.b));
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

    /// S5's window IS the size bound, its label cut IS the shared
    /// verdict, and an existing four-source pair is never duplicated
    /// — the new pair carries bit 4 alone (frozen docs can never
    /// hold it, so the source vocabulary stays epoch-safe).
    #[test]
    fn exhaustive_extension_windows_prunes_and_never_duplicates() {
        let full = || vec![(1u64, 100u32)];
        let mut c = Candidates {
            // ids 0..3: three ~100-node units sharing labels, one
            // 84-node unit outside the size window vs 100
            units: vec![
                unit(100, full()),
                unit(100, full()),
                unit(100, vec![(2, 100)]),
                unit(84, full()),
            ],
            pairs: vec![PairRow {
                a: 0,
                b: 1,
                sources: 0b0010,
            }],
            tally: Tally::default(),
        };
        extend_exhaustive(&mut c);
        let s5: Vec<(usize, usize, u8)> = c
            .pairs
            .iter()
            .filter(|p| p.sources == 1 << 4)
            .map(|p| (p.a, p.b, p.sources))
            .collect();
        // (0,1) stays four-source only; (0,2)/(1,2) are label-cut;
        // (3,*) sits outside the window on every side ≥100... except
        // 84·100 >= 85·100 is FALSE, so 84 pairs with nothing
        assert_eq!(s5, vec![], "no fresh pair survives here");
        assert_eq!(
            (c.tally.s5_already, c.tally.s5_pruned_label, c.tally.s5_new),
            (1, 2, 0)
        );
        // a genuinely similar unseen pair DOES land, bit 4 alone
        let mut c2 = Candidates {
            units: vec![unit(100, full()), unit(90, full())],
            pairs: vec![],
            tally: Tally::default(),
        };
        extend_exhaustive(&mut c2);
        assert_eq!(c2.tally.s5_new, 1);
        assert_eq!(
            (c2.pairs[0].a, c2.pairs[0].b, c2.pairs[0].sources),
            (0, 1, 1 << 4)
        );
    }
}
