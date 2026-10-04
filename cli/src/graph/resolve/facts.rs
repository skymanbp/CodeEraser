//! The configuration a resolve/1 request carries, read once per sweep
//! (plan v2.33 wave W2a): what the four ladders the core holds learn
//! from files besides the sites — `pyproject.toml`'s source directories
//! and dependencies, the declared `[graph.search_roots]`, the Lua
//! `package.path` templates the walk read, each go.mod's module and
//! replace directives, and the compile databases clangd would find with
//! the chain each entry's argv defines. Reading and parsing those files
//! is the measuring side's; which candidate a site tries is the core's.
//! Kept as text here and lowered to segment ids per request (lower.rs).

use crate::graph::compdb::{self};
use crate::graph::compdb_find::{self, Found};
use crate::graph::compdb_flags::Chain;
use crate::graph::gomod::{self, GoMod};
use crate::graph::ladder::{Scope, lua_path::Template, members};
use crate::graph::roots;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One sweep's configuration facts, as text.
#[derive(Default)]
pub struct Facts {
    pub py_roots: Vec<String>,
    pub py_deps: Vec<String>,
    pub lua_roots: Vec<String>,
    pub lua_templates: Vec<Template>,
    pub go_mods: Vec<GoMod>,
    pub c_roots: Vec<String>,
    pub c: Databases,
}

/// The compile databases: every distinct chain, the files the JSON
/// entries seat (file, the entry's working directory, chain) in the
/// order clangd's probes find them, the directories whose probe found a
/// JSON database, and each `compile_flags.txt` directory's chain.
#[derive(Default)]
pub struct Databases {
    pub chains: Vec<Chain>,
    pub seats: Vec<(String, Option<String>, usize)>,
    pub json_dirs: BTreeSet<String>,
    pub flags: BTreeMap<String, usize>,
}

impl Facts {
    pub fn of(scope: &Scope) -> Facts {
        let py = roots::pyproject(scope.root);
        let declared = |lang: &str| -> Vec<String> {
            scope
                .search_roots
                .get(lang)
                .into_iter()
                .flatten()
                .cloned()
                .collect()
        };
        Facts {
            py_roots: py.as_ref().map_or_else(Vec::new, |p| p.source_dirs.clone()),
            py_deps: py.map_or_else(Vec::new, |p| p.deps),
            lua_roots: declared("lua"),
            lua_templates: scope.lua.iter().cloned().collect(),
            go_mods: members(scope, "go.mod", gomod::parse, |m| m.module.is_some()),
            c_roots: declared("c"),
            c: Databases::gather(scope.root, scope.files),
        }
    }

    /// Only the compile databases: what the deadcode request needs for
    /// the forced-include arcs.
    pub fn databases(root: &Path, files: &BTreeSet<String>) -> Facts {
        Facts {
            c: Databases::gather(root, files),
            ..Facts::default()
        }
    }
}

impl Databases {
    /// The databases and the files their entries seat: a JSON database
    /// read once however many probes find it, an entry kept when its
    /// unit is a walked file.
    pub fn gather(root: &Path, files: &BTreeSet<String>) -> Databases {
        let mut db = Databases::default();
        let mut ids: BTreeMap<Chain, usize> = BTreeMap::new();
        let mut read = BTreeSet::new();
        for Found { dir, probe, rel } in compdb_find::found(root, files.iter()) {
            if probe == 2 {
                if let Some(f) = compdb::parse_flags(root, &rel) {
                    let chain = db.chain_id(&mut ids, f.chain);
                    db.flags.insert(dir, chain);
                }
                continue;
            }
            db.json_dirs.insert(dir);
            if !read.insert(rel.clone()) {
                continue;
            }
            let entries = compdb::parse(root, &rel).map(|parsed| parsed.entries);
            for e in entries.into_iter().flatten() {
                if files.contains(&e.unit) {
                    let chain = db.chain_id(&mut ids, e.chain);
                    db.seats.push((e.unit, e.dir, chain));
                }
            }
        }
        db
    }

    /// One chain's id, structural equality deciding: two entries with
    /// one chain are one closure.
    fn chain_id(&mut self, ids: &mut BTreeMap<Chain, usize>, chain: Chain) -> usize {
        *ids.entry(chain.clone()).or_insert_with(|| {
            self.chains.push(chain);
            self.chains.len() - 1
        })
    }
}
