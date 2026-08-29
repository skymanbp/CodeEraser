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
    /// A declared submodule's (index `files.owner`, or a package
    /// or section under such a file): a READER of this tree — its
    /// references count, and it is never a candidate of any verdict
    /// (plan v2.18 step #12).
    pub foreign: bool,
}

/// Dense node identities: every walked file plus every edge target —
/// a BTreeSet, so the id assignment is a function of the graph and
/// the wire bytes are shuffle-proof (G11). Kind reuses the wire
/// granularity codes; PACKAGE is taken from the edges' STORED
/// granularity, never inferred from absence — the old "not a walked
/// file ⇒ package" guess minted image assets and dangling doc refs
/// as package nodes (M5-close review LOW). `foreign` is the index's
/// per-file fact; a package or section node is foreign when it sits
/// under (or on) a foreign file's path, so a submodule's package rows
/// never surface as this tree's `reported` verdicts.
pub fn nodes_of(files: &[String], edges: &[GraphEdge], foreign: &BTreeSet<String>) -> Vec<Node> {
    let file_set: BTreeSet<&str> = files.iter().map(String::as_str).collect();
    let pkg_dsts: BTreeSet<&str> = edges
        .iter()
        .filter(|e| e.granularity == super::wire::GRAN_PACKAGE)
        .map(|e| e.dst_path.as_str())
        .collect();
    let foreign_dirs: BTreeSet<&str> = foreign
        .iter()
        .flat_map(|p| {
            p.match_indices('/')
                .map(move |(i, _)| &p[..i])
                .chain(std::iter::once(""))
        })
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
            // the root package "" holds own files too, so a foreign
            // file never makes the ROOT foreign — only real prefixes
            let foreign = foreign.contains(&path)
                || (kind == super::wire::GRAN_PACKAGE
                    && !path.is_empty()
                    && foreign_dirs.contains(path.as_str()));
            Node {
                path,
                unit,
                kind,
                foreign,
            }
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
#[path = "../../tests/unit/graph/nodes.rs"]
mod tests;
