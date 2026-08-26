//! The frozen-universe FAMILY binding — one skeleton for every
//! per-corpus instrument family: the gate opening (envelope +
//! sibling anchor) and the two closing idioms (coverage floor,
//! self drift walk). Extracted when the repo's own ratchet caught
//! the THIRD family re-instantiating the second's skeleton token
//! for token; the generate/freeze half retired with the one-shot
//! generators (git history).

use serde_json::Value;
use std::collections::BTreeMap;

use super::universe::{assert_doc_envelope, doc_stem};

/// One universe family: its doc-family name plus the two per-family
/// functions the gate opening needs (the doc identity fields left
/// with the generators — the frozen docs carry their own). Const-
/// constructible so each instrument declares exactly one.
pub struct UniverseFamily {
    pub family: &'static str,
    pub constants: fn() -> Value,
    pub summarize: fn(&[Value]) -> Value,
}

impl UniverseFamily {
    /// The shared gate opening: iterate the family's frozen docs,
    /// assert the nine-key envelope and the graph-slice sibling
    /// anchor, then hand each doc to the family's own checks.
    pub fn each_consistent(&self, mut per: impl FnMut(&str, &Value)) {
        super::each_frozen_doc(self.family, |path, doc| {
            let name = assert_doc_envelope(path, doc, self.family, self.summarize, self.constants);
            assert_sibling_anchor(path, doc, &name);
            per(path, doc);
        });
    }
}

/// The CI-persistent anchor every universe family shares: same pinned
/// tip, same path→sha256 inventory and same exclusion tally as the
/// graph-slice sibling — one tip, one tree, N frozen views.
pub fn assert_sibling_anchor(path: &str, doc: &Value, name: &Option<String>) {
    let slice = super::load(&super::eval_doc(&doc_stem("graph-slice", name)));
    assert_eq!(
        doc["corpus"]["tip"], slice["corpus"]["tip"],
        "{path}: tip differs from the slice sibling"
    );
    assert_eq!(
        super::str_pairs(doc, "files", "path", "sha256"),
        super::str_pairs(&slice, "files", "path", "sha256"),
        "{path}: inventory differs from the slice sibling"
    );
    assert_eq!(
        doc["excluded"], slice["excluded"],
        "{path}: exclusion tally differs from the slice sibling"
    );
}

/// Every listed key must be positive in the accumulated cross-corpus
/// map — the anti-vacuous closing every universe gate ends with.
pub fn assert_covered<'k>(
    map: &BTreeMap<String, u64>,
    keys: impl IntoIterator<Item = &'k str>,
    what: &str,
) {
    for k in keys {
        assert!(
            map.get(k).copied().unwrap_or(0) > 0,
            "no {k} {what} across the frozen universes"
        );
    }
}

/// The self working-tree drift gate: every sha-matched frozen row
/// must re-derive byte-identically through the family's row throat,
/// and enough rows must still match for the gate to mean anything.
pub fn assert_self_tracks(family: &str, row: fn(&str, &str, &str) -> Value, floor: usize) {
    let doc = super::load(&super::eval_doc(family));
    let verified = super::each_frozen_match(&doc, |frozen, path, lang, text| {
        assert_eq!(
            frozen,
            &row(path, lang, text),
            "{path}: row throat drifted from the frozen universe"
        );
    });
    assert!(
        verified >= floor,
        "drift gate near-vacuous: {verified} rows"
    );
}
