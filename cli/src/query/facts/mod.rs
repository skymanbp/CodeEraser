//! The fact tables (design booklet §4.2): the request half the
//! measuring side assembles from its own index — request-local ids,
//! name hashes, integers, nothing else (§5.9.2) — and the labels
//! every answer reads back through. Only the tables the program
//! names are built, and the costly ones (a scan for the complexity
//! numbers, the T3 and docdup judgments, the mention pass, a read of
//! every file for its line count) only when their predicate is read.

pub mod graph;
pub mod pairs;
pub mod text;
pub mod units;

use super::legend;
use super::lexed::Lexed;
use crate::dedup::index::Index;
use crate::dedup::pairs::Blocks;
use crate::graph::deadcode::{Advisory, GraphWire};
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The labels: an id of each sort back to the text it stands for.
#[derive(Default, Debug)]
pub struct Labels {
    pub nodes: Vec<String>,
    pub dirs: Vec<String>,
    pub units: Vec<String>,
    /// Every name the program or a table spelled, by hash.
    pub names: BTreeMap<u64, String>,
    pub sets: Vec<String>,
}

impl Labels {
    /// The directories the core built (`[label, name, hash]` by id, the
    /// root's label `.`): each label, and each name by its hash.
    pub fn adopt_dirs(&mut self, dirs: Vec<(String, String, u64)>) {
        for (label, name, hash) in dirs {
            self.names.entry(hash).or_insert(name);
            self.dirs.push(label);
        }
    }

    /// One answer value as text, by the sort the core answered.
    pub fn render(&self, sort: i64, v: i128) -> String {
        let at = |list: &[String], word: &str| {
            usize::try_from(v)
                .ok()
                .and_then(|i| list.get(i))
                .cloned()
                .unwrap_or_else(|| format!("{word}#{v}"))
        };
        match legend::sort_name(sort) {
            "node" => at(&self.nodes, "node"),
            "dir" => at(&self.dirs, "dir"),
            "unit" => at(&self.units, "unit"),
            "set" => at(&self.sets, "set"),
            "sym" => u64::try_from(v)
                .ok()
                .and_then(|h| self.names.get(&h))
                .cloned()
                .unwrap_or_else(|| format!("#{v}")),
            _ => v.to_string(),
        }
    }
}

/// The assembled request half.
pub struct Facts {
    /// Rows per predicate code, each table sorted and deduplicated.
    pub tables: BTreeMap<u32, Vec<Vec<u64>>>,
    /// The tree form (graph::tree), when the program reads a directory.
    pub tree: Option<serde_json::Value>,
    pub labels: Labels,
}

/// The sink the assemblers fill: a table is a set, and only a table
/// the program named is kept. The schema's codes by name are the
/// core's (the lexed program's predicate names, CE.Query.Schema).
pub struct Sink {
    codes: BTreeMap<String, u32>,
    wanted: BTreeSet<u32>,
    tables: BTreeMap<u32, BTreeSet<Vec<u64>>>,
}

impl Sink {
    fn new(program: &Lexed) -> Sink {
        let wanted = program.referenced.clone();
        Sink {
            codes: program.schema(),
            tables: wanted.iter().map(|c| (*c, BTreeSet::new())).collect(),
            wanted,
        }
    }

    fn code(&self, name: &str) -> u32 {
        *self
            .codes
            .get(name)
            .unwrap_or_else(|| panic!("schema predicate {name:?}"))
    }

    /// Whether the program reads this predicate.
    pub fn wants(&self, name: &str) -> bool {
        self.wanted.contains(&self.code(name))
    }

    pub fn wants_any(&self, names: &[&str]) -> bool {
        names.iter().any(|n| self.wants(n))
    }

    /// One row of a predicate's table; dropped when nothing reads it.
    pub fn row(&mut self, name: &str, row: Vec<u64>) {
        let code = self.code(name);
        if let Some(t) = self.tables.get_mut(&code) {
            t.insert(row);
        }
    }

    fn finish(self) -> BTreeMap<u32, Vec<Vec<u64>>> {
        self.tables
            .into_iter()
            .map(|(c, rows)| (c, rows.into_iter().collect()))
            .collect()
    }
}

/// What every assembler reads.
pub struct Ctx<'a> {
    pub root: &'a Path,
    pub idx: &'a Index,
    pub core: &'a str,
    pub wire: &'a GraphWire,
    pub nodes: &'a graph::Nodes,
    pub blocks: &'a Blocks,
}

/// The index refreshed, the graph wire built, and every table the
/// program names filled.
pub fn assemble(root: &Path, db: Option<PathBuf>, core: &str, program: &Lexed) -> Result<Facts> {
    let (blocks, idx, db_path) = crate::dedup::snapshot(root, db)?;
    let wire = crate::graph::deadcode::wire_of(root, &idx, &db_path, Advisory::No)?;
    let nodes = graph::Nodes::build(&wire, &idx)?;
    let mut sink = Sink::new(program);
    let mut labels = Labels {
        nodes: nodes.labels.clone(),
        names: dictionary(program),
        sets: program.sets.clone(),
        ..Labels::default()
    };
    let ctx = Ctx {
        root,
        idx: &idx,
        core,
        wire: &wire,
        nodes: &nodes,
        blocks: &blocks,
    };
    graph::fill(&ctx, &mut sink)?;
    let units = units::fill(&ctx, &mut sink, &mut labels)?;
    pairs::fill(&ctx, &units, &mut sink)?;
    text::fill(&ctx, &program.sets, &mut sink, &mut labels)?;
    let tree = graph::tree(&nodes, &mut sink);
    Ok(Facts {
        tables: sink.finish(),
        tree,
        labels,
    })
}

/// The reverse dictionary a `sym` answer reads through: every enum
/// word the assembler can spell, and every name the program spelled.
fn dictionary(program: &Lexed) -> BTreeMap<u64, String> {
    let mut names: BTreeMap<u64, String> = legend::vocabulary()
        .map(|w| (legend::sym(w), w.to_string()))
        .collect();
    names.extend(program.names().map(|(h, n)| (h, n.clone())));
    names
}
