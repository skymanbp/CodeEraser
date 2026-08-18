//! Shared audit-instrument machinery, second-family extraction (the
//! repo's own ratchet caught eval_docdup_audit re-growing
//! eval_t3_audit token for token — bite seventeen): ONE review-table
//! registry for every audit family, ONE domain-separated identity
//! hash, ONE assembly writer and ONE audit-frame walk parameterized
//! by an AuditFamily spec. Family files keep only their vocabularies,
//! their bespoke rows (edit axis, retired evidence) and their tamper
//! cases.

use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Every frozen review table of every audit family, mounted by name
/// (include_str! makes a missing corpus a compile error — a whole
/// corpus can never go silently blind, G10). Three families grew
/// three copies of this table before it became one registry.
pub const REVIEWS: [(&str, &str, &str); 15] = [
    (
        "graph",
        "cobra",
        include_str!("../eval_graph_review/cobra.json"),
    ),
    (
        "graph",
        "requests",
        include_str!("../eval_graph_review/requests.json"),
    ),
    (
        "graph",
        "ripgrep",
        include_str!("../eval_graph_review/ripgrep.json"),
    ),
    (
        "graph",
        "self",
        include_str!("../eval_graph_review/self.json"),
    ),
    (
        "graph",
        "zod",
        include_str!("../eval_graph_review/zod.json"),
    ),
    ("t3", "cobra", include_str!("../eval_t3_review/cobra.json")),
    (
        "t3",
        "requests",
        include_str!("../eval_t3_review/requests.json"),
    ),
    (
        "t3",
        "ripgrep",
        include_str!("../eval_t3_review/ripgrep.json"),
    ),
    ("t3", "self", include_str!("../eval_t3_review/self.json")),
    ("t3", "zod", include_str!("../eval_t3_review/zod.json")),
    (
        "docdup",
        "cobra",
        include_str!("../eval_docdup_review/cobra.json"),
    ),
    (
        "docdup",
        "requests",
        include_str!("../eval_docdup_review/requests.json"),
    ),
    (
        "docdup",
        "ripgrep",
        include_str!("../eval_docdup_review/ripgrep.json"),
    ),
    (
        "docdup",
        "self",
        include_str!("../eval_docdup_review/self.json"),
    ),
    (
        "docdup",
        "zod",
        include_str!("../eval_docdup_review/zod.json"),
    ),
];

/// One family's mounted ground truth for one corpus — a wrong name is
/// a loud panic, per family, as DATA.
pub fn review_of(family: &str, corpus: &str) -> Value {
    let text = REVIEWS
        .iter()
        .find(|(f, c, _)| *f == family && *c == corpus)
        .map(|(_, _, t)| *t)
        .unwrap_or_else(|| panic!("no {family} review table for {corpus}"));
    serde_json::from_str(text).unwrap_or_else(|e| panic!("{family}/{corpus}: {e}"))
}

/// Domain-separated identity hash: sha256("domain|f1|f2|…") over the
/// row's named fields in order, strings verbatim and integers in
/// decimal — the ONE derivation every sample generator and every
/// verify gate repeats (two families each grew their own).
pub fn identity_hash(domain: &str, row: &Value, fields: &[&str]) -> String {
    let parts: Vec<String> = fields
        .iter()
        .map(|k| match &row[*k] {
            Value::String(s) => s.clone(),
            v => v
                .as_i64()
                .unwrap_or_else(|| panic!("{k}: not a string or integer"))
                .to_string(),
        })
        .collect();
    super::content_sha(&format!("{domain}|{}", parts.join("|")))
}

/// One family's audit shape: where its census/sample lives, the
/// identity fields every derived row must echo, the closed truth
/// vocabulary and the gaps key its tables must account for.
pub struct AuditFamily {
    pub identity: &'static [&'static str],
    pub truths: &'static [&'static str],
    pub gaps_key: &'static str,
}

