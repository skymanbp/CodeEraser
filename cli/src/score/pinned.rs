//! The trend road's pinned-soft baseline (plan v2.18 step #14, O34).
//! `ce trend` judges every historical commit with `establish` — an
//! absolute score, no ratchet noise — against ONE soft line the
//! caller pins. The baseline that carried that pin used to be two
//! EMPTY tables and no digest, so the core's ratchet compared the
//! request's own rows against nothing (every member "added", the
//! discrete condition held on any clone pair) and its fence compared
//! the request's declared digest against an absent one — a false
//! `knobs_digest` drift on every commit whose ce.toml was not the
//! shipped default. Trend read none of it because it ignored `fail`.
//! The pinned baseline is now the request's OWN facts: its
//! continuous rows (three columns — the class column never enters a
//! baseline), its discrete members, the pinned soft line, and the
//! digest it declares. Identity in, identity out: the only condition
//! a historical point can still trip is one the core names, and
//! trend now refuses the point by that name (trend::measure).

use serde_json::{Value, json};

/// The identity baseline for one pinned-soft measurement.
pub(crate) fn baseline(
    continuous: &[[u64; 4]],
    discrete: &[u64],
    soft: u64,
    digest: Option<u64>,
) -> Value {
    let rows: Vec<[u64; 3]> = continuous.iter().map(|[u, c, v, _]| [*u, *c, *v]).collect();
    let mut doc = json!({
        "continuous": rows,
        "discrete": discrete,
        "softLine": soft,
    });
    // ABSENT when none is declared, never null: the core's fence is
    // Maybe-equality, and two absents agree by rule where a `null`
    // against an absent request digest would agree by accident
    if let Some(d) = digest {
        doc["knobsDigest"] = json!(d);
    }
    doc
}

#[cfg(test)]
#[path = "../../tests/unit/score/pinned.rs"]
mod tests;
