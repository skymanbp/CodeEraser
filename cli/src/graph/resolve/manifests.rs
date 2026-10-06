//! What the manifests say that the core reads outside a sweep's sites
//! (resolve/1, split from mod.rs): the declared-target pass's targets
//! (deadcode/targets.rs) and the mounts table's package privacy
//! (mounts.rs).

use super::{complete, facts, request};
use crate::corelink::judged;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// What the manifests the declared-target pass found declare: each R
/// package's code by its root (the core's `packages`; a DESCRIPTION that
/// cannot be read or names no package is none), the cabals' walked
/// executable and test mains (`mains`; a .cabal that cannot be read
/// declares nothing) and the Cargo.toml files' walked crate roots
/// (`crates`; one that does not read declares none). No manifest, no
/// request.
pub struct Declared {
    pub packages: BTreeMap<String, BTreeSet<String>>,
    pub mains: BTreeSet<String>,
    pub crates: BTreeSet<String>,
}

/// The manifests the declared-target pass found, by kind.
pub struct Manifests<'m> {
    pub descriptions: &'m BTreeSet<String>,
    pub cabals: &'m BTreeSet<String>,
    pub cargo: &'m BTreeSet<String>,
}

pub fn declared(
    root: &Path,
    files: &BTreeSet<String>,
    found: &Manifests,
) -> Result<Declared, String> {
    let r = request::descriptions(root, found.descriptions.iter());
    let hs = request::texts(root, found.cabals.iter());
    if r.is_empty() && hs.is_empty() && found.cargo.is_empty() {
        return Ok(Declared {
            packages: BTreeMap::new(),
            mains: BTreeSet::new(),
            crates: BTreeSet::new(),
        });
    }
    let mut body = json!({
        "files": files,
        "r": { "descriptions": r },
        "hs": { "cabals": hs },
        "rs": { "manifests": found.cargo },
    });
    let reply = complete_toml(&mut body, root, found.cargo.iter())?;
    let rows: Vec<(String, BTreeSet<String>)> = judged::table(&reply, "packages")?;
    Ok(Declared {
        packages: rows.into_iter().collect(),
        mains: judged::table(&reply, "mains")?,
        crates: judged::table(&reply, "crates")?,
    })
}

/// The walked Haskell and Rust files their owning manifest keeps private
/// (the mounts table's bit 1): `hs` maps each Haskell file to the cabal
/// the directory scan found nearest (each read and sent once; a file
/// whose cabal cannot be read is not kept), `rs` each Rust file to its
/// nearest Cargo.toml (each read once, as a TOML fact). No owner, no
/// request.
pub fn private(
    root: &Path,
    files: &BTreeSet<String>,
    hs: &BTreeMap<String, String>,
    rs: &BTreeMap<String, String>,
) -> Result<BTreeSet<String>, String> {
    let cabals: BTreeSet<&String> = hs.values().collect();
    let texts = request::texts(root, cabals.into_iter());
    let read: BTreeSet<&String> = texts.iter().map(|(rel, _)| *rel).collect();
    let owners: Vec<(&String, &String)> = hs.iter().filter(|(_, c)| read.contains(c)).collect();
    if owners.is_empty() && rs.is_empty() {
        return Ok(BTreeSet::new());
    }
    let mut body = json!({
        "files": files,
        "hs": { "cabals": texts, "owners": owners },
        "rs": { "owners": rs.iter().collect::<Vec<_>>() },
    });
    let manifests: BTreeSet<&String> = rs.values().collect();
    judged::table(
        &complete_toml(&mut body, root, manifests.into_iter())?,
        "private",
    )
}

/// `complete`, the given Cargo.toml files' TOML facts carried up front.
fn complete_toml<'a>(
    body: &mut Value,
    root: &Path,
    manifests: impl Iterator<Item = &'a String>,
) -> Result<Value, String> {
    let rows: Vec<(i64, String, String)> =
        manifests.map(|m| (3, m.clone(), String::new())).collect();
    facts::answer_facts(body, root, &rows)?;
    complete(body, root)
}
