//! Structural fingerprints for the T3 cold path (design vol.2 §4):
//! the pre-order NAMED-node kind spine of a parse tree, per-unit kind
//! sequences, k-shingles over them, and the kind histogram feeding
//! the admissible label-intersection prune (§4.3). Pure functions —
//! no I/O; the SQLite cache lives in unitcache.rs.

use std::collections::{BTreeMap, BTreeSet};

/// Bump when the spine/shingle/histogram semantics change: it sits in
/// the meta cache key (schema v5), so stale unitsig rows are wiped.
pub const STRUCT_REV: i64 = 1;

/// Shingle width over the kind sequence. Decided, not derived
/// (registered before any evaluation data existed — instruments
/// publish the full cut table instead of tuning this silently).
pub const STRUCT_SHINGLE: usize = 4;

/// One named node of the pre-order walk: (start_line, end_line,
/// kind code), 1-based inclusive lines — the unit spans in
/// `fourclass::units` use the same convention, so containment is a
/// direct range check.
pub type Spine = Vec<(usize, usize, u64)>;

/// Kind text → stable u64 code. fnv1a of the grammar's kind string:
/// no registry to maintain, and cross-language collisions only blur a
/// prefilter that is conservative anyway (the TED judge never sees
/// codes, it sees request-local dense labels).
pub fn kind_code(kind: &str) -> u64 {
    super::tokens::fnv1a(kind.as_bytes())
}

/// Pre-order spine of every NAMED node under `root`. Anonymous nodes
/// (punctuation, keywords) are token trivia, not structure; pre-order
/// keeps the sequence deterministic and documentable.
pub fn spine(root: tree_sitter::Node) -> Spine {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.is_named() {
            out.push((
                node.start_position().row + 1,
                node.end_position().row + 1,
                kind_code(node.kind()),
            ));
        }
        // push children reversed so the stack pops them in order
        for i in (0..node.child_count()).rev() {
            if let Some(child) = node.child(i as u32) {
                stack.push(child);
            }
        }
    }
    out
}

/// The kind-code sequence of one unit: every spine entry whose line
/// range sits inside the unit's span, in spine (pre-order) order.
/// Nested definitions are INCLUDED on purpose — a unit's structure is
/// its whole subtree, which is what the TED judge will compare.
pub fn unit_seq(spine: &Spine, start_line: usize, end_line: usize) -> Vec<u64> {
    spine
        .iter()
        .filter(|(s, e, _)| start_line <= *s && *e <= end_line)
        .map(|(_, _, k)| *k)
        .collect()
}

/// Deduplicated k-shingle hashes of a kind sequence (k =
/// STRUCT_SHINGLE). A sequence shorter than k yields the empty set —
/// admission floors live upstream, not here.
pub fn shingles(seq: &[u64]) -> BTreeSet<u64> {
    seq.windows(STRUCT_SHINGLE)
        .map(|w| {
            let mut bytes = Vec::with_capacity(8 * STRUCT_SHINGLE);
            for k in w {
                bytes.extend_from_slice(&k.to_le_bytes());
            }
            super::tokens::fnv1a(&bytes)
        })
        .collect()
}

/// Kind histogram — the multiset the §4.3 label-intersection bound
/// consumes (I = Σ min(c1, c2)).
pub fn histogram(seq: &[u64]) -> BTreeMap<u64, u32> {
    let mut h = BTreeMap::new();
    for k in seq {
        *h.entry(*k).or_insert(0u32) += 1;
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shingles_are_order_sensitive_and_deduplicated() {
        let ab: Vec<u64> = vec![1, 2, 3, 4, 5];
        let ba: Vec<u64> = vec![5, 4, 3, 2, 1];
        assert_ne!(shingles(&ab), shingles(&ba), "order must matter");
        assert_eq!(shingles(&[1, 2, 3]).len(), 0, "below k yields empty");
        let rep: Vec<u64> = vec![7; 10];
        assert_eq!(shingles(&rep).len(), 1, "identical windows collapse");
    }

    #[test]
    fn histogram_counts_the_multiset() {
        let h = histogram(&[9, 9, 4]);
        assert_eq!(h.get(&9), Some(&2));
        assert_eq!(h.get(&4), Some(&1));
    }
}
