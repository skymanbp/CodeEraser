//! The `Protocol` bit's name tables (sealed criterion §3.2, the NAME ×
//! language half of name.rs): the framework names a loader spells for
//! the author — Python unittest/xunit/pluggy/Django/reflection
//! prefixes, TS filename × export-name conventions, Haskell autogen
//! modules and hspec, the C-family entries a linker, a JVM or a
//! language runtime looks up by name, the Java methods the platform
//! calls, the Lua metamethods and host callbacks, the R app entry
//! points — so a declaration carrying one is reached though no file
//! names it. Split from name.rs when plan v2.30 step 4's Lua and R
//! tables took it past 300 lines; the path half (test files, the
//! runner table) stays there. A leaf: name.rs reads it, never the
//! reverse. Computed at wire time and stored nowhere, like every row
//! of name.rs.

use crate::scan::lang::Lang;

/// The name tables are the core's since plan v2.32 step 2
/// (CE.Lang.Common.Protocol, read off `tables/1`): Python's
/// loader hooks and prefixes, the TypeScript file forms, Java's platform
/// methods, the C family's loader and runtime entries and prefixes,
/// Lua's metamethods and plugin entries, LÖVE's callbacks and R's host
/// entries, each with its sources beside it.
fn names() -> &'static crate::tables::Protocol {
    &crate::tables::get().protocol
}

/// Whether `lang`'s loader, runtime or host calls `name` for the
/// author in the file at `rel`: a name in the language's table, a name
/// behind one of its prefixes, or a row the file's own name or place
/// decides.
pub(super) fn protocol(lang: Lang, rel: &str, name: &str) -> bool {
    let (names, prefixes) = tables(lang);
    listed(names, name) || prefixes.iter().any(|p| starred(name, p, "")) || by_file(lang, rel, name)
}

/// A language's name table and name prefixes.
fn tables(lang: Lang) -> (&'static [&'static str], &'static [&'static str]) {
    let n = names();
    match lang {
        Lang::Python => (n.py_names, n.py_prefixes),
        Lang::C | Lang::Cpp => (n.c_names, n.c_prefixes),
        Lang::Java => (n.java_names, &[]),
        Lang::Lua => (n.lua_names, &[]),
        Lang::R => (n.r_names, &[]),
        _ => (&[], &[]),
    }
}

/// The rows a file's own name or place decides: the TS file forms,
/// Haskell's generated modules and hspec's `spec`, and LÖVE's callbacks
/// in the two files it fills its `love` table from.
fn by_file(lang: Lang, rel: &str, name: &str) -> bool {
    let base = rel.rsplit('/').next().unwrap_or(rel);
    match lang {
        Lang::TypeScript | Lang::Tsx => ts_protocol(rel, base, name),
        Lang::Haskell => {
            base.starts_with("Paths_")
                || base.starts_with("PackageInfo_")
                || (name == "spec" && starred(base, "", "Spec.hs"))
        }
        Lang::Lua => matches!(base, "main.lua" | "conf.lua") && listed(names().love_names, name),
        _ => false,
    }
}

/// Whether a name table holds `name` — the shape every name list here
/// takes, and the dead-code entry names too (graph/deadcode/flags.rs).
pub(crate) fn listed(table: &[&str], name: &str) -> bool {
    table.contains(&name)
}

/// `<prefix>*<suffix>` with `*` non-empty — the one pattern the name
/// tables and name.rs's path half both read, here on the leaf side.
pub(super) fn starred(base: &str, prefix: &str, suffix: &str) -> bool {
    base.len() > prefix.len() + suffix.len() && base.starts_with(prefix) && base.ends_with(suffix)
}

/// Filename form × export name: the stem rows, then the directory
/// rows (`pages/**`; Remix `routes/**` and `app/root.*`).
fn ts_protocol(rel: &str, base: &str, name: &str) -> bool {
    let stem = base.rsplit_once('.').map_or(base, |(stem, _)| stem);
    let by_stem = names()
        .ts_by_stem
        .iter()
        .any(|&(stems, names)| listed(stems, stem) && listed(names, name));
    let in_dir = |d: &str| rel.split('/').rev().skip(1).any(|c| c == d);
    by_stem
        || (in_dir("pages") && listed(names().ts_pages, name))
        || ((in_dir("routes") || (stem == "root" && in_dir("app")))
            && listed(names().ts_routes, name))
}
