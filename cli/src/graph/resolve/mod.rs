//! The reference ladders the core holds (plan v2.33 wave W2a; design
//! booklet docs/reference/algorithm-track.md §6 row W2): Python, Lua,
//! Go and C / C++ resolve in `resolve/1`. This side reads the tree and
//! the configuration files, lowers every string to a segment id
//! (lower.rs), sends one request per sweep with only the sites that
//! need resolving, and maps each reply row back to the ladder's
//! `Outcome` — so the edge store, deadcode, `ce graph --sites` and the
//! precision documents read the answers they always read. The other
//! languages' ladders still run on this side during the track.
//!
//! The core is the one this process names (the global `--core`, then
//! CE_CORE_BIN, a sibling of this binary, PATH), held open across the
//! process's requests. A core that cannot answer is a named refusal:
//! there is no copy of the search on this side to fall back on.

mod facts;
mod intern;
mod lower;
mod tokens;

use super::ladder::{Outcome, Reason, Scope, Site};
use crate::corelink::{Link, judged};
use crate::scan::lang::Lang;
use facts::Facts;
use lower::{Input, Lowered};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Mutex;

/// The capability the core must offer, the request kind, the proto that
/// minted the family.
pub const CAP: &str = "resolve/1";
pub const KIND: &str = "resolve";
pub const SINCE: &str = "8.1.0";

/// Whether the core holds this language's ladder.
pub fn in_core(lang: Lang) -> bool {
    matches!(
        lang,
        Lang::Python | Lang::Go | Lang::C | Lang::Cpp | Lang::Lua
    )
}

/// The sites' outcomes, in order — every site a language `in_core`.
pub fn outcomes(sites: &[(Lang, &Site)], scope: &Scope) -> Result<Vec<Outcome>, String> {
    if sites.is_empty() {
        return Ok(Vec::new());
    }
    let facts = scope.memo.cached("resolve:facts", "", || Facts::of(scope));
    let input = Input {
        files: scope.files,
        includes: scope.includes,
        root: scope.root,
    };
    let lowered = lower::lower(&input, &facts, sites)?;
    let reply = ask(lowered.body.clone())?;
    consume(&reply, &lowered, sites.len())
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
    let facts = Facts::databases(root, files);
    if facts.c.chains.is_empty() {
        return Ok(());
    }
    let none = BTreeMap::new();
    let input = Input {
        files,
        includes: &none,
        root,
    };
    let lowered = lower::lower(&input, &facts, &[])?;
    let reply = ask(lowered.body.clone())?;
    consume(&reply, &lowered, 0)?;
    let arcs: Vec<[usize; 2]> = judged::table(&reply, "forced")?;
    for [u, h] in arcs {
        let (unit, header) = (name(&lowered.files, u)?, name(&lowered.files, h)?);
        if let (Some(u), Some(h)) = (
            ids.get(&(unit.as_str(), "")),
            ids.get(&(header.as_str(), "")),
        ) {
            wire.insert([*u as i64, *h as i64, crate::graph::wire::EDGE_IMPORT, 3]);
        }
    }
    Ok(())
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

/// The reply rows as outcomes; a row of another shape, a target out of
/// its table or a count that disagrees is wire skew, named.
fn consume(reply: &Value, lowered: &Lowered, sent: usize) -> Result<Vec<Outcome>, String> {
    let rows: Vec<[i64; 4]> = judged::table(reply, "results")?;
    if rows.len() != sent || judged::count(reply, "sites")? != sent {
        return Err("resolve/1: wire skew: one result per site sent".into());
    }
    rows.iter().map(|row| outcome(*row, lowered)).collect()
}

fn outcome([rung, kind, target, reason]: [i64; 4], lowered: &Lowered) -> Result<Outcome, String> {
    let rung = u8::try_from(rung).map_err(|_| "resolve/1: wire skew: rung".to_string())?;
    let at = usize::try_from(target).unwrap_or(usize::MAX);
    Ok(match kind {
        0 => Outcome::Resolved {
            path: name(&lowered.files, at)?,
            rung,
        },
        1 => Outcome::ResolvedPackage {
            dir: name(&lowered.dirs, at)?,
            rung,
        },
        2 => Outcome::External { rung },
        3 => Outcome::Unresolved(
            Reason::from_code(reason).ok_or("resolve/1: wire skew: reason code")?,
        ),
        _ => return Err("resolve/1: wire skew: outcome code".into()),
    })
}

fn name(table: &[String], at: usize) -> Result<String, String> {
    table
        .get(at)
        .cloned()
        .ok_or_else(|| "resolve/1: wire skew: target out of its table".to_string())
}
