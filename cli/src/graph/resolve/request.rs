//! One resolve/1 request (plan v2.33 W2-text, proto 9.0.0): what this
//! side read, as text — the walked paths, the sites' specifiers as the
//! detector produced them, the `[graph.search_roots]` table, the Lua
//! templates the walk read, each go.mod's, each R DESCRIPTION's and each
//! .cabal file's text,
//! every walked Java file's header as the walk read it (package, imports,
//! type declarations: ladder/java_header.rs),
//! the root `pyproject.toml` and every JSON compile database decoded (a library
//! read; the rules applied to them are the core's), each flags file's
//! text, and the facts that need the file system: the root's absolute
//! text, the databases clangd's probes find, each C-family file's
//! include list (read in the walk), and the response files the core
//! asked for (`wanted`).

use crate::graph::compdb_find::{self, Found};
use crate::graph::ladder::Site;
use crate::graph::store::kind_code;
use crate::scan::lang::Lang;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// What a request reads of the sweep besides its sites.
pub struct Input<'a> {
    pub files: &'a BTreeSet<String>,
    pub root: &'a Path,
    pub configs: &'a [String],
    pub search_roots: &'a BTreeMap<String, BTreeSet<String>>,
    pub lua: Vec<[&'a str; 2]>,
    pub includes: &'a BTreeMap<String, Vec<String>>,
    pub java: &'a BTreeMap<String, crate::graph::ladder::java_header::Header>,
}

/// The sweep's part of a request: everything but the sites. The walk's
/// configs of the names the core reads (the package's `resolve.configs`,
/// where a name opening with `*` is a basename suffix) go as text: go.mod
/// and a .cabal read as UTF-8 (one that is not is none), a DESCRIPTION
/// lossy (it may declare `Encoding: latin1`).
pub fn tree(input: &Input) -> Value {
    let wanted = &crate::tables::get().resolve.configs;
    let named = |name: &'static str| {
        input.configs.iter().filter(move |c| {
            let base = c.rsplit('/').next().unwrap_or(c);
            let hit = name
                .strip_prefix('*')
                .map_or(base == name, |suffix| base.ends_with(suffix));
            hit && wanted.contains(&name)
        })
    };
    json!({
        "files": input.files,
        "config": { "searchRoots": input.search_roots },
        "py": { "pyproject": pyproject(input.root) },
        "lua": { "templates": input.lua },
        "go": { "mods": texts(input.root, named("go.mod")) },
        "r": { "descriptions": descriptions(input.root, named("DESCRIPTION")) },
        "hs": { "cabals": texts(input.root, named("*.cabal").collect::<BTreeSet<_>>().into_iter()) },
        "java": { "headers": headers(input.java) },
        "c": databases(input.root, input.files, input.includes),
    })
}

/// Each file's text read as UTF-8, by its path, in the order given; one
/// that cannot be read is none.
pub fn texts<'a>(root: &Path, rels: impl Iterator<Item = &'a String>) -> Vec<(&'a String, String)> {
    rels.filter_map(|rel| Some((rel, std::fs::read_to_string(root.join(rel)).ok()?)))
        .collect()
}

/// Each readable DESCRIPTION's text, lossy, by its path.
pub fn descriptions<'a>(
    root: &Path,
    rels: impl Iterator<Item = &'a String>,
) -> Vec<(&'a String, String)> {
    rels.filter_map(|rel| {
        let bytes = std::fs::read(root.join(rel)).ok()?;
        Some((rel, String::from_utf8_lossy(&bytes).into_owned()))
    })
    .collect()
}

/// Each Java header the walk read, `[path, package, [import], [type]]`:
/// an import `[name, star, static, line]`, a type `[name, [super],
/// [member], first, last]`.
fn headers(java: &BTreeMap<String, crate::graph::ladder::java_header::Header>) -> Vec<Value> {
    fn ty(t: &crate::graph::ladder::java_header::TypeDecl) -> Value {
        let members: Vec<Value> = t.members.iter().map(ty).collect();
        json!([t.name, t.supers, members, t.lines.0, t.lines.1])
    }
    java.iter()
        .map(|(path, h)| {
            let imports: Vec<Value> = h
                .imports
                .iter()
                .map(|i| json!([i.name, i.star, i.is_static, i.line]))
                .collect();
            let types: Vec<Value> = h.types.iter().map(ty).collect();
            json!([path, h.package, imports, types])
        })
        .collect()
}

