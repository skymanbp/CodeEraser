//! Declaration-level relocation: the MEASUREMENT half (plan v2.29
//! step 10, O48). A unit whose body cannot clear the line-evidence
//! floor still has an identity — its name, its kind and its arity —
//! and when that identity leaves one file and arrives in exactly one
//! other inside a single changeset, the relocation is visible at the
//! declaration level although no station can open.
//!
//! Measured here: which declaration keys vanished from a pair's
//! before side, which appeared on its after side, and the 1-based
//! inclusive span each occupies. DECIDED elsewhere (CE.FourClass.Decl):
//! whether two of them are the same declaration. Names never cross —
//! only a fnv1a hash, a kind word and two line numbers (ADR-002 A6).

use super::units::Unit;
use crate::dedup::tokens::fnv1a;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// One declaration candidate. `key` stays on THIS side of the wire;
/// `kind` is the word from `fourclass::kinds`; the span is 1-based
/// inclusive, the same convention `dupSpans` already ships.
#[derive(Clone, Debug)]
pub struct Decl {
    pub key: String,
    pub kind: i64,
    pub start: usize,
    pub end: usize,
}

/// The wire identity of a declaration: fnv1a of its unit key. The
/// kind rides beside it rather than inside the hash so the judgment
/// can state the pairing rule instead of hiding it in a digest.
pub fn key_hash(d: &Decl) -> u64 {
    fnv1a(d.key.as_bytes())
}

/// The (key, kind) multiplicity of one side of a pair.
fn tally(units: &[Unit]) -> BTreeMap<(&str, i64), usize> {
    let mut out: BTreeMap<(&str, i64), usize> = BTreeMap::new();
    for u in units {
        *out.entry((u.key.as_str(), u.kind)).or_default() += 1;
    }
    out
}

/// Declarations that occur EXACTLY once on `side` and not at all on
/// `other`. Multiplicity above one is not an identity — two same-key
/// methods in different impl blocks share a key by construction
/// (attack review F7) — so such a key is no candidate at all, and
/// the core refuses a duplicated row by name if this ever drifts.
/// Sorted on (key, kind): the request is a set, so its order is
/// ours to fix, and a fixed order keeps the bytes deterministic.
fn one_sided(side: &[Unit], other: &[Unit]) -> Vec<Decl> {
    let (mine, theirs) = (tally(side), tally(other));
    let mut out: Vec<Decl> = side
        .iter()
        .filter(|u| {
            let k = (u.key.as_str(), u.kind);
            mine[&k] == 1 && !theirs.contains_key(&k)
        })
        .map(|u| Decl {
            key: u.key.clone(),
            kind: u.kind,
            start: u.start_line,
            end: u.end_line,
        })
        .collect();
    out.sort_by(|a, b| (&a.key, a.kind).cmp(&(&b.key, b.kind)));
    out
}

/// One pair's two tables: keys that vanished from `before`, keys that
/// appeared in `after`. Both empty is a MEASUREMENT ("nothing moved
/// in or out of this file"), which the wire keeps distinct from an
/// absent table ("this aligner does not measure declarations").
pub fn tables(before: &[Unit], after: &[Unit]) -> (Vec<Decl>, Vec<Decl>) {
    (one_sided(before, after), one_sided(after, before))
}

fn rows(ds: &[Decl]) -> Value {
    ds.iter()
        .map(|d| json!([key_hash(d), d.kind, d.start, d.end]))
        .collect()
}

/// The pair's two request values. Always sent once measured: the
/// CORE holds the cap (CE.FourClass.Decl.declCap) and names the
/// refusal in its reply, so there is no second authority here to
/// drift away from it.
pub fn request_keys(t: &(Vec<Decl>, Vec<Decl>)) -> (Value, Value) {
    (rows(&t.0), rows(&t.1))
}

#[cfg(test)]
#[path = "../../tests/unit/fourclass/decls.rs"]
mod tests;
