//! The frozen-universe FAMILY binding — one skeleton for every
//! per-corpus instrument family (t3-universe, docdup-segments,
//! docdup-oracle): the sibling-anchored generate with its double-run
//! proof, the gate opening (envelope + sibling anchor) and the two
//! closing idioms (coverage floor, self drift walk). Extracted when
//! the repo's own ratchet caught the THIRD family re-instantiating
//! the second's skeleton token for token — the exact recurrence the
//! twelfth bite's universe.rs extraction was meant to end; the
//! family-level shape (not just the walk and envelope) is the fact.

use serde_json::Value;
use std::collections::BTreeMap;

use super::universe::{
    UniverseParts, WalkedFile, assert_doc_envelope, doc_stem, universe_doc, universe_parts,
    walk_tree,
};

/// One universe family: its doc identity plus the three per-family
/// functions every envelope operation needs. Const-constructible so
/// each instrument declares exactly one.
pub struct UniverseFamily {
    pub family: &'static str,
    pub schema: &'static str,
    pub method: &'static str,
    pub row: fn(&str, &str, &str) -> Value,
    pub constants: fn() -> Value,
    pub summarize: fn(&[Value]) -> Value,
}

impl UniverseFamily {
    /// Rows + envelope from one walk.
    pub fn build(
        &self,
        name: &Option<String>,
        tip: &str,
        walked: &[WalkedFile],
        excluded: &BTreeMap<&'static str, u64>,
    ) -> Value {
        let parts: UniverseParts = universe_parts(
            walked,
            excluded.clone(),
            self.row,
            (self.constants)(),
            self.summarize,
        );
        universe_doc(self.schema, self.method, name, tip, parts)
    }

    /// The family generate: sibling tip pin + walk + per-family proof
    /// + double-run byte identity.
    pub fn generate(
        &self,
        sibling: &str,
        prove: impl Fn(&Value, &[WalkedFile]),
    ) -> (Option<String>, Value, Vec<WalkedFile>) {
        universe_gen(sibling, prove, |name, tip, walked, excluded| {
            self.build(name, tip, walked, excluded)
        })
    }

    /// Generate and write in one motion — the whole generator body of
    /// a family with no extra generation-time proofs.
    pub fn freeze(&self, sibling: &str, prove: impl Fn(&Value, &[WalkedFile])) {
        let (name, doc, _) = self.generate(sibling, prove);
        super::write_universe(self.family, &name, &doc);
    }

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

/// Generate one frozen per-corpus doc against a sibling family: pin
/// the sibling's tip, walk the tree, run the generation-time proof,
/// build TWICE from two independent walks and require byte identity
/// (the doc must be a function of the tree, nothing else).
pub fn universe_gen(
    sibling: &str,
    prove: impl Fn(&Value, &[WalkedFile]),
    build: impl Fn(&Option<String>, &str, &[WalkedFile], &BTreeMap<&'static str, u64>) -> Value,
) -> (Option<String>, Value, Vec<WalkedFile>) {
    let (name, tip) = super::graph_corpus();
    let anchor = super::load(&super::eval_doc(&doc_stem(sibling, &name)));
    assert_eq!(
        anchor["corpus"]["tip"].as_str().expect("tip"),
        tip,
        "the doc must freeze its {sibling} sibling's pinned tip"
    );
    let (walked, excluded) = walk_tree(&tip);
    prove(&anchor, &walked);
    let doc = build(&name, &tip, &walked, &excluded);
    let again = {
        let (w2, e2) = walk_tree(&tip);
        build(&name, &tip, &w2, &e2)
    };
    assert_eq!(
        serde_json::to_string(&doc).expect("ser"),
        serde_json::to_string(&again).expect("ser"),
        "double run diverged — the doc is not a function of the tree"
    );
    (name, doc, walked)
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
