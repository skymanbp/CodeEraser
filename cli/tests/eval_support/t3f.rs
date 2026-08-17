//! T3 precision-instrument shared surface (M5-3f): sampled-row →
//! frozen-pair lookup, the unit span/verbatim throat, and the live
//! judgment leg — ONE binding for the audit assembly
//! (eval_t3_audit.rs) and the precision scorer, so the assembler and
//! the scorer can never disagree about what a sampled pair is or how
//! it was judged. Tree building, wire codec and the clone verdict
//! are the PRODUCT throats (dedup::t3); the only instrument-side
//! code is the loop.

use codeeraser::corelink::Link;
use codeeraser::dedup::candidates::Candidates;
use codeeraser::dedup::t3::{self, tree, wire};
use codeeraser::fourclass::units;
use codeeraser::scan::lang::Lang;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Closed audit truth vocabulary (design §9.2). `boilerplate` and
/// `t1t2` are deliberately single classes: folding either into
/// `unrelated` hides exactly what the calibration run found.
pub const T3_TRUTHS: [&str; 6] = [
    "clone",
    "variant",
    "boilerplate",
    "unrelated",
    "t1t2",
    "generated",
];

/// The clone-row edit axis (an ablation axis, not decoration).
pub const T3_EDITS: [&str; 5] = ["rename", "reorder", "insert", "type_sub", "control_tweak"];

/// Pre-registered denominator top-up floor: the audit walks the
/// frozen rank order into the backups until judged-clone rows reach
/// this many (the EVAL-SET top-up mechanism, T3 edition).
pub const MIN_ANSWERED: u64 = 40;

/// Identity fields every derived row must echo verbatim (G4).
pub const T3_IDENTITY: [&str; 10] = [
    "corpus", "lang", "band", "source", "a_path", "a_key", "a_nth", "b_path", "b_key", "b_nth",
];

/// The t3 family's mount into the ONE review registry (the C3
/// by-name discipline, T-G10).
pub fn t3_review_doc(corpus: &str) -> Value {
    super::review_of("t3", corpus)
}

/// The frozen sample's MAIN rows of one corpus, in doc (audit-domain)
/// order — the shared of_corpus filter under the t3 sample's key.
pub fn corpus_mains<'a>(sample: &'a Value, corpus: &str) -> Vec<&'a Value> {
    super::of_corpus(sample["main"].as_array().expect("main"), corpus)
}

/// corpus_mains with each row's pinned tip asserted against the
/// caller's — the generator entry.
pub fn main_rows<'a>(sample: &'a Value, corpus: &str, tip: &str) -> Vec<&'a Value> {
    let rows = corpus_mains(sample, corpus);
    for r in &rows {
        assert_eq!(
            r["tip"].as_str(),
            Some(tip),
            "{corpus}: sampled row pinned to a different tip"
        );
    }
    rows
}

/// (key, nth) → 1-based inclusive line span of one file's units,
/// resolved through the SAME segmentation throat the unitsig cache
/// used — a second span derivation could slice text the judge never
/// saw.
pub fn file_spans(text: &str, lang: Lang) -> BTreeMap<(String, i64), (usize, usize)> {
    let segs = units::segments(text, lang);
    units::with_nth(&segs)
        .into_iter()
        .map(|(u, nth)| ((u.key.clone(), nth), (u.start_line, u.end_line)))
        .collect()
}

/// Verbatim lines start..=end (1-based inclusive), joined with \n.
pub fn slice_lines(text: &str, start: usize, end: usize) -> String {
    text.lines()
        .skip(start - 1)
        .take(end - start + 1)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Walked texts keyed by path — the judge and the verbatim assembler
/// read the SAME strings the detectors saw.
pub fn texts_by_path(walked: &[super::WalkedFile]) -> BTreeMap<&str, (&str, &str)> {
    walked
        .iter()
        .map(|(p, code, text)| (p.as_str(), (*code, text.as_str())))
        .collect()
}

/// One sampled row's judgment by the shipped judge.
#[derive(Clone, Copy)]
pub enum Judgment {
    Scored { ted: i64, n1: i64, n2: i64 },
    Prefiltered,
    OverCap,
    Forest,
}

impl Judgment {
    /// The verdict the precision instrument scores, through the ONE
    /// product threshold binding (dedup::t3::is_clone).
    pub fn is_clone(self) -> bool {
        matches!(self, Judgment::Scored { ted, n1, n2 } if t3::is_clone(ted, n1, n2))
    }

    pub fn label(self) -> &'static str {
        match self {
            Judgment::Scored { .. } => "scored",
            Judgment::Prefiltered => "prefiltered",
            Judgment::OverCap => "over_cap",
            Judgment::Forest => "forest",
        }
    }
}

