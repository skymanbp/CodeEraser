//! What the compile databases say about each C-family file (plan v2.30
//! step 5b, item 14): the databases clangd would find (compdb_find.rs),
//! the chain each entry's argv defines (compdb_flags.rs), the files a
//! `compile_flags.txt` covers, and — since a database lists translation
//! units and a header has no entry — the include closure of each chain:
//! a header is compiled with the chain of every translation unit whose
//! includes reach it, so its own `#include` lines are answered under
//! those chains (headers via their including units; clangd guesses a
//! nearby unit's command instead, `inferMissingCompileCommands`, which
//! the register's no-guess rule forbids). The closure walks the include
//! lists the walk read (Scope::includes, ladder/c_head.rs) and never
//! opens a file. One index per sweep (ladder::Memo); the deadcode
//! request gathers the entries alone for the forced-include arcs.

use super::{Scope, c_search};
use crate::graph::compdb::{self, Flags};
use crate::graph::compdb_find::{self, Found};
use crate::graph::compdb_flags::Chain;
use crate::graph::roots;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::rc::Rc;

/// The include DAG one chain's closure read, by child.
type Parents = BTreeMap<String, BTreeSet<String>>;

/// One seat: a file, the working directory of the entry seating it
/// (None = outside the tree), its chain, and whether a JSON entry
/// (a translation unit by the format's definition) rather than a
/// flags file's coverage seated it.
struct Seat {
    file: String,
    dir: Option<String>,
    chain: usize,
    unit: bool,
}

#[derive(Default)]
pub struct Index {
    /// Every distinct chain: the databases' entries and, one each, the
    /// flags files that cover a file.
    pub chains: Vec<Chain>,
    seats: Vec<Seat>,
    /// file → the chains its own seats define.
    units: BTreeMap<String, BTreeSet<usize>>,
    /// Per chain, the closure's include DAG.
    parents: Vec<Parents>,
    /// Directories whose clangd probe found a JSON database.
    json_dirs: BTreeSet<String>,
    /// compile_flags.txt by the directory holding it.
    flags: BTreeMap<String, Flags>,
}

/// The sweep's index: gathered once per sweep, its closure walked once.
pub fn index(scope: &Scope) -> Rc<Index> {
    scope.memo.cached("c:index", "", || {
        let mut idx = Index::gather(scope.root, scope.files);
        idx.close(scope);
        idx
    })
}

impl Index {
    /// The databases and what seats each file — no closure yet: what
    /// the deadcode request needs for the forced-include arcs.
    pub fn gather(root: &Path, files: &BTreeSet<String>) -> Index {
        let mut idx = Index::default();
        let mut ids: BTreeMap<Chain, usize> = BTreeMap::new();
        let mut read = BTreeSet::new();
        for Found { dir, probe, rel } in compdb_find::found(root, files.iter()) {
            if probe == 2 {
                if let Some(f) = compdb::parse_flags(root, &rel) {
                    idx.flags.insert(dir, f);
                }
                continue;
            }
            idx.json_dirs.insert(dir);
            if !read.insert(rel.clone()) {
                continue;
            }
            let entries = compdb::parse(root, &rel).map(|db| db.entries);
            for e in entries.into_iter().flatten() {
                if files.contains(&e.unit) {
                    let chain = idx.chain_id(&mut ids, e.chain);
                    idx.seat(e.unit, e.dir, chain, true);
                }
            }
        }
        idx.cover(files, &mut ids);
        idx
    }

    fn chain_id(&mut self, ids: &mut BTreeMap<Chain, usize>, chain: Chain) -> usize {
        *ids.entry(chain.clone()).or_insert_with(|| {
            self.chains.push(chain);
            self.chains.len() - 1
        })
    }

    fn seat(&mut self, file: String, dir: Option<String>, chain: usize, unit: bool) {
        self.units.entry(file.clone()).or_default().insert(chain);
        self.seats.push(Seat {
            file,
            dir,
            chain,
            unit,
        });
    }

    /// The files a compile_flags.txt covers: every C-family file with
    /// no entry whose nearest database directory holds one — clangd's
    /// rule, the nearest ancestor holding any database decides, a JSON
    /// one first. A covered file compiles with that chain from the
    /// flags file's directory.
    fn cover(&mut self, files: &BTreeSet<String>, ids: &mut BTreeMap<Chain, usize>) {
        let covered: Vec<(String, String)> = files
            .iter()
            .filter(|f| compdb_find::is_c(f) && !self.units.contains_key(*f))
            .filter_map(|f| self.nearest_flags(f).map(|d| (f.clone(), d)))
            .collect();
        for (file, dir) in covered {
            let chain = self.chain_id(ids, self.flags[&dir].chain.clone());
            let unit = compdb_find::is_unit(&file);
            self.seat(file, Some(dir), chain, unit);
        }
    }

