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
/// granularity codes; PACKAGE is taken from the edges' STORED
/// granularity, never inferred from absence — the old "not a walked
/// file ⇒ package" guess minted image assets and dangling doc refs
/// as package nodes (M5-close review LOW).
pub fn nodes_of(files: &[String], edges: &[GraphEdge]) -> Vec<Node> {
    let file_set: BTreeSet<&str> = files.iter().map(String::as_str).collect();
    let pkg_dsts: BTreeSet<&str> = edges
        .iter()
        .filter(|e| e.granularity == super::wire::GRAN_PACKAGE)
        .map(|e| e.dst_path.as_str())
        .collect();
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
            } else if pkg_dsts.contains(path.as_str()) {
                super::wire::GRAN_PACKAGE
            } else {
                super::wire::GRAN_FILE
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
        // A package at the REPO ROOT has path "", and `format!("{}/",
        // "")` is "/" — which no repo-relative member starts with, so
        // the root package contained nothing and its files read as
        // unreferenced (measured: a root lib.go imported by cmd/main.go
        // was reported dead).
        let prefix = match pkg.path.as_str() {
            "" => String::new(),
            p => format!("{p}/"),
        };
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

    fn edge(src: &str, dst: &str, unit: &str, granularity: i64) -> GraphEdge {
        GraphEdge {
            src: src.into(),
            dst_path: dst.into(),
            dst_unit: unit.into(),
            kind: 0,
            rung: 1,
            granularity,
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
        let g = crate::graph::wire::GRAN_PACKAGE;
        let files = vec!["a.rs".to_string(), "lib/b.rs".to_string()];
        let edges = vec![
            edge("a.rs", "lib", "", g),
            edge("a.rs", "doc.md", "intro", 2),
        ];
        let mut rev_files = files.clone();
        rev_files.reverse();
        let rev_edges = vec![
            edge("a.rs", "doc.md", "intro", 2),
            edge("a.rs", "lib", "", g),
        ];
        assert_eq!(triples(&files, &edges), triples(&rev_files, &rev_edges));
        // containment arcs (named AND root package) live in
        // tests/graph_containment.rs — behaviour, not identity
    }

    /// An absent-file target is a FILE node unless an edge's stored
    /// granularity SAYS package: the asset / dangling-ref shapes the
    /// old absence-inference minted as packages (review LOW).
    #[test]
    fn absent_targets_are_files_unless_an_edge_says_package() {
        let files = vec!["doc.md".to_string()];
        let edges = vec![
            edge("doc.md", "art/logo.png", "", 0),
            edge("doc.md", "gone.md", "", 0),
            edge("doc.md", "pkg", "", crate::graph::wire::GRAN_PACKAGE),
        ];
        let by: std::collections::BTreeMap<String, i64> = nodes_of(&files, &edges)
            .into_iter()
            .map(|n| (n.path, n.kind))
            .collect();
        assert_eq!(by["art/logo.png"], crate::graph::wire::GRAN_FILE);
        assert_eq!(by["gone.md"], crate::graph::wire::GRAN_FILE);
        assert_eq!(by["pkg"], crate::graph::wire::GRAN_PACKAGE);
    }
}
