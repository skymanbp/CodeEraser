//! T3 candidate generation (design vol.2 §4.2): the four-source
//! union, judged by nothing — this pass never consults TSED or TED,
//! so the candidate universe freezes before the judge exists (RM2:
//! no judge picks its own denominator). Two provably admissible
//! prunes (§4.3) shrink the future TED workload with zero false
//! negatives; every drop lands in the tally, never in silence. Since
//! plan v2.33 W3 the prunes and the exhaustive source S5 are the
//! core's (candidates/1, candidate_wire.rs): this side generates.
//! Lives beside t3/ on purpose: the T-G13 ancestry gate requires the
//! frozen sample to PRECEDE, in history, every file under dedup/t3 and
//! CE/Clone, and the sample is drawn from what this module produces.
//! The four source walks live in sources.rs (300-line gate).

use super::index::Index;
use super::unitcache;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

/// The clone judgment's thresholds and ceilings, read off the core's
/// package (plan v2.33 W3, CE.Limits): the admission floor below which
/// a "clone" is a signature, not an implementation (§4.1), the 85/100
/// ratio the bounds and the judgment share (§4.4), the per-tree node
/// ceiling and the per-request pair ceiling — one owner, CE.Clone.Cost.
pub fn limits() -> &'static crate::tables::CloneLimits {
    &crate::tables::get().limits.clone
}

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
/// No in-tree reader since the sample generators retired (0c7c936):
/// the table is the revival base the frozen docs' labels compile
/// against (EVAL-SET.md「再生成」). s5 (bit 4) is the M5-close
/// exhaustive source and PRODUCT-ONLY — frozen docs never carry bit 4.
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

/// Every count the pipeline sheds anywhere — published, never silent;
/// the write-only ones are the frozen docs' summary rows (revival base).
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
    /// S5 ledger (the core's, candidate_wire.rs): pairs the size window admitted,
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
/// The four sources generate here; the two admissible bounds (§4.3)
/// and, with `exhaustive`, the S5 source run in the core over `link`
/// (candidates/1, plan v2.33 W3). The frozen-instrument path asks
/// without S5: its universe is the four-source epoch's.
pub fn collect(
    root: &Path,
    idx: &Index,
    link: &mut crate::corelink::Link,
    exhaustive: bool,
) -> Result<Candidates> {
    let units = admitted(idx)?;
    let instances = idx.all_instances()?;
    let mut g = super::sources::Gen::new(&units);
    let union = g.union(root, &instances)?;
    let mut tally = g.tally;
    let pairs = super::candidate_wire::pass(link, &units, &union, exhaustive, &mut tally)?;
    Ok(Candidates {
        units,
        pairs,
        tally,
    })
}

/// unitsig facts joined to their symbols spans, floored at the
/// package's `min_unit_nodes`. The span join is total by the
/// identity-orphans invariant `ce clone --units` asserts (3b).
fn admitted(idx: &Index) -> Result<Vec<Unit>> {
    let spans: BTreeMap<(String, String, i64), (i64, i64)> =
        crate::graph::symbols::symbol_rows(idx)?
            .into_iter()
            .map(|s| ((s.path, s.key, s.nth), (s.start_line, s.end_line)))
            .collect();
    Ok(unitcache::fact_rows(idx)?
        .into_iter()
        .filter(|f| f.nodes >= limits().min_unit_nodes)
        .map(|f| {
            let key = (f.path.clone(), f.key.clone(), f.nth);
            let (start_line, end_line) = spans[&key];
            // the GRAMMAR, not the extension string (batch-7 slice
            // 15): raw extensions split one language into buckets —
            // a byte-identical copy from a.ts into b.mts scored
            // TED 0 and was dropped by the cross-language gate
            // before the judge ever saw it. Units only exist for
            // indexed (judged) files, so from_path resolves; the
            // raw-extension fallback keeps totality honest.
            let lang = crate::scan::lang::Lang::from_path(std::path::Path::new(&f.path))
                .map(|l| l.name().to_string())
                .unwrap_or_else(|| f.path.rsplit('.').next().unwrap_or("").to_string());
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

