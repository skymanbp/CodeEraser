//! One resolve/1 request, lowered (plan v2.33 wave W2a): the walked
//! files and their directories as segment ids, the core's vocabulary
//! and the affix rows, each language's facts, the sites. Every string
//! is interned here and nowhere else; the request carries the tree
//! once, with only the sites that need resolving.
//!
//! Files travel in path order (the C closure's work order reads it); a
//! site's own file that is not a walked file (an instrument's case)
//! follows them, marked unwalked, so the core can read its directory
//! without ever answering it as a target.

use super::facts::{Databases, Facts};
use super::intern::Intern;
use super::tokens;
use crate::graph::compdb::{self};
use crate::graph::compdb_flags::{Chain, Search};
use crate::graph::ladder::Site;
use crate::graph::roots;
use crate::scan::lang::Lang;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

/// What a request reads of the sweep besides its facts.
pub struct Input<'a> {
    pub files: &'a BTreeSet<String>,
    pub includes: &'a BTreeMap<String, Vec<String>>,
    pub root: &'a Path,
}

/// The request body and the two tables a reply names by index.
pub struct Lowered {
    pub body: Value,
    pub files: Vec<String>,
    pub dirs: Vec<String>,
}

pub fn lower(input: &Input, facts: &Facts, sites: &[(Lang, &Site)]) -> Result<Lowered, String> {
    let mut seg = Intern::default();
    let vocab = vocab(&mut seg);
    let origins: BTreeSet<&str> = sites
        .iter()
        .map(|(_, s)| s.from)
        .filter(|f| !input.files.contains(*f))
        .collect();
    let files: Vec<String> = input
        .files
        .iter()
        .map(String::as_str)
        .chain(origins.iter().copied())
        .map(str::to_string)
        .collect();
    let file_ids: HashMap<&str, usize> = files.iter().map(String::as_str).zip(0..).collect();
    let (dirs, dir_rows, dir_ids) = dir_table(&files, &mut seg);
    let walked = input.files.len();
    let file_rows: Vec<[i64; 4]> = files
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let lang = Lang::from_path(Path::new(f)).map_or(7, |l| l as i64);
            let dir = dir_ids[roots::parent_dir(f).as_str()];
            [dir, seg.id(base(f)), lang, i64::from(i < walked)]
        })
        .collect();
    let site_rows = sites
        .iter()
        .map(|(lang, s)| tokens::row(*lang, s, file_ids[s.from] as i64, &mut seg))
        .collect::<Result<Vec<_>, String>>()?;
    let c = c_facts(input, &facts.c, &facts.c_roots, &file_ids, &mut seg);
    let py = json!({
        "roots": facts.py_roots.iter().map(|r| seg.pieces(r, '/')).collect::<Vec<_>>(),
        "deps": facts.py_deps.iter().map(|d| seg.id(d)).collect::<Vec<_>>(),
    });
    let lua = lua_facts(facts, &mut seg);
    let go = go_facts(facts, &mut seg);
    let affixes = affix_rows(&files, &dirs, facts, &mut seg);
    let body = json!({
        "segs": seg.count(), "vocab": vocab, "affixes": affixes, "dirs": dir_rows,
        "files": file_rows, "sites": site_rows, "py": py, "lua": lua, "go": go, "c": c,
    });
    Ok(Lowered { body, files, dirs })
}

/// The core's words then its affixes, each interned, in its order.
fn vocab(seg: &mut Intern) -> Vec<i64> {
    let words = &crate::tables::get().resolve;
    words
        .words
        .iter()
        .chain(words.affixes.iter())
        .map(|w| seg.id(w))
        .collect()
}

