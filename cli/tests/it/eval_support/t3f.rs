//! T3 precision-instrument shared surface (M5-3f): the frozen
//! identity fields, the review-registry mount and the sampled-row
//! corpus filter. The live judgment leg (Judgment, score_pairs)
//! retired with the one-shot audit instruments (git history).

use serde_json::Value;

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
