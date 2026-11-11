//! The anchor walk of the T1/T2 extension (a child of pairs.rs, v2.35
//! imports lane): instances grouped by fingerprint hash, every candidate
//! pair visited — chained above the hot cap — and the files any shared
//! hash touches. pairs.rs verifies the pairs this walk yields; the T3
//! candidate source reads the same walk's near runs through it.

use crate::dedup::index::Instance;
use std::collections::{BTreeMap, BTreeSet};

/// Above this group size, pairing switches from full pairwise
/// (C(n,2)) to an adjacent chain (n-1 pairs) sorted by (file, tok).
/// Attack-review D4: skipping hot groups made detection FALL TO ZERO
/// as duplication rose (65 identical files → 0 blocks); chaining
/// keeps every instance in ≥1 verified pair at linear cost. The cap is
/// the core's (CE.Dedup.Cost.hotCap, off the package since plan v2.33
/// W3): the T3 sources and the docdup coarse filter chain by the same
/// one there.
pub(crate) fn hot_cap() -> usize {
    crate::tables::get().limits.dedup.hot_cap
}

/// One visit of the hash-group pairing walk.
pub(crate) enum GroupEvent<'a> {
    /// A group crossed the hot cap and was chained instead of paired.
    Chained,
    Pair(&'a Instance, &'a Instance),
}

/// Group instances by fingerprint hash and visit every candidate
/// pair (chained above the hot cap) — ONE grouping walk for the T1/T2
/// extension pass and the S3 candidate source, so the two can never
/// disagree about which anchors exist.
pub(crate) fn each_hash_pair<'a>(instances: &'a [Instance], mut f: impl FnMut(GroupEvent<'a>)) {
    let mut by_hash: BTreeMap<u64, Vec<&Instance>> = BTreeMap::new();
    for inst in instances {
        by_hash.entry(inst.hash).or_default().push(inst);
    }
    for group in by_hash.values_mut().filter(|g| g.len() > 1) {
        if group.len() > hot_cap() {
            f(GroupEvent::Chained);
            group.sort_by(|x, y| (&x.file, x.start_tok).cmp(&(&y.file, y.start_tok)));
            for w in group.windows(2) {
                f(GroupEvent::Pair(w[0], w[1]));
            }
        } else {
            for (i, a) in group.iter().enumerate() {
                for b in &group[i + 1..] {
                    f(GroupEvent::Pair(a, b));
                }
            }
        }
    }
}

/// Files that participate in any shared hash — the caller provides
/// token streams for exactly these.
pub fn candidate_files(instances: &[Instance]) -> BTreeSet<String> {
    let mut by_hash: BTreeMap<u64, Vec<&Instance>> = BTreeMap::new();
    for inst in instances {
        by_hash.entry(inst.hash).or_default().push(inst);
    }
    by_hash
        .values()
        .filter(|g| g.len() > 1)
        .flat_map(|g| g.iter().map(|i| i.file.clone()))
        .collect()
}