fn base(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Every directory holding a file, root excluded, in path order (a
/// parent sorts before its children): the paths, the `[parent, name]`
/// rows, and each path's id (the root is 0).
fn dir_table(
    files: &[String],
    seg: &mut Intern,
) -> (Vec<String>, Vec<[i64; 2]>, HashMap<String, i64>) {
    let all: BTreeSet<String> = files
        .iter()
        .flat_map(|f| {
            roots::ancestors(&roots::parent_dir(f))
                .filter(|d| !d.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect();
    let mut ids: HashMap<String, i64> = HashMap::from([(String::new(), 0)]);
    let mut rows = Vec::new();
    let mut paths = vec![String::new()];
    for d in all {
        let parent = ids[roots::parent_dir(&d).as_str()];
        rows.push([parent, seg.id(base(&d))]);
        ids.insert(d.clone(), rows.len() as i64);
        paths.push(d);
    }
    (paths, rows, ids)
}

/// `[affix, prefix, whole]` for every file and directory name that ends
/// with an affix the core appends or tests (its vocabulary's) or a Lua
/// template glues on (the piece before the template's first `/`).
fn affix_rows(files: &[String], dirs: &[String], facts: &Facts, seg: &mut Intern) -> Vec<[i64; 3]> {
    let names: BTreeSet<&str> = files.iter().chain(dirs).map(|p| base(p)).collect();
    let heads = facts.lua_templates.iter().map(|t| head_tail(&t.suffix).0);
    let affixes: BTreeSet<&str> = crate::tables::get()
        .resolve
        .affixes
        .iter()
        .copied()
        .chain(heads)
        .filter(|a| !a.is_empty())
        .collect();
    let mut rows = BTreeSet::new();
    for a in affixes {
        for n in names.iter().filter(|n| n.ends_with(a)) {
            rows.insert([seg.id(a), seg.id(&n[..n.len() - a.len()]), seg.id(n)]);
        }
    }
    rows.into_iter().collect()
}

/// A template suffix's piece before its first `/` and the rest.
fn head_tail(suffix: &str) -> (&str, Option<&str>) {
    match suffix.split_once('/') {
        Some((h, t)) => (h, Some(t)),
        None => (suffix, None),
    }
}

fn lua_facts(facts: &Facts, seg: &mut Intern) -> Value {
    let templates: Vec<Vec<i64>> = facts
        .lua_templates
        .iter()
        .map(|t| {
            let (head, tail) = head_tail(&t.suffix);
            let dir = seg.pieces(&t.dir, '/');
            let mut row = vec![seg.id(head), dir.len() as i64];
            row.extend(dir);
            row.extend(tail.map_or_else(Vec::new, |t| seg.pieces(t, '/')));
            row
        })
        .collect();
    json!({
        "roots": facts.lua_roots.iter().map(|r| seg.pieces(r, '/')).collect::<Vec<_>>(),
        "templates": templates,
    })
}

fn go_facts(facts: &Facts, seg: &mut Intern) -> Value {
    let mut mods = Vec::new();
    let mut replaces = Vec::new();
    for (k, m) in facts.go_mods.iter().enumerate() {
        let dir = seg.dir(&m.dir);
        let mut row = vec![dir.len() as i64];
        row.extend(dir);
        row.extend(seg.pieces(m.module.as_deref().unwrap_or_default(), '/'));
        mods.push(row);
        for (old, new) in &m.replaces {
            let path = new.starts_with("./") || new.starts_with("../");
            let dotted = new.split('/').next().is_some_and(|h| h.contains('.'));
            let old = seg.pieces(old, '/');
            let mut row = vec![
                k as i64,
                i64::from(path) * 2 + i64::from(dotted),
                old.len() as i64,
            ];
            row.extend(old);
            row.extend(seg.pieces(new, '/'));
            replaces.push(row);
        }
    }
    json!({ "mods": mods, "replaces": replaces })
}

fn c_facts(
    input: &Input,
    db: &Databases,
    roots_c: &[String],
    file_ids: &HashMap<&str, usize>,
    seg: &mut Intern,
) -> Value {
    let (mut searches, mut forced) = (Vec::new(), Vec::new());
    for (c, chain) in db.chains.iter().enumerate() {
        chain_rows(input.root, c as i64, chain, seg, &mut searches, &mut forced);
    }
    let seats: Vec<Vec<i64>> = db
        .seats
        .iter()
        .map(|(file, dir, chain)| {
            let mut row = vec![
                file_ids[file.as_str()] as i64,
                *chain as i64,
                i64::from(dir.is_some()),
            ];
            row.extend(dir.as_deref().map_or_else(Vec::new, |d| seg.dir(d)));
            row
        })
        .collect();
    let includes: Vec<Vec<i64>> = input
        .files
        .iter()
        .filter_map(|f| Some((file_ids[f.as_str()], input.includes.get(f)?)))
        .flat_map(|(id, specs)| specs.iter().map(move |s| (id, s)))
        .map(|(id, spec)| {
            let (name, system) = tokens::c_form(spec);
            let mut row = vec![id as i64, i64::from(system)];
            row.extend(seg.pieces(name, '/'));
            row
        })
        .collect();
    json!({
        "roots": roots_c.iter().map(|r| seg.pieces(r, '/')).collect::<Vec<_>>(),
        "chains": db.chains.iter().map(|c| [i64::from(c.msvc), i64::from(c.own_dir)]).collect::<Vec<_>>(),
        "searches": searches, "forced": forced, "seats": seats,
        "jsonDirs": db.json_dirs.iter().map(|d| seg.dir(d)).collect::<Vec<_>>(),
        "flags": db.flags.iter().map(|(d, c)| [vec![*c as i64], seg.dir(d)].concat()).collect::<Vec<_>>(),
        "includes": includes,
    })
}

/// One chain's searched places (class 0 quote, 1 bracket, 2 system)
/// and its forced includes — an absolute one placed into the tree here
/// (or outside it), a spelling left to the core's search.
fn chain_rows(
    root: &Path,
    c: i64,
    chain: &Chain,
    seg: &mut Intern,
    searches: &mut Vec<Vec<i64>>,
    forced: &mut Vec<Vec<i64>>,
) {
    for (class, places) in [&chain.quote, &chain.bracket, &chain.system]
        .iter()
        .enumerate()
    {
        for place in places.iter() {
            let (kind, dir) = match place {
                Search::Dir(d) => (0, d),
                Search::Framework(d) => (1, d),
            };
            searches.push([vec![c, class as i64, kind], seg.dir(dir)].concat());
        }
    }
    for spec in &chain.forced {
        let row = if compdb::is_absolute(spec) {
            match compdb::relativize(&compdb::root_text(root), "", spec) {
                Some(p) => [vec![c, 1], seg.dir(&p)].concat(),
                None => vec![c, 2],
            }
        } else {
            [vec![c, 0], seg.pieces(spec, '/')].concat()
        };
        forced.push(row);
    }
}