impl AuditFamily {
    /// One corpus's frozen table against the sampled rows: embedded
    /// name, bijection, verbatim identity echo, tip echo, closed
    /// verdicts, gaps accounted — the family closure sees each row
    /// AFTER the frame holds, for its bespoke axes only.
    pub fn check_frame(
        &self,
        corpus: &str,
        doc: &Value,
        sample_rows: &[Value],
        mut extra: impl FnMut(&str, &Value, &Value),
    ) {
        super::check_audit_frame(corpus, doc, sample_rows, |rank, row, s| {
            for key in self.identity {
                assert_eq!(row[*key], s[*key], "{corpus}/{rank}: {key} echo drifted");
            }
            assert_eq!(doc["tip"], s["tip"], "{corpus}: tip vs sampled tip");
            self.check_verdict(corpus, rank, row);
            extra(rank, row, s);
        });
        super::assert_gaps_accounted(corpus, doc, self.gaps_key);
    }

    /// truth in the closed vocabulary; why holds the mechanism floor.
    pub fn check_verdict(&self, corpus: &str, rank: &str, row: &Value) {
        let truth = row["truth"].as_str().expect("truth");
        assert!(
            self.truths.contains(&truth),
            "{corpus}/{rank}: unknown truth {truth:?}"
        );
        let why = row["why"].as_str().expect("why").trim();
        assert!(
            why.len() >= 15,
            "{corpus}/{rank}: why is not a mechanism: {why:?}"
        );
    }
}

/// Tally rows-by-truth across corpora and table sections; returns
/// (row total, truth counts) for the coverage and seat gates.
pub fn tally_truths(
    corpora: &[&str],
    doc_of: impl Fn(&str) -> Value,
    sections: &[&str],
) -> (usize, BTreeMap<String, u64>) {
    let (mut total, mut by_truth) = (0, BTreeMap::new());
    for corpus in corpora {
        let doc = doc_of(corpus);
        for key in sections {
            for row in doc[*key].as_array().expect(key) {
                *by_truth
                    .entry(row["truth"].as_str().expect("truth").to_string())
                    .or_insert(0) += 1;
            }
        }
        total += doc[sections[0]].as_array().expect("rows").len();
    }
    (total, by_truth)
}

/// The coverage half every audit exit gate is: per-corpus frame
/// checks (inside doc_of), primary-section total == the sample's
/// main size, and every promised vocabulary seat nonzero.
pub fn assert_coverage_and_seats(
    sample: &Value,
    corpora: &[&str],
    doc_of: impl Fn(&str) -> Value,
    sections: &[&str],
    seats: &[&str],
) -> BTreeMap<String, u64> {
    let (total, by_truth) = tally_truths(corpora, doc_of, sections);
    assert_eq!(
        total,
        sample["main"].as_array().expect("main").len(),
        "audited rows must cover the sample exactly"
    );
    super::assert_nonzero_seats(&by_truth, seats, "promised vocabulary seat is empty");
    by_truth
}

/// The shared tamper prologue: mount the pristine table, run the
/// table-driven mutation battery, hand the pristine back for the
/// family's bespoke cases.
pub fn tamper_prologue(
    family: &str,
    corpus: &str,
    mutations: &[(&str, &str, &str)],
    check: &dyn Fn(&Value),
) -> Value {
    let pristine = review_of(family, corpus);
    super::assert_tampering_refused(&pristine, mutations, check);
    pristine
}

/// The frozen-sample doc envelope every sample generator writes:
/// schema, both hash domains, provenance, method, then the family's
/// own body fields merged on top.
pub fn sample_doc(schema: &str, domains: (&str, &str), method: &str, body: Value) -> Value {
    let mut doc = json!({
        "schema": schema,
        "domains": {"rank": domains.0, "audit": domains.1},
        "generated_from": super::generated_from(),
        "method": method,
    });
    for (k, v) in body.as_object().expect("body").clone() {
        doc[k] = v;
    }
    doc
}
