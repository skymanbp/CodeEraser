//! Clone-group (k-way family) aggregation over verified pairwise
//! blocks (M2 attack-review R8): n mutual copies emit C(n,2) — or
//! chained n-1 — pairwise blocks, inflating the block denominator. A
//! group is a connected component over block endpoint spans, where
//! only IDENTICAL spans are the same occurrence. Overlapping but
//! non-identical spans stay distinct members ON PURPOSE: merging them
//! let one file bridge unrelated families into a single "k-way"
//! group, and let a same-file pair collapse to one member — both
//! confirmed by e2e repro (attack review 2026-08-07, R8 batch).
//! Pairwise blocks stay in the report (the R12 ratchet metric and
//! the audit/precommit diff filter consume them); groups are the
//! deduplicated view for humans and the M4 judgment layer.

use super::pairs::Block;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One occurrence of the cloned region: an exact endpoint line span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub file: String,
    pub start: usize,
    pub end: usize,
}

/// A clone family: endpoint spans connected by verified blocks. Line
/// spans are inclusive and coarser than token ranges, so a same-file
/// pair whose two disjoint runs split one physical line still shows
/// two members sharing that boundary line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub members: Vec<Member>,
    /// Pairwise blocks aggregated into this family.
    pub blocks: usize,
    /// Longest verified token run among those blocks.
    pub tokens: usize,
}

/// Aggregate pairwise blocks into families. Deterministic: groups are
/// ordered by their first member, members by (file, start, end).
pub fn group(blocks: &[Block]) -> Vec<Group> {
    let mut ids: BTreeMap<(&str, usize, usize), usize> = BTreeMap::new();
    let mut edges = Vec::with_capacity(blocks.len());
    for b in blocks {
        let a = intern(&mut ids, (b.a_file.as_str(), b.a_start, b.a_end));
        let c = intern(&mut ids, (b.b_file.as_str(), b.b_start, b.b_end));
        edges.push((a, c));
    }
    let mut uf: Vec<usize> = (0..ids.len()).collect();
    for &(a, c) in &edges {
        union(&mut uf, a, c);
    }
    collect(&mut uf, &ids, blocks, &edges)
}

fn intern<'b>(
    ids: &mut BTreeMap<(&'b str, usize, usize), usize>,
    key: (&'b str, usize, usize),
) -> usize {
    let next = ids.len();
    *ids.entry(key).or_insert(next)
}

fn find(uf: &mut [usize], x: usize) -> usize {
    let mut root = x;
    while uf[root] != root {
        root = uf[root];
    }
    let mut cur = x;
    while uf[cur] != root {
        let next = uf[cur];
        uf[cur] = root; // path compression
        cur = next;
    }
    root
}

fn union(uf: &mut [usize], a: usize, b: usize) {
    let (ra, rb) = (find(uf, a), find(uf, b));
    if ra != rb {
        uf[rb] = ra;
    }
}

/// Components → groups: every interned span is a member of exactly
/// one group; every block folds its stats into its component.
fn collect(
    uf: &mut [usize],
    ids: &BTreeMap<(&str, usize, usize), usize>,
    blocks: &[Block],
    edges: &[(usize, usize)],
) -> Vec<Group> {
    let mut order: Vec<usize> = Vec::new();
    let mut groups: BTreeMap<usize, Group> = BTreeMap::new();
    for (&(f, s, e), &id) in ids {
        let root = find(uf, id);
        let g = groups.entry(root).or_insert_with(|| {
            order.push(root);
            Group {
                members: Vec::new(),
                blocks: 0,
                tokens: 0,
            }
        });
        g.members.push(Member {
            file: f.to_string(),
            start: s,
            end: e,
        });
    }
    for (b, &(a, _)) in blocks.iter().zip(edges) {
        let g = groups.get_mut(&find(uf, a)).expect("endpoint interned");
        g.blocks += 1;
        g.tokens = g.tokens.max(b.tokens);
    }
    order
        .into_iter()
        .map(|r| groups.remove(&r).expect("root collected"))
        .collect()
}
