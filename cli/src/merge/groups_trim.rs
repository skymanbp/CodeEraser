//! A T1/T2 fragment family trimmed to its members' common shape (plan
//! v2.31 step 7, step-7 ruling 3): a member is a line span, and the
//! token run behind it starts and ends mid-line, so a boundary line can
//! hold half a statement its twin's does not. Each member's top nodes
//! (its maximal selected subtrees, comments left out) get a shape key;
//! the longest run of member 0's keys that every member holds as a
//! contiguous run is kept — the earliest in member 0 among equals, each
//! other member's first occurrence — and each member sends only that run
//! under its synthetic root, priced by the run's own lines. No run in
//! common, and the family is not one shape.

use crate::dedup::t3::tree::{Top, UnitTree, fragment_of};
use crate::dedup::tokens::fnv1a;

/// The mark a leaf takes in a shape key: its label is open, as in the
/// core's isomorphism (a leaf may differ, only a leaf).
const LEAF: u64 = u64::MAX;

/// One top tree's shape key: the fnv1a of its postorder (internal
/// label or the leaf mark, lld relative to the tree's first node) — two
/// tops share a key exactly when merge/1 reads them as one shape.
pub fn shape_key(t: &UnitTree) -> u64 {
    let base = t.lld.first().copied().unwrap_or(0);
    let mut bytes = Vec::with_capacity(t.lab.len() * 16);
    for (i, (&lab, &lld)) in t.lab.iter().zip(&t.lld).enumerate() {
        let mark = if lld == i as i64 { LEAF } else { lab };
        bytes.extend(mark.to_le_bytes());
        bytes.extend((lld - base).to_le_bytes());
    }
    fnv1a(&bytes)
}

/// Each member's fragment tree over the kept run and the run's lines,
/// or None when the members hold no run of tops in common.
pub fn trim(members: &[&[Top]]) -> Option<Vec<(UnitTree, (usize, usize))>> {
    let keys: Vec<Vec<u64>> = members
        .iter()
        .map(|tops| tops.iter().map(|t| shape_key(&t.tree)).collect())
        .collect();
    let (first, rest) = keys.split_first()?;
    let (start, len) = window(first, rest)?;
    let run = &first[start..start + len];
    members
        .iter()
        .zip(&keys)
        .map(|(tops, k)| {
            let at = k.windows(len).position(|w| w == run)?;
            fragment_of(&tops[at..at + len])
        })
        .collect()
}

/// The longest run of `first` every other sequence holds contiguously,
/// as (start, length) in `first` — the earliest start among equals;
/// None when no run is common.
pub fn window(first: &[u64], rest: &[Vec<u64>]) -> Option<(usize, usize)> {
    let mut common: Vec<usize> = (1..=first.len()).collect();
    for other in rest {
        let reach = suffixes(first, other);
        for (c, r) in common.iter_mut().zip(reach) {
            *c = (*c).min(r);
        }
    }
    let mut best: Option<(usize, usize)> = None;
    for (end, &len) in common.iter().enumerate() {
        if len > 0 && best.is_none_or(|(_, b)| len > b) {
            best = Some((end + 1 - len, len));
        }
    }
    best
}

/// For each position a of `a`, the longest common suffix of `a[..=a]`
/// with any prefix of `b` ending anywhere — every shorter run ending at
/// a is then in `b` too.
fn suffixes(a: &[u64], b: &[u64]) -> Vec<usize> {
    let mut prev = vec![0usize; b.len() + 1];
    let mut out = Vec::with_capacity(a.len());
    for &x in a {
        let mut row = vec![0usize; b.len() + 1];
        for (j, &y) in b.iter().enumerate() {
            if x == y {
                row[j + 1] = prev[j] + 1;
            }
        }
        out.push(row.iter().copied().max().unwrap_or(0));
        prev = row;
    }
    out
}

/// The core's T1/T2 shape (CE.Merge.Align.isomorphic, step-6 ruling
/// 2), asked before sending so its refusal never lands on the live road: one
/// lld column, and every internal node's label and hash member 0's.
pub fn isomorphic(trees: &[&UnitTree]) -> bool {
    let Some((first, rest)) = trees.split_first() else {
        return true;
    };
    rest.iter().all(|t| {
        t.lld == first.lld
            && (0..t.lab.len()).all(|i| {
                t.lld[i] == i as i64 || (t.lab[i] == first.lab[i] && t.leaf[i] == first.leaf[i])
            })
    })
}
