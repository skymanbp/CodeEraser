//! The calls that open a reference site (plan v2.30 step 4; the reader
//! is graph/sites/call.rs): Lua loads a module by `require` and runs a
//! file by `dofile` / `loadfile`, R sources a file by `source` and
//! attaches a package by the `library` family — neither language has an
//! import statement. Housed beside the node-kind tables of graph/spec.rs
//! and split out of it on the file-length line (plan v2.30 step 5b).

use crate::scan::lang::Lang;

/// A call naming its target by an argument. Its own table, not a
/// SiteKind row: which node is a call and where its callee hangs are
/// the grammar's facts (LangSpec::call_kinds / call_fields, the
/// spelling the recursion arcs read), so a row names only what varies
/// — the bare callee names; the one package a callee may be qualified
/// by (`base::source("x.R")`, plan v2.30 step 5b — Lua spells no
/// qualified call); and `unquoted`, the argument that turns off reading
/// a bare identifier as the target. The argument naming the target is
/// the callee's first formal (`formals`): R matches a name first —
/// exactly, or as a prefix no other formal shares (graph/sites/call.rs)
/// — and a Lua call passes none, so the first unnamed argument is the
/// target. That argument must be a string literal, or an identifier
/// where the callee reads one unevaluated.
#[derive(serde::Serialize)]
pub struct CallSite {
    pub label: &'static str,
    pub callees: &'static [&'static str],
    pub package: &'static str,
    pub unquoted: Option<&'static str>,
}

/// Lua: a module by `require` (a dotted name, the package.path search),
/// a file by `dofile` / `loadfile` (a path) — called directly or under
/// protection (LUA_PROTECTED).
const LUA_CALLS: [CallSite; 2] = [
    CallSite {
        label: "require",
        callees: &["require"],
        package: "",
        unquoted: None,
    },
    CallSite {
        label: "load",
        callees: &["dofile", "loadfile"],
        package: "",
        unquoted: None,
    },
];

/// Lua calls a loader under protection too: `pcall(require, "x")` runs
/// `require("x")` and hands back its error instead of raising it — the
/// idiom for an optional module. The wrapper's first argument is the
/// protected function, spelled as a bare name, and that function's own
/// arguments follow the `leading` ones: the function for `pcall`, the
/// function and the message handler for `xpcall` (Lua 5.2 on, and
/// LuaJIT, pass the rest on; 5.1's `xpcall` takes none). A function
/// named any other way (`pcall(m.require, "x")`) is no row's callee.
const LUA_PROTECTED: [(&str, usize); 2] = [("pcall", 1), ("xpcall", 2)];

/// R: a file by `source` (its first formal is `file`), a package by the
/// `library` family (`package`), every callee living in `base`.
/// `library` and `require` read a bare name as the package unless the
/// call passes `character.only`; `requireNamespace` and `loadNamespace`
/// evaluate the argument, so a bare name there is a variable (R's own
/// help pages, base `library` and `ns-load`).
const R_CALLS: [CallSite; 3] = [
    CallSite {
        label: "source",
        callees: &["source", "sys.source"],
        package: "base",
        unquoted: None,
    },
    CallSite {
        label: "library",
        callees: &["library", "require"],
        package: "base",
        unquoted: Some("character.only"),
    },
    CallSite {
        label: "library",
        callees: &["requireNamespace", "loadNamespace"],
        package: "base",
        unquoted: None,
    },
];

/// The formals before `...` of each R callee, one line per callee as
/// `name: formals`, read off R-devel 4.6.0's help pages (base `source`,
/// `sys.source`, `library`, `ns-load`; 2026-09-26). The table stops at
/// the dots: a formal after them matches only exactly (R Language
/// Definition §4.3.2) and is never a row's target. The first formal is
/// the target. One literal, not a tuple table — rows of one shape rhyme
/// under the clone gate.
pub(crate) const R_FORMALS: &str = "\
source: file local echo print.eval exprs spaced verbose prompt.echo max.deparse.length width.cutoff deparseCtrl chdir catch.aborts encoding continue.echo skip.echo keep.source
sys.source: file envir chdir keep.source keep.parse.data toplevel.env
library: package help pos lib.loc character.only logical.return warn.conflicts quietly verbose mask.ok exclude include.only attach.required
require: package lib.loc quietly warn.conflicts character.only mask.ok exclude include.only attach.required
requireNamespace: package
loadNamespace: package lib.loc keep.source partial versionCheck keep.parse.data
";

/// The calls that open a site in one language (CallSite).
pub fn calls(lang: Lang) -> &'static [CallSite] {
    match lang {
        Lang::Lua => &LUA_CALLS,
        Lang::R => &R_CALLS,
        _ => &[],
    }
}

/// The protected-call wrappers of one language, each with the count of
/// arguments before the protected function's own (LUA_PROTECTED).
pub fn protected(lang: Lang) -> &'static [(&'static str, usize)] {
    match lang {
        Lang::Lua => &LUA_PROTECTED,
        _ => &[],
    }
}

/// The formals of one callee (R_FORMALS), space-separated; a Lua callee,
/// and any name the table lacks, has none.
pub fn formals(callee: &str) -> &'static str {
    R_FORMALS
        .lines()
        .find_map(|row| row.strip_prefix(callee)?.strip_prefix(": "))
        .unwrap_or("")
}
