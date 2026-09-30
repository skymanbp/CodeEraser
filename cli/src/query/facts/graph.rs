//! The node universe and its tables: the graph wire's nodes (files,
//! packages, sections, walked assets — the same dense ids graph/1
//! judges, nodes.rs's one assignment) with the prose-only files the
//! docdup family reads appended after them; the directory tree the
//! structure family builds, over every node path, with a package
//! seated at its own directory; the roles, the references, the
//! unresolved counts, the languages, the line counts.

use super::{Ctx, Labels, Sink};
use crate::dedup::index::Index;
use crate::graph::deadcode::GraphWire;
use crate::graph::wire::{GRAN_PACKAGE, GRAN_SECTION};
use crate::query::legend::{self, KIND_ASSET, KIND_PROSE, NODE_KINDS, REF_KINDS, ROLE_NAMES};
use crate::scan::lang::Lang;
use crate::structure::tree;
use anyhow::Result;
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

/// The node, role, reference, language, line and tree tables.
pub fn fill(ctx: &Ctx<'_>, sink: &mut Sink, labels: &mut Labels) -> Result<()> {
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
    if sink.wants_any(&["in_dir", "dir", "parent", "dir_name"]) {
        tree_tables(nodes, sink, labels);
    }
    Ok(())
}

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

/// The directory tree over every file-shaped and section path, a
/// package seated at its own directory; root is dir 0 and reads `.`.
fn tree_tables(nodes: &Nodes, sink: &mut Sink, labels: &mut Labels) {
    let file_paths: Vec<String> = nodes
        .kinds
        .iter()
        .zip(&nodes.paths)
        .filter(|(k, _)| **k != GRAN_PACKAGE as usize)
        .map(|(_, p)| p.clone())
        .collect();
    let mut t = tree::build(&file_paths);
    for (i, kind) in nodes.kinds.iter().enumerate() {
        let dir = if *kind == GRAN_PACKAGE as usize {
            tree::dir_id(&mut t, &nodes.paths[i])
        } else {
            tree::dir_of(&t, &nodes.paths[i]).unwrap_or(0)
        };
        sink.row("in_dir", vec![i as u64, dir as u64]);
    }
    let mut by_id: Vec<String> = vec![String::new(); t.dirs.len()];
    for (path, id) in &t.ids {
        by_id[*id] = path.clone();
    }
    for (id, dir) in t.dirs.iter().enumerate() {
        sink.row("dir", vec![id as u64]);
        if id != 0 {
            sink.row("parent", vec![id as u64, dir.parent as u64]);
        }
        let name = by_id[id]
            .rsplit('/')
            .next()
            .filter(|n| !n.is_empty())
            .unwrap_or(".");
        labels
            .names
            .entry(legend::sym(name))
            .or_insert_with(|| name.to_string());
        sink.row("dir_name", vec![id as u64, legend::sym(name)]);
    }
    labels.dirs = by_id
        .into_iter()
        .map(|p| if p.is_empty() { ".".to_string() } else { p })
        .collect();
}
