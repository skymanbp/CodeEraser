//! The graph request's node rows and arcs (split from `wire_of` at the
//! E01 line, plan v2.35 5C): the entry globs compiled once, then each
//! node's row over one id space, the cached edges, the containment arcs
//! and a build's forced includes.

use super::super::load::GraphEdge;
use super::super::nodes::{self, Node};
use super::{edge_wire, node_row, targets};
use crate::config::Config;
use anyhow::Result;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// [graph] entry_globs compile ONCE, through the one ce.toml glob
/// dialect (scan::globs; the hand-rolled three-form matcher retired in
/// plan v2.18 step #14) — a pattern the dialect cannot read is a named
/// error here, never a silent no-match.
pub(super) fn entry_set(root: &Path, config: &Config) -> Result<crate::scan::globs::Inclusions> {
    crate::scan::globs::compile_inclusions(root, &config.graph.entry_globs, "[graph] entry_globs")
        .map_err(anyhow::Error::msg)
}

/// The node rows and the arcs over one id space (split from wire_of at
/// the E01 line, plan v2.35 5C): each node's [lang, kind, roles] row,
/// then the cached edges, the containment arcs and a build's forced
/// includes.
pub(super) fn rows_and_arcs(
    root: &Path,
    (config, entries): (&Config, &crate::scan::globs::Inclusions),
    (files, edges): (&[String], &[GraphEdge]),
    (nodes, ids): (&[Node], &BTreeMap<(&str, &str), usize>),
) -> Result<(Vec<Value>, BTreeSet<[i64; 4]>)> {
    let file_set: BTreeSet<String> = files.iter().cloned().collect();
    let declared = targets::Declared::gather(root, &file_set, &config.graph.declared_roots())
        .map_err(anyhow::Error::msg)?;
    let rows: Vec<Value> = nodes
        .iter()
        .map(|n| node_row(root, n, entries, &declared))
        .collect();
    let mut wire = edge_wire(edges, ids)?;
    nodes::contain(nodes, ids, declared.package_code_by_root(), &mut wire);
    // a build's forced includes (`-include x.h`) are arcs the source
    // text never spells: unit → header, beside the containment arcs
    // (plan v2.30 step 5b, item 14; the core's resolve/1 since v2.33 W2a)
    super::super::resolve::forced_wire(root, &file_set, ids, &mut wire)
        .map_err(anyhow::Error::msg)?;
    Ok((rows, wire))
}
