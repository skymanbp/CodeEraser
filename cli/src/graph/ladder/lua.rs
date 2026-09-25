//! Lua rungs (plan v2.30 step 4; design booklet §8 row Lua). A Lua
//! module is whatever `package.searchers` finds (Lua 5.4 manual §6.3):
//! `require` first answers a name `package.loaded` already holds — the
//! standard libraries, LuaJIT's built-ins — and otherwise asks each
//! template of `package.path` in turn, `?` standing for the name with
//! its dots turned into slashes. The sites (graph/sites/call.rs) walk:
//!   R1 `require "a.b"`: `a/b` in every search directory — the tree
//!      root, `src` and `lua` (the luarocks and Neovim layouts) and the
//!      declared `[graph.search_roots] lua`, each tried as `a/b.lua`
//!      then `a/b/init.lua` (the standard order), and every template
//!      the tree's own files assign to `package.path` as it is written
//!      (lua_path.rs, Scope::lua). Two directories answering two
//!      different files is ambiguous_root: which template a run tries
//!      first is the order its path was built in, no fact of the text;
//!   R2 `dofile` / `loadfile` (label `load`): the path beside the
//!      loading file, then under the tree root — a path is relative to
//!      the working directory, the script's own or the project root by
//!      convention; the first hit;
//!   R3 External: a standard library or built-in name (STDLIB) — it
//!      never reaches the searchers, whatever files the tree holds.
//! Anything else is out_of_scope: a module a rock or a C library
//! provides, a directory a run adds by computing it
//! (`string.format("%s/?.lua", dir)`) — misses, never a guessed edge.

use super::{Outcome, Reason, Scope, Site, paths};
use crate::graph::roots;
use crate::mention::conv::protocol::listed;
use std::collections::{BTreeMap, BTreeSet};

/// The names `package.loaded` holds before any searcher runs: Lua
/// 5.1–5.4's standard libraries (manual §6; `bit32` is 5.2's, `utf8`
/// 5.3's) and LuaJIT's built-in extension modules
/// (luajit.org/extensions.html).
const STDLIB: &str = "string table math io os coroutine debug package bit32 utf8 \
                      ffi bit jit jit.util jit.profile table.new table.clear string.buffer";

/// The search directories every tree has: its root and the two layouts
/// booklet §8 names.
const DEFAULT_DIRS: [&str; 3] = ["", "src", "lua"];

/// The two suffixes the standard path tries a directory with, in order.
const STANDARD: [&str; 2] = [".lua", "/init.lua"];

pub fn resolve(site: &Site, scope: &Scope) -> Outcome {
    let (from, spec) = (site.from, site.spec);
    match site.kind {
        "require" if listed(STDLIB, spec) => Outcome::External { rung: 3 },
        "require" => module(spec, scope),
        "load" => loaded(from, spec, scope),
        _ => Outcome::Unresolved(Reason::Unsupported),
    }
}

/// R1: the module's path in every search directory; one distinct file
/// resolves, two refuse. A name with an empty segment (`a..b`, a
/// leading or trailing dot or slash) names no file.
fn module(spec: &str, scope: &Scope) -> Outcome {
    let name = spec.replace('.', "/");
    if name.split('/').any(str::is_empty) {
        return Outcome::Unresolved(Reason::OutOfScope);
    }
    let dirs = scope.memo.cached("lua-dirs", "", || searched(scope));
    let hits: BTreeSet<String> = dirs
        .iter()
        .filter_map(|(dir, suffixes)| {
            suffixes
                .iter()
                .filter_map(|s| roots::join_rel(dir, &format!("{name}{s}")))
                .find(|p| scope.files.contains(p))
        })
        .collect();
    paths::one_of(hits, 1).unwrap_or(Outcome::Unresolved(Reason::OutOfScope))
}

/// Every search directory with the suffixes it is tried with: the
/// defaults and the declared roots take the standard two, a template
/// adds its own; `.lua` goes first wherever it is tried (a directory's
/// templates, like the standard path, list `?.lua` before
/// `?/init.lua`).
fn searched(scope: &Scope) -> BTreeMap<String, Vec<String>> {
    let declared = scope.search_roots.get("lua").into_iter().flatten();
    let standard = DEFAULT_DIRS
        .iter()
        .map(|d| d.to_string())
        .chain(declared.cloned())
        .flat_map(|dir| STANDARD.map(|s| (dir.clone(), s.to_string())));
    let written = scope.lua.iter().map(|t| (t.dir.clone(), t.suffix.clone()));
    let mut dirs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (dir, suffix) in standard.chain(written) {
        let suffixes = dirs.entry(dir).or_default();
        if !suffixes.contains(&suffix) {
            suffixes.push(suffix);
        }
    }
    for suffixes in dirs.values_mut() {
        suffixes.sort_by_key(|s| s != ".lua");
    }
    dirs
}

/// R2: the path beside the loading file, then under the tree root; the
/// first hit.
fn loaded(from: &str, spec: &str, scope: &Scope) -> Outcome {
    paths::beside_or_root(from, spec, scope)
        .map_or(Outcome::Unresolved(Reason::OutOfScope), |path| {
            Outcome::Resolved { path, rung: 2 }
        })
}
