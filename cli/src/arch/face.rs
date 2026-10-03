//! The one document of `ce arch`, the MCP tool `architecture` and the
//! GUI's reports hub (design booklet §7.3): the directory layers, the
//! cut arcs with the file references behind each, the clusters, the
//! files outside their cluster's directory, the impact of the focus
//! and the per-directory metrics. The core judges them (arch/1) and
//! lays the document out (document/1, CE.Arch.Document); this side
//! sends the tables both answers are made of, each path's place in
//! string order, and puts the paths back into the document and its
//! console lines (crate::document); the CLI prints them. A
//! document with a `degraded` reason carries no answer.

use super::tables::{self, Tables};
use super::wire::{self, Reply};
use crate::document::{self, Answer, Request, Resolve, Why};
use crate::graph::deadcode::{Advisory, wire_of};
use crate::structure::tree;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// The whole leg: the graph wire off a refreshed index, the tree over
/// its measured files, the tables, the core's judgment and document.
/// A focus path that names no measured file is an error and no
/// document.
pub fn run(root: &Path, db: Option<PathBuf>, core: &str, focus: &[String]) -> Result<Answer> {
    let (idx, db_path) = crate::dedup::refreshed_index(root, db)?;
    let w = wire_of(root, &idx, &db_path, Advisory::No)?;
    drop(idx);
    let paths = tables::measured_paths(&w);
    let t = tree::build(&paths);
    let lines = paths
        .iter()
        .map(|p| line_count(root, p))
        .collect::<Result<Vec<i64>>>()?;
    let tables = tables::assemble(&w, &t, focus, &lines)?;
    let mut held = document::open(core);
    let mut why = Why::default();
    let req = match wire::judge(&mut held, &tables)? {
        Ok(reply) => request(&tables, &reply),
        Err(e) => Request::new("arch").empty(&TABLES).degraded(why.add(e)),
    };
    let req = req
        .range("files", tables.paths.len())
        .range("dirs", tables.dir_paths.len())
        .range("why", why.count());
    let names = Names { t: &tables, why };
    document::assemble_over(core, held, req, &names)
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

/// The request tables sent back, the answer tables, each path's place
/// in the joint string order of the file paths and the slashed
/// directories (the order the folded references are listed in), and
/// each directory's width in bytes and in characters (the console
/// pads its metrics table by them; the core cannot measure a string).
fn request(t: &Tables, r: &Reply) -> Request {
    let slashed: Vec<String> = t.dir_paths.iter().map(|d| slashed(d)).collect();
    let rank = document::ranks(t.paths.iter().chain(&slashed).map(String::as_str));
    let (by_file, by_dir) = rank.split_at(t.paths.len());
    let numbered = |ranks: &[usize]| -> Vec<[usize; 2]> {
        ranks.iter().enumerate().map(|(i, r)| [i, *r]).collect()
    };
    let focus: Vec<[i64; 1]> = t.focus.iter().map(|f| [*f]).collect();
    let mut req = Request::new("arch")
        .rows("files", &t.files)
        .rows("dirs", &t.dirs)
        .rows("edges", &t.edges)
        .rows("pkgEdges", &t.pkg_edges)
        .rows("focus", focus)
        .rows("rankFiles", numbered(by_file))
        .rows("rankDirs", numbered(by_dir))
        .rows("widths", widths(&t.dir_paths));
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

/// `[dir, bytes, chars]` for every directory (the root is 0 bytes).
fn widths(dirs: &[String]) -> Vec<[usize; 3]> {
    let width = |(i, d): (usize, &String)| [i, d.len(), d.chars().count()];
    dirs.iter().enumerate().map(width).collect()
}

/// The arch document's strings: the paths, the directories, and the
/// reason the judgment did not happen.
struct Names<'a> {
    t: &'a Tables,
    why: Why,
}

impl Resolve for Names<'_> {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        match class {
            "path" => document::at(&self.t.paths, ints),
            "dir" => document::at(&self.t.dir_paths, ints),
            "slashed" => document::at(&self.t.dir_paths, ints).map(|d| slashed(&d)),
            "why" => self.why.at(ints),
            _ => None,
        }
    }
}

/// A directory as an arc end: its path with a trailing slash, the
/// root as `./`.
pub fn slashed(dir: &str) -> String {
    if dir.is_empty() {
        "./".into()
    } else {
        format!("{dir}/")
    }
}