/// The root `pyproject.toml` decoded, null when it is absent or not TOML.
fn pyproject(root: &Path) -> Value {
    std::fs::read_to_string(root.join("pyproject.toml"))
        .ok()
        .and_then(|text| text.parse::<toml::Table>().ok())
        .and_then(|doc| serde_json::to_value(doc).ok())
        .unwrap_or(Value::Null)
}

/// The C object: the root's text, the databases found (each JSON one
/// decoded once, each flags file read), the include lists.
pub fn databases(
    root: &Path,
    files: &BTreeSet<String>,
    includes: &BTreeMap<String, Vec<String>>,
) -> Value {
    let found = compdb_find::found(root, files.iter());
    let (mut json, mut flags) = (BTreeMap::new(), BTreeMap::new());
    for Found { probe, rel, .. } in &found {
        let text = || std::fs::read_to_string(root.join(rel)).ok();
        if *probe == 2 {
            flags.entry(rel).or_insert_with(text);
        } else {
            json.entry(rel).or_insert_with(|| {
                text().and_then(|t| serde_json::from_str::<Vec<Value>>(&t).ok())
            });
        }
    }
    let dbs: Vec<(&str, usize, &str)> = found
        .iter()
        .map(|f| (f.dir.as_str(), f.probe, f.rel.as_str()))
        .collect();
    json!({
        "root": compdb_find::root_text(root),
        "dbs": dbs,
        "json": json.into_iter().collect::<Vec<_>>(),
        "flags": flags.into_iter().collect::<Vec<_>>(),
        "responses": [],
        "includes": includes.iter().filter(|(f, _)| files.contains(*f)).collect::<Vec<_>>(),
    })
}

/// The sites, each `[lang, kind, from, spec]`, `from` an index into the
/// walked files then the sites' own files the walk did not hold (the
/// `origins`, returned in order); a Java site adds its line (the Java
/// rungs read the header's import on it and the types enclosing it).
pub fn sites(
    files: &BTreeSet<String>,
    sites: &[(Lang, &Site)],
) -> Result<(Value, Vec<String>), String> {
    let origins: BTreeSet<&str> = sites
        .iter()
        .map(|(_, s)| s.from)
        .filter(|f| !files.contains(*f))
        .collect();
    let index: BTreeMap<&str, usize> = files
        .iter()
        .map(String::as_str)
        .chain(origins.iter().copied())
        .zip(0..)
        .collect();
    let rows = sites
        .iter()
        .map(|(lang, s)| {
            let kind = kind_code(s.kind).map_err(|e| e.to_string())?;
            let mut row = vec![
                json!(*lang as i64),
                json!(kind),
                json!(index[s.from]),
                json!(s.spec),
            ];
            if *lang == Lang::Java {
                row.push(json!(s.line));
            }
            Ok(Value::Array(row))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok((
        Value::Array(rows),
        origins.into_iter().map(str::to_string).collect(),
    ))
}

/// The response files the core named and the request did not carry,
/// read and added (lossy UTF-8, null when unreadable); an empty list
/// means the request is complete. A name asked for twice is wire skew.
pub fn answer_wanted(body: &mut Value, root: &Path, wanted: &[String]) -> Result<(), String> {
    let carried = body["c"]["responses"]
        .as_array_mut()
        .ok_or("resolve/1: request has no response table")?;
    for rel in wanted {
        if carried.iter().any(|r| r[0] == *rel) {
            return Err(format!("resolve/1: wire skew: {rel} asked for twice"));
        }
        let text = std::fs::read(root.join(rel))
            .ok()
            .map(|b| String::from_utf8_lossy(&b).into_owned());
        carried.push(json!([rel, text]));
    }
    Ok(())
}
