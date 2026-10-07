//! The reference ladders the core holds (plan v2.33 wave W2a; on text
//! since W2-text, proto 9.0.0; design booklet
//! docs/reference/algorithm-track.md §3, §6): Python, TypeScript / TSX,
//! Rust, Lua, Go, C / C++, R, Java, Haskell, Markdown and HTML resolve in
//! `resolve/1`, with the readers of their configuration files (the tsconfig chains
//! and package.json files, the Cargo.toml files, go.mod, R's
//! DESCRIPTION, the .cabal files, the root pyproject.toml's keys, the
//! compile databases, their response and flag files). This side sends
//! what it read (request.rs) — one request per sweep with only the sites
//! that need resolving — answers the core's `wanted` response files and
//! `tsWanted` facts by reading them (facts.rs: the file system, a TOML
//! document, a Rust file's syntax tree), and maps each reply row back to
//! the ladder's `Outcome`, so the edge store, deadcode, `ce graph
//! --sites` and the precision documents read the answers they always
//! read; a batch with Markdown or HTML sites carries what this side read
//! of the documents up front (markdown.rs: anchor sets, reference tables,
//! the walked assets, each page's head facts and id set), so it asks no
//! extra round.
//!
//! The core is the one this process names (the global `--core`, then
//! CE_CORE_BIN, a sibling of this binary, PATH), held open across the
//! process's requests. A core that cannot answer is a named refusal:
//! there is no copy of the search on this side to fall back on.

mod facts;
mod manifests;
mod markdown;
mod request;

pub use manifests::{Declared, Manifests, declared, private};

use super::ladder::{Outcome, Reason, Rung, Scope, Site};
use crate::corelink::{Link, judged};
use crate::scan::lang::Lang;
use request::Input;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Mutex;

/// The capability the core must offer, the request kind, the proto that
/// minted the family's text form.
pub const CAP: &str = "resolve/1";
pub const KIND: &str = "resolve";
pub const SINCE: &str = "9.0.0";

/// Whether the core holds this language's ladder.
pub fn in_core(lang: Lang) -> bool {
    matches!(
        lang,
        Lang::Python
            | Lang::TypeScript
            | Lang::Tsx
            | Lang::Rust
            | Lang::Go
            | Lang::Markdown
            | Lang::Html
            | Lang::Haskell
            | Lang::C
            | Lang::Cpp
            | Lang::Lua
            | Lang::R
            | Lang::Java
    )
}

/// The sites' outcomes, in order — every site a language `in_core`. The
/// sweep's part of the request is read once per sweep (the memo), the
/// response files and facts the core asks for kept in it; each Rust
/// site's syntax-tree fact at its row goes up front (every Rust rung
/// reads it), and so do the Markdown and HTML facts (the document
/// facts, markdown.rs).
pub fn outcomes(sites: &[(Lang, &Site)], scope: &Scope) -> Result<Vec<Outcome>, String> {
    if sites.is_empty() {
        return Ok(Vec::new());
    }
    let tree = scope.memo.cached("resolve:tree", "", || {
        let input = Input {
            files: scope.files,
            root: scope.root,
            configs: scope.configs,
            search_roots: scope.search_roots,
            lua: scope
                .lua
                .iter()
                .map(|t| [t.dir.as_str(), t.suffix.as_str()])
                .collect(),
            includes: scope.includes,
            java: scope.java,
            crate_roots: scope.crate_roots,
        };
        RefCell::new(request::tree(&input))
    });
    let (rows, origins) = request::sites(scope.files, sites)?;
    let mut body = tree.borrow().clone();
    body["sites"] = rows;
    body["origins"] = json!(origins);
    markdown::add(&mut body, scope, sites);
    let rows: Vec<(i64, String, String)> = sites
        .iter()
        .filter(|(lang, _)| *lang == Lang::Rust)
        .map(|(_, s)| (4, s.from.to_string(), s.line.saturating_sub(1).to_string()))
        .collect();
    facts::answer_facts(&mut body, scope.root, &rows)?;
    let reply = complete(&mut body, scope.root)?;
    tree.borrow_mut()["c"]["responses"] = body["c"]["responses"].clone();
    tree.borrow_mut()["ts"]["facts"] = body["ts"]["facts"].clone();
    mapped(&reply, sites.len())
}

/// One reply row: `[rung, outcome, target, reason]`.
type Row = (Rung, i64, Option<String>, i64);

/// The reply's rows as outcomes, in order, a section row with its slug
/// (`sections`, by row); a row count other than the sites sent, or a slug
/// for no section row, is wire skew, named.
fn mapped(reply: &Value, sent: usize) -> Result<Vec<Outcome>, String> {
    let rows: Vec<Row> = judged::table(reply, "results")?;
    if rows.len() != sent || judged::count(reply, "sites")? != sent {
        return Err("resolve/1: wire skew: one result per site sent".into());
    }
    let mut sections = judged::table::<Vec<(usize, Option<String>)>>(reply, "sections")?
        .into_iter()
        .peekable();
    let out = rows
        .into_iter()
        .enumerate()
        .map(|(i, row)| outcome(row, sections.next_if(|(at, _)| *at == i).map(|(_, s)| s)))
        .collect::<Result<Vec<_>, String>>()?;
    match sections.next() {
        Some(_) => Err("resolve/1: wire skew: a section slug for no section row".into()),
        None => Ok(out),
    }
}

