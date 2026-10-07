//! The one document of `ce arch`, the MCP tool `architecture` and the
//! GUI's reports hub (design booklet §7.3): the directory layers, the
//! cut arcs with the file references behind each, the clusters, the
//! files outside their cluster's directory, the impact of the focus
//! and the per-directory metrics. The core builds the tables from the
//! paths and judges them (arch/1), and lays the document out
//! (document/1, CE.Arch.Document); this side sends the paths and the
//! arcs, then the tables the core built and the paths and directory
//! names back; the core spells them into the document and its console
//! lines, orders them and measures them (crate::document); the CLI
//! prints them. A document with a `degraded` reason carries no answer.

use super::wire::{self, Ask, Built, Reply};
use crate::document::{self, Answer, Request, Why};
use crate::graph::deadcode::{Advisory, wire_of};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// The whole leg: the graph wire off a refreshed index, its measured
/// files and arcs with their line counts, the core's tables, judgment
/// and document. A focus path that names no measured file is an error
/// (the core's fault) and no document.
pub fn run(root: &Path, db: Option<PathBuf>, core: &str, focus: &[String]) -> Result<Answer> {
    let (idx, db_path) = crate::dedup::refreshed_index(root, db)?;
    let w = wire_of(root, &idx, &db_path, Advisory::No)?;
    drop(idx);
    let arcs = crate::structure::rows::arcs(&w);
    let lines = (arcs.paths.iter())
        .map(|p| line_count(root, p))
        .collect::<Result<Vec<i64>>>()?;
    let ask = Ask {
        arcs,
        lines,
        focus: focus.to_vec(),
    };
    let mut held = document::open(core);
    let mut why = Why::default();
    let (built, judgment) = wire::judge(&mut held, &ask)?;
    let req = match judgment {
        Ok(reply) => request(&built, &reply),
        Err(e) => Request::new("arch").empty(&TABLES).degraded(why.add(e)),
    };
    let req = req
        .range("files", ask.arcs.paths.len())
        .range("dirs", built.dir_paths.len())
        .range("why", why.count())
        .text("path", &ask.arcs.paths)
        .text("dir", &built.dir_paths)
        .text("why", why.list());
    document::assemble_over(core, held, req)
}

/// A file's total lines (scan's own count; a file deleted mid-run has
/// none).
fn line_count(root: &Path, path: &str) -> Result<i64> {
    let bytes = crate::scan::walk::read_surviving(&root.join(path))?;
    Ok(bytes.map_or(0, |b| crate::scan::metrics::size::total_lines(&b) as i64))
}

/// The document request's tables: the arch/1 request's, its answer's,
/// and the two rank tables.
const TABLES: [&str; 13] = [
    "files",
    "dirs",
    "edges",
    "pkgEdges",
    "focus",
    "layers",
    "cuts",
    "clusters",
    "misplaced",
    "impact",
    "metrics",
    "rankFiles",
    "rankDirs",
];

/// The request tables sent back and the answer tables (the core
/// orders the paths and measures the directories off the strings).
fn request(t: &Built, r: &Reply) -> Request {
    let focus: Vec<[i64; 1]> = t.focus.iter().map(|f| [*f]).collect();
    let mut req = Request::new("arch").rows("focus", focus);
    for (key, _) in wire::BUILT.lines().filter_map(|l| l.split_once(' ')) {
        req = req.rows(key, t.rows(key));
    }
    for key in [
        "layers",
        "cuts",
        "clusters",
        "misplaced",
        "impact",
        "metrics",
    ] {
        req = req.rows(key, r.rows(key));
    }
    req
}