    /// The directory of the flags file covering a file, None when the
    /// nearest database directory holds a JSON database or no ancestor
    /// holds anything.
    fn nearest_flags(&self, file: &str) -> Option<String> {
        let dir = roots::parent_dir(file);
        roots::ancestors(&dir)
            .find_map(|d| {
                if self.json_dirs.contains(d) {
                    Some(None)
                } else {
                    self.flags.contains_key(d).then(|| Some(d.to_string()))
                }
            })
            .flatten()
    }

    /// The closure of every chain. The MSVC stack grows with the DAG
    /// it is read from, so an MSVC chain's pass repeats until it adds
    /// nothing; the GNU dialect reads no stack and one pass settles.
    fn close(&mut self, scope: &Scope) {
        self.parents = vec![Parents::new(); self.chains.len()];
        for c in 0..self.chains.len() {
            while self.pass(c, scope) && self.chains[c].msvc {}
        }
    }

    /// One closure pass over a chain from its seats: forced includes
    /// first (the entry directory then the quoted chain), then every
    /// `#include` of every reached file under the chain. Whether the
    /// pass added a parent link.
    fn pass(&mut self, c: usize, scope: &Scope) -> bool {
        let chain = self.chains[c].clone();
        let seats: Vec<(String, Option<String>)> = self
            .seats
            .iter()
            .filter(|s| s.chain == c)
            .map(|s| (s.file.clone(), s.dir.clone()))
            .collect();
        let mut seen: BTreeSet<String> = seats.iter().map(|(f, _)| f.clone()).collect();
        let mut work: Vec<String> = seen.iter().cloned().collect();
        let mut grew = false;
        for (file, dir) in &seats {
            for spec in &chain.forced {
                let hit = c_search::forced(dir.as_deref(), spec, &chain, scope.root, scope.files);
                if let Some(to) = hit {
                    grew |= self.link(c, file, to, &mut seen, &mut work);
                }
            }
        }
        while let Some(from) = work.pop() {
            let stack = self.stack(c, &from);
            for spec in scope.includes.get(&from).into_iter().flatten() {
                for to in c_search::targets(&from, spec, &chain, &stack, scope) {
                    grew |= self.link(c, &from, to, &mut seen, &mut work);
                }
            }
        }
        grew
    }

    fn link(
        &mut self,
        c: usize,
        from: &str,
        to: String,
        seen: &mut BTreeSet<String>,
        work: &mut Vec<String>,
    ) -> bool {
        let new = self.parents[c]
            .entry(to.clone())
            .or_default()
            .insert(from.to_string());
        if seen.insert(to.clone()) {
            work.push(to);
        }
        new
    }

    /// The chains a file compiles under: its own seats, else the chains
    /// whose closure reaches it. Empty = no database speaks of it.
    pub fn chains_of(&self, file: &str) -> BTreeSet<usize> {
        if let Some(own) = self.units.get(file) {
            return own.clone();
        }
        (0..self.chains.len())
            .filter(|c| self.parents[*c].contains_key(file))
            .collect()
    }

    /// The directories of the files on the include paths from a
    /// chain's units down to `file` — cl's "currently opened include
    /// files" for the quoted form (the MSVC dialect; empty otherwise).
    pub fn stack(&self, chain: usize, file: &str) -> BTreeSet<String> {
        if !self.chains[chain].msvc {
            return BTreeSet::new();
        }
        let mut seen = BTreeSet::new();
        let mut work = vec![file.to_string()];
        while let Some(f) = work.pop() {
            for p in self.parents[chain].get(&f).into_iter().flatten() {
                if seen.insert(p.clone()) {
                    work.push(p.clone());
                }
            }
        }
        seen.iter().map(|p| roots::parent_dir(p)).collect()
    }

    /// The forced includes of every translation unit, resolved: the
    /// (unit, header) arcs a build declares outside the source text.
    fn forced_arcs(&self, root: &Path, files: &BTreeSet<String>) -> Vec<(String, String)> {
        let mut out = BTreeSet::new();
        for s in self.seats.iter().filter(|s| s.unit) {
            let chain = &self.chains[s.chain];
            for spec in &chain.forced {
                if let Some(h) = c_search::forced(s.dir.as_deref(), spec, chain, root, files) {
                    out.insert((s.file.clone(), h));
                }
            }
        }
        out.into_iter().collect()
    }
}

/// The forced-include arcs as wire edges (deadcode.rs): unit → header,
/// the import kind, the database's rung — beside the stored edges,
/// which come only from source sites, and the containment arcs.
pub fn forced_wire(
    root: &Path,
    files: &BTreeSet<String>,
    ids: &BTreeMap<(&str, &str), usize>,
    wire: &mut BTreeSet<[i64; 4]>,
) {
    for (unit, header) in Index::gather(root, files).forced_arcs(root, files) {
        let (Some(u), Some(h)) = (
            ids.get(&(unit.as_str(), "")),
            ids.get(&(header.as_str(), "")),
        ) else {
            continue;
        };
        wire.insert([*u as i64, *h as i64, crate::graph::wire::EDGE_IMPORT, 3]);
    }
}