/// The build's forced includes (`-include x.h`) as unit → header import
/// arcs on the deadcode wire — arcs the source text never spells. Both
/// ends are file nodes of `ids`; an arc to a file outside them is none.
pub fn forced_wire(
    root: &Path,
    files: &BTreeSet<String>,
    ids: &BTreeMap<(&str, &str), usize>,
    wire: &mut BTreeSet<[i64; 4]>,
) -> Result<(), String> {
    let mut body =
        json!({ "files": files, "c": request::databases(root, files, &BTreeMap::new()) });
    if body["c"]["dbs"].as_array().is_none_or(Vec::is_empty) {
        return Ok(());
    }
    let reply = complete(&mut body, root)?;
    let arcs: Vec<[String; 2]> = judged::table(&reply, "forced")?;
    for [unit, header] in arcs {
        if let (Some(u), Some(h)) = (
            ids.get(&(unit.as_str(), "")),
            ids.get(&(header.as_str(), "")),
        ) {
            wire.insert([*u as i64, *h as i64, crate::graph::wire::EDGE_IMPORT, 3]);
        }
    }
    Ok(())
}

/// Each JSON compile database's response files, as the core's expansion
/// names them (readable, missing, cyclic or depth-limited): the resolve
/// key's inputs (compdb_find::facts). Only the databases that hold an
/// `@` byte are asked about.
pub fn responses(
    root: &Path,
    files: &BTreeSet<String>,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut body =
        json!({ "files": files, "c": request::databases(root, files, &BTreeMap::new()) });
    let reply = complete(&mut body, root)?;
    let rows: Vec<(String, Vec<String>)> = judged::table(&reply, "responses")?;
    Ok(rows.into_iter().collect())
}

/// Each walked tsconfig's extends chain as the core walks it: every
/// config the chain reaches, the tsconfig first (a broken chain: what it
/// reached before the break) — the resolve key's input (keys.rs). No
/// tsconfig asked, no request.
pub fn ts_reached(
    root: &Path,
    chains: &BTreeSet<&String>,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    if chains.is_empty() {
        return Ok(BTreeMap::new());
    }
    let facts: Vec<Value> = chains.iter().map(|c| facts::fact(root, 0, c, "")).collect();
    let mut body = json!({ "ts": { "chains": chains, "facts": facts } });
    let rows: Vec<(String, Vec<String>)> = judged::table(&complete(&mut body, root)?, "tsReached")?;
    Ok(rows.into_iter().collect())
}

/// Ask until the core names no response file and no file-system fact the
/// request lacks.
fn complete(body: &mut Value, root: &Path) -> Result<Value, String> {
    loop {
        let reply = ask(body.clone())?;
        let wanted: Vec<String> = judged::table(&reply, "wanted")?;
        let facts: Vec<(i64, String, String)> = judged::table(&reply, "tsWanted")?;
        if wanted.is_empty() && facts.is_empty() {
            return Ok(reply);
        }
        request::answer_wanted(body, root, &wanted)?;
        facts::answer_facts(body, root, &facts)?;
    }
}

/// The process's link to the core, opened on first use and dropped on
/// any failure so the next request opens a fresh one.
static LINK: Mutex<Option<Link>> = Mutex::new(None);

fn ask(body: Value) -> Result<Value, String> {
    let mut held = LINK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if held.is_none() {
        let (link, _) = Link::open(crate::tables::core_flag())?;
        *held = Some(link);
    }
    let link = held.as_mut().expect("opened above");
    let reply = judged::ask(link, CAP, SINCE, KIND, body);
    if reply.is_err() {
        *held = None;
    }
    let reply = reply.map_err(|e| format!("resolve/1: {e}"))?;
    judged::degraded(&reply).map_err(|e| format!("resolve/1: {e}"))?;
    Ok(reply)
}

/// One reply row as an outcome, a section's slug from its `sections`
/// row; a row of another shape, a code out of its table or a slug that
/// belongs to no section row is wire skew, named.
fn outcome(
    (rung, kind, target, reason): Row,
    section: Option<Option<String>>,
) -> Result<Outcome, String> {
    if section.is_some() != (kind == 5) {
        return Err("resolve/1: wire skew: a section row and its slug".into());
    }
    match (kind, target) {
        (2, _) => Ok(Outcome::External { rung }),
        (3, _) => Reason::from_code(reason)
            .map(Outcome::Unresolved)
            .ok_or_else(|| "resolve/1: wire skew: reason code".into()),
        (_, Some(path)) => targeted(kind, path, rung, section.flatten())
            .ok_or_else(|| "resolve/1: wire skew: outcome code".into()),
        (0 | 1 | 4..=6, None) => Err("resolve/1: wire skew: no target".into()),
        _ => Err("resolve/1: wire skew: outcome code".into()),
    }
}

/// A target-bearing outcome code's outcome: a file, a package directory,
/// a file through a re-export surface, a section, an inert file.
fn targeted(kind: i64, path: String, rung: Rung, slug: Option<String>) -> Option<Outcome> {
    Some(match kind {
        0 => Outcome::Resolved { path, rung },
        1 => Outcome::ResolvedPackage { dir: path, rung },
        4 => Outcome::ResolvedVia { path, rung },
        5 => Outcome::ResolvedSection { path, slug, rung },
        6 => Outcome::ResolvedInert { path, rung },
        _ => return None,
    })
}

/// The configuration readers' differential gate (tests subrepo): the
/// frozen 92e728b1 readers against the core's, through `inspect`.
#[cfg(test)]
#[path = "../../../tests/unit/graph/resolve/mod.rs"]
mod tests;