/// Sampled row → (a, b) unit indices in the candidate pass; the pair
/// must be a SURVIVOR (the sample was drawn from the survivors and
/// the pool digest anchors it — a miss here is drift, not data).
pub fn pair_units(c: &Candidates, row: &Value) -> (usize, usize) {
    let by_id: BTreeMap<(&str, &str, i64), usize> = c
        .units
        .iter()
        .enumerate()
        .map(|(i, u)| ((u.path.as_str(), u.key.as_str(), u.nth), i))
        .collect();
    let side = |p: &str, k: &str, n: &str| {
        let key = (
            row[p].as_str().expect(p),
            row[k].as_str().expect(k),
            row[n].as_i64().expect(n),
        );
        *by_id
            .get(&key)
            .unwrap_or_else(|| panic!("sampled unit {key:?} not in the candidate pass"))
    };
    let (a, b) = (
        side("a_path", "a_key", "a_nth"),
        side("b_path", "b_key", "b_nth"),
    );
    assert!(
        c.pairs.iter().any(|p| (p.a, p.b) == (a, b)),
        "sampled pair is not a survivor of the live candidate pass"
    );
    (a, b)
}

/// The build outcome for a unit set: wire trees where the walk found
/// a single capped root, the ledger reason everywhere else.
pub struct BuiltUnits {
    pub trees: BTreeMap<usize, tree::UnitTree>,
    pub dropped: BTreeMap<usize, Judgment>,
}

/// One parse per file over the walked texts; node counts asserted
/// against the frozen candidate pass (the same-source counterfactual
/// the product driver carries, instrument edition).
pub fn build_unit_trees(
    c: &Candidates,
    texts: &BTreeMap<&str, (&str, &str)>,
    ids: &BTreeSet<usize>,
) -> BuiltUnits {
    let mut by_file: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    let mut out = BuiltUnits {
        trees: BTreeMap::new(),
        dropped: BTreeMap::new(),
    };
    for &i in ids {
        if c.units[i].nodes > wire::UNIT_NODE_CAP {
            out.dropped.insert(i, Judgment::OverCap);
        } else {
            by_file.entry(&c.units[i].path).or_default().push(i);
        }
    }
    for (path, unit_ids) in by_file {
        let (code, text) = texts[path];
        let spans: Vec<(usize, usize)> = unit_ids
            .iter()
            .map(|&i| (c.units[i].start_line as usize, c.units[i].end_line as usize))
            .collect();
        let built = tree::file_trees(text, super::lang_of(code), &spans);
        assert_eq!(built.len(), spans.len(), "{path}: parse failed");
        for (&i, b) in unit_ids.iter().zip(built) {
            match b {
                tree::Built::Forest(_) => {
                    out.dropped.insert(i, Judgment::Forest);
                }
                tree::Built::Tree(t) => {
                    assert_eq!(
                        t.lab.len() as i64,
                        c.units[i].nodes,
                        "{path}: tree walk disagrees with the frozen node count"
                    );
                    out.trees.insert(i, t);
                }
            }
        }
    }
    out
}

/// Judge sampled rows with the shipped judge: product tree build,
/// product wire codec, product verdict; one clone.request (a sample
/// is far below pairCap). Returns one Judgment per row, in order.
pub fn judge_sample(
    c: &Candidates,
    texts: &BTreeMap<&str, (&str, &str)>,
    rows: &[&Value],
    link: &mut Link,
) -> Vec<Judgment> {
    let pairs: Vec<(usize, usize)> = rows.iter().map(|r| pair_units(c, r)).collect();
    let ids: BTreeSet<usize> = pairs.iter().flat_map(|&(a, b)| [a, b]).collect();
    let built = build_unit_trees(c, texts, &ids);
    let mut sendable: Vec<(usize, usize)> = pairs
        .iter()
        .copied()
        .filter(|&(a, b)| built.trees.contains_key(&a) && built.trees.contains_key(&b))
        .collect();
    sendable.sort_unstable();
    sendable.dedup();
    let scored = score_pairs(&built.trees, &sendable, link);
    pairs
        .iter()
        .map(|&(a, b)| {
            [a, b]
                .iter()
                .find_map(|i| built.dropped.get(i).copied())
                .or_else(|| scored.get(&(a, b)).copied())
                .unwrap_or(Judgment::Prefiltered)
        })
        .collect()
}

/// One clone.request over the live link, laid out by the product's
/// own chunk throat (wire::chunk_request).
fn score_pairs(
    trees: &BTreeMap<usize, tree::UnitTree>,
    sendable: &[(usize, usize)],
    link: &mut Link,
) -> BTreeMap<(usize, usize), Judgment> {
    if sendable.is_empty() {
        return BTreeMap::new();
    }
    assert!(sendable.len() <= wire::PAIR_CAP, "sample exceeds pairCap");
    let (order, body) = wire::chunk_request(sendable, |g| &trees[&g]);
    let reply = link.request("clone", body).expect("clone.request");
    let (rows, _counts) = wire::parse_result(&reply).expect("clone.result");
    // the instrument's verdict stays its local is_clone binding
    // (frozen semantics); the wire's ADR-008 P1 bit is dropped here,
    // and the product run()'s per-row ensure is what proves the two
    // can never fork silently
    rows.into_iter()
        .map(|(i, j, (ted, n1, n2, _))| ((order[i], order[j]), Judgment::Scored { ted, n1, n2 }))
        .collect()
}
