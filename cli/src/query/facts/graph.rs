//! The node universe and its tables: the graph wire's nodes (files,
//! packages, sections, walked assets — the same dense ids graph/1
//! judges, nodes.rs's one assignment) with the prose-only files the
//! docdup family reads appended after them; the roles, the references,
//! the unresolved counts, the languages, the line counts; and every
//! node's path for the directory tree the core builds over them, a
//! package seated at its own directory (plan v2.33 W1 item 3,
//! CE.Query.Tree).

use super::{Ctx, Sink};
use crate::dedup::index::Index;
use crate::graph::deadcode::GraphWire;
use crate::graph::wire::{GRAN_PACKAGE, GRAN_SECTION};
use crate::query::legend::{self, KIND_ASSET, KIND_PROSE, NODE_KINDS, REF_KINDS, ROLE_NAMES};
use crate::scan::lang::Lang;
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

/// The universe: label, kind (a `legend::NODE_KINDS` index) and path
/// per node, and the file-shaped nodes by path.
pub struct Nodes {
    pub labels: Vec<String>,
    pub kinds: Vec<usize>,
    pub paths: Vec<String>,
    by_path: BTreeMap<String, u64>,
}

impl Nodes {
    pub fn build(w: &GraphWire, idx: &Index) -> Result<Nodes> {
        let mut n = Nodes {
            labels: Vec::new(),
            kinds: Vec::new(),
            paths: Vec::new(),
            by_path: BTreeMap::new(),
        };
        for node in &w.nodes {
            let (kind, label) = if node.asset {
                (KIND_ASSET, node.path.clone())
            } else if node.kind == GRAN_SECTION {
                (
                    GRAN_SECTION as usize,
                    format!("{}#{}", node.path, node.unit),
                )
            } else if node.kind == GRAN_PACKAGE {
                (GRAN_PACKAGE as usize, format!("{}/", node.path))
            } else {
                (0, node.path.clone())
            };
            n.seat(kind, label, node.path.clone());
        }
        for path in crate::docdup::judge::candidates::prose_files(idx)? {
            n.seat(KIND_PROSE, path.clone(), path);
        }
        Ok(n)
    }

    fn seat(&mut self, kind: usize, label: String, path: String) {
        if Nodes::file_shaped(kind) {
            self.by_path.insert(path.clone(), self.labels.len() as u64);
        }
        self.labels.push(label);
        self.kinds.push(kind);
        self.paths.push(path);
    }

    /// A file, a walked asset or a prose file — a node with bytes.
    pub fn file_shaped(kind: usize) -> bool {
        kind == 0 || kind == KIND_ASSET || kind == KIND_PROSE
    }

    /// The node of a file-shaped path.
    pub fn of_path(&self, path: &str) -> Option<u64> {
        self.by_path.get(path).copied()
    }

    pub fn len(&self) -> usize {
        self.labels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

/// The node, role, reference, language and line tables.
pub fn fill(ctx: &Ctx<'_>, sink: &mut Sink) -> Result<()> {
    let nodes = ctx.nodes;
    for (i, kind) in nodes.kinds.iter().enumerate() {
        let id = i as u64;
        sink.row("node", vec![id, legend::sym(NODE_KINDS[*kind])]);
        if *kind == 0 {
            sink.row("file", vec![id]);
        }
        if Nodes::file_shaped(*kind) {
            let lang = Lang::from_path(Path::new(&nodes.paths[i])).map_or("unknown", Lang::name);
            sink.row("lang", vec![id, legend::sym(lang)]);
        }
        // a prose node has no wire row and no role
        let roles = ctx
            .wire
            .rows
            .get(i)
            .and_then(|r| r[2].as_i64())
            .unwrap_or(0);
        for (bit, name) in ROLE_NAMES.iter().enumerate() {
            if roles & (1 << bit) != 0 {
                sink.row("role", vec![id, legend::sym(name)]);
            }
        }
    }
    for [from, to, kind, rung] in &ctx.wire.edges {
        let name = REF_KINDS.get(*kind as usize).copied().unwrap_or("unknown");
        sink.row(
            "ref",
            vec![*from as u64, *to as u64, legend::sym(name), *rung as u64],
        );
    }
    if sink.wants("unresolved") {
        for (path, _total, unres) in crate::graph::load::graph_rows(ctx.idx)?.3 {
            if let (Some(node), true) = (nodes.of_path(&path), unres > 0) {
                sink.row("unresolved", vec![node, unres as u64]);
            }
        }
    }
    if sink.wants("lines") {
        line_counts(ctx, sink)?;
    }
    Ok(())
}

/// The directory tables the core builds, by name.
const DIR_TABLES: [&str; 4] = ["in_dir", "dir", "parent", "dir_name"];

/// The tree form, when the program reads a directory table: every
/// node's path in node order and the package nodes. The core builds the
/// tree over the other nodes' paths, seats each package at its own
/// directory and answers the four tables with each directory's label
/// and name (CE.Query.Tree), so the sink sends none of them.
pub fn tree(nodes: &Nodes, sink: &mut Sink) -> Option<Value> {
    if !sink.wants_any(&DIR_TABLES) {
        return None;
    }
    for name in DIR_TABLES {
        let code = sink.code(name);
        sink.tables.remove(&code);
    }
    let packages: Vec<usize> = (nodes.kinds.iter().enumerate())
        .filter(|(_, k)| **k == GRAN_PACKAGE as usize)
        .map(|(i, _)| i)
        .collect();
    Some(json!({"paths": nodes.paths, "packages": packages}))
}

#[cfg(test)]
#[path = "../../../tests/unit/query/facts/graph.rs"]
mod frozen;

/// One read per file-shaped node (assets too: a text asset has
/// lines, and a binary one counts what its bytes hold).
fn line_counts(ctx: &Ctx<'_>, sink: &mut Sink) -> Result<()> {
    for (i, kind) in ctx.nodes.kinds.iter().enumerate() {
        if !Nodes::file_shaped(*kind) {
            continue;
        }
        let full = ctx.root.join(&ctx.nodes.paths[i]);
        let Some(bytes) = crate::scan::walk::read_surviving(&full)? else {
            continue;
        };
        let n = crate::scan::metrics::size::total_lines(&bytes);
        sink.row("lines", vec![i as u64, n as u64]);
    }
    Ok(())
}
