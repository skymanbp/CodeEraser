//! The calls that open a reference site (plan v2.30 step 4; the reader
//! is graph/sites/call.rs): Lua loads a module by `require` and runs a
//! file by `dofile` / `loadfile`, R sources a file by `source` and
//! attaches a package by the `library` family — neither language has an
//! import statement. Housed beside the node-kind tables of graph/spec.rs
//! and split out of it on the file-length line (plan v2.30 step 5b).

use crate::scan::lang::Lang;
use crate::tables::leak::leaked;

leaked! {
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
/// where the callee reads one unevaluated. The rows are the core's
/// since plan v2.32 step 2 (each language's `calls`, its `protected`
/// wrappers and R's formals, in CE.Lang.<Language> and CE.Lang.Common),
/// read off `tables/1` (crate::tables).
    pub struct CallSite {
        pub label: &'static str,
        pub callees: &'static [&'static str],
        pub package: &'static str,
        pub unquoted: Option<&'static str>,
    }
}

/// The calls that open a site in one language (CallSite).
pub fn calls(lang: Lang) -> &'static [CallSite] {
    crate::tables::get()
        .calls
        .calls
        .get(lang)
        .map_or(&[], Vec::as_slice)
}

/// The protected-call wrappers of one language, each with the count of
/// arguments before the protected function's own: Lua's `pcall` runs
/// its first argument, `xpcall` its first after a message handler.
pub fn protected(lang: Lang) -> &'static [(String, usize)] {
    crate::tables::get()
        .calls
        .protected
        .get(lang)
        .map_or(&[], Vec::as_slice)
}

/// The formals before `...` of one R callee, in order; a Lua callee,
/// and any name the table lacks, has none. The first is the target.
pub fn formals(callee: &str) -> &'static [&'static str] {
    crate::tables::get()
        .calls
        .r_formals
        .iter()
        .find_map(|&(name, formals)| (name == callee).then_some(formals))
        .unwrap_or(&[])
}
