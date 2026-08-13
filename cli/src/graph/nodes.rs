//! The graph's node-identity throat (F19, split from deadcode.rs):
//! dense node identities and the synthetic containment arcs. deadcode
//! consumes it today; the M5-3 join consumes the SAME functions so
//! both verdicts stand on one id assignment — the exit criterion is
//! an assertion of byte-equal assignments, and a second copy of this
//! logic is exactly what it forbids.

use super::load::GraphEdge;
use std::collections::{BTreeMap, BTreeSet};

pub struct Node {
    pub path: String,
    pub unit: String,
    pub kind: i64,
}

/// Dense node identities: every walked file plus every edge target —
/// a BTreeSet, so the id assignment is a function of the graph and
/// the wire bytes are shuffle-proof (G11). Kind reuses the wire
/// granularity codes.
pub fn nodes_of(files: &[String], edges: &[GraphEdge]) -> Vec<Node> {
    let file_set: BTreeSet<&str> = files.iter().map(String::as_str).collect();
    let mut set: BTreeSet<(String, String)> =
        files.iter().map(|p| (p.clone(), String::new())).collect();
    for e in edges {
        set.insert((e.dst_path.clone(), e.dst_unit.clone()));
    }
    set.into_iter()
        .map(|(path, unit)| {
            let kind = if !unit.is_empty() {
                super::wire::GRAN_SECTION
            } else if file_set.contains(path.as_str()) {
                super::wire::GRAN_FILE
            } else {
                super::wire::GRAN_PACKAGE
            };
            Node { path, unit, kind }
        })
        .collect()
}

/// Position of every node in the dense assignment, keyed by identity.
pub fn ids(nodes: &[Node]) -> BTreeMap<(&str, &str), usize> {
    nodes
        .iter()
        .enumerate()
        .map(|(i, n)| ((n.path.as_str(), n.unit.as_str()), i))
        .collect()
}

/// Synthetic containment arcs: package node → every file under its
/// directory. rung 1: containment is a fact, not a resolution
/// mechanism, and it must survive every rung ceiling.
pub fn contain(nodes: &[Node], ids: &BTreeMap<(&str, &str), usize>, wire: &mut BTreeSet<[i64; 4]>) {
    for pkg in nodes.iter().filter(|n| n.kind == super::wire::GRAN_PACKAGE) {
        let prefix = format!("{}/", pkg.path);
        let p = ids[&(pkg.path.as_str(), "")] as i64;
        for member in nodes
            .iter()
            .filter(|n| n.kind == super::wire::GRAN_FILE && n.path.starts_with(&prefix))
        {
            let m = ids[&(member.path.as_str(), "")] as i64;
            wire.insert([p, m, super::wire::EDGE_CONTAIN, 1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(src: &str, dst: &str, unit: &str) -> GraphEdge {
        GraphEdge {
            src: src.into(),
            dst_path: dst.into(),
            dst_unit: unit.into(),
            kind: 0,
            rung: 1,
        }
    }

    /// Identity triples of the dense assignment for `files`/`edges`.
    fn triples(files: &[String], edges: &[GraphEdge]) -> Vec<(String, String, i64)> {
        nodes_of(files, edges)
            .into_iter()
            .map(|n| (n.path, n.unit, n.kind))
            .collect()
    }

    /// The id assignment is a function of the graph, not of input
    /// order — reversing the inputs yields the identical node list.
    #[test]
    fn assignment_is_shuffle_proof() {
        let files = vec!["a.rs".to_string(), "lib/b.rs".to_string()];
        let edges = vec![edge("a.rs", "lib", ""), edge("a.rs", "doc.md", "intro")];
        let mut rev_files = files.clone();
        rev_files.reverse();
        let rev_edges = vec![edge("a.rs", "doc.md", "intro"), edge("a.rs", "lib", "")];
        assert_eq!(triples(&files, &edges), triples(&rev_files, &rev_edges));
        // package "lib" contains lib/b.rs via a synthetic arc
        let nodes = nodes_of(&files, &edges);
        let ids = ids(&nodes);
        let mut wire = BTreeSet::new();
        contain(&nodes, &ids, &mut wire);
        let p = ids[&("lib", "")] as i64;
        let m = ids[&("lib/b.rs", "")] as i64;
        assert!(wire.contains(&[p, m, crate::graph::wire::EDGE_CONTAIN, 1]));
    }
}
