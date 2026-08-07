//! Clone-group (k-way family) aggregation over verified pairwise
//! blocks (M2 attack-review R8): n mutual copies emit C(n,2) — or
//! chained n-1 — pairwise blocks, inflating the block denominator. A
//! group is a connected component over block endpoints, where
//! overlapping spans in the same file count as one occurrence.
//! Pairwise blocks stay in the report (the R12 ratchet metric and the
//! audit/precommit diff filter consume them); groups are the
//! deduplicated view for humans and the M4 judgment layer.

use super::pairs::Block;
use serde::Serialize;
use std::collections::BTreeMap;

/// One occurrence of the cloned region: a merged line span in a file.
#[derive(Debug, Clone, Serialize)]
pub struct Member {
    pub file: String,
    pub start: usize,
    pub end: usize,
}

/// A k-way clone family: every member holds a copy of the region.
#[derive(Debug, Clone, Serialize)]
pub struct Group {
    pub members: Vec<Member>,
    /// Pairwise blocks aggregated into this family.
    pub blocks: usize,
    /// Longest verified token run among those blocks.
    pub tokens: usize,
}

/// Aggregate pairwise blocks into families. Deterministic: groups are
/// ordered by their first member, members by (file, start line).
pub fn group(blocks: &[Block]) -> Vec<Group> {
    let mut ids: BTreeMap<(&str, usize, usize), usize> = BTreeMap::new();
    let mut edges = Vec::with_capacity(blocks.len());
    for b in blocks {
        let a = intern(&mut ids, (b.a_file.as_str(), b.a_start, b.a_end));
        let c = intern(&mut ids, (b.b_file.as_str(), b.b_start, b.b_end));
        edges.push((a, c));
    }
    let mut uf: Vec<usize> = (0..ids.len()).collect();
    let spans: Vec<(&str, usize, usize, usize)> =
        ids.iter().map(|(&(f, s, e), &id)| (f, s, e, id)).collect();
    link_overlaps(&mut uf, &spans);
    for &(a, c) in &edges {
        union(&mut uf, a, c);
    }
    collect(&mut uf, &spans, blocks, &edges)
}

fn intern<'b>(
    ids: &mut BTreeMap<(&'b str, usize, usize), usize>,
    key: (&'b str, usize, usize),
) -> usize {
    let next = ids.len();
    *ids.entry(key).or_insert(next)
}

/// Same-file spans that overlap are the same occurrence. dominant()
/// only drops CONTAINED runs, so distinct maximal runs still overlap
/// (R8's 97 mutually-overlapping pairs) — span equality alone would
/// double-count them. Spans arrive sorted by (file, start, end).
fn link_overlaps(uf: &mut [usize], spans: &[(&str, usize, usize, usize)]) {
    let mut prev: Option<(usize, &str, usize)> = None;
    for &(f, s, e, id) in spans {
        prev = match prev {
            Some((pid, pf, pe)) if pf == f && s <= pe => {
                union(uf, pid, id);
                Some((pid, pf, pe.max(e)))
            }
            _ => Some((id, f, e)),
        };
    }
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

/// Components → groups: merge overlapping member spans per file, then
/// fold every block's stats into its component. Within one group the
/// spans keep their global (file, start, end) order — a subsequence
/// of a sorted sequence — so a last-member check suffices to merge.
fn collect(
    uf: &mut [usize],
    spans: &[(&str, usize, usize, usize)],
    blocks: &[Block],
    edges: &[(usize, usize)],
) -> Vec<Group> {
    let mut order: Vec<usize> = Vec::new();
    let mut groups: BTreeMap<usize, Group> = BTreeMap::new();
    for &(f, s, e, id) in spans {
        let root = find(uf, id);
        let g = groups.entry(root).or_insert_with(|| {
            order.push(root);
            Group {
                members: Vec::new(),
                blocks: 0,
                tokens: 0,
            }
        });
        match g.members.last_mut() {
            Some(m) if m.file == f && s <= m.end => m.end = m.end.max(e),
            _ => g.members.push(Member {
                file: f.to_string(),
                start: s,
                end: e,
            }),
        }
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
