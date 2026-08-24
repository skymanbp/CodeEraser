//! Shared audit-instrument machinery, second-family extraction (the
//! repo's own ratchet caught eval_docdup_audit re-growing
//! eval_t3_audit token for token — bite seventeen): ONE review-table
//! registry for every audit family, ONE domain-separated identity
//! hash and ONE frozen-sample envelope writer. The AuditFamily frame
//! walk and its coverage/tamper drivers retired with the one-shot
//! audit instruments (git history, 0c7c936 wave).

use serde_json::Value;

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
