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
