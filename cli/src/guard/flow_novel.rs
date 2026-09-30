//! What a write brings (plan v2.31 step 5; design booklet §5.4): the
//! findings of the after text that the before text did not carry, as
//! a multiset difference over `(unit name, kind, variable name)` —
//! the guard's own novel semantics (the duplicate probe's, K step 11).
//! A line number never enters the key: a finding that moved because
//! lines were added above it is the same finding. A renamed unit or
//! variable is a new key, and so a new finding.

use crate::flow_report::Placed;
use std::collections::BTreeMap;

/// A finding's identity across the two sides of one write.
pub type Key = (String, u8, String);

pub fn key(p: &Placed) -> Key {
    (p.unit.clone(), p.kind, p.var.clone().unwrap_or_default())
}

/// The after findings the before side does not account for, in the
/// after side's order: each before finding cancels one after finding
/// with its key.
pub fn novel<'a>(before: &[Placed], after: &'a [Placed]) -> Vec<&'a Placed> {
    let mut left: BTreeMap<Key, usize> = BTreeMap::new();
    for p in before {
        *left.entry(key(p)).or_default() += 1;
    }
    after
        .iter()
        .filter(|p| match left.get_mut(&key(p)) {
            Some(n) if *n > 0 => {
                *n -= 1;
                false
            }
            _ => true,
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/guard/flow_novel.rs"]
mod tests;
