//! Per-language site-kind tables for the graph subsystem (design
//! brief §1/§4, git history: docs/reviews/2026-08-12-m5-2-graph-
//! design.md): which tree-sitter node kinds open a SITE and where the
//! specifier string lives. Tuple-table shape per fourclass/kinds.rs;
//! `scan/spec.rs` must not grow these — its fn_kinds drives the M1
//! metrics and the dogfood ratchet, a separate concern.
//!
//! Sites are resolution-FREE: detection needs only these tables, so
//! the sample universe freezes before any resolver exists (the
//! instrument-first spine — the resolver must never choose its own
//! precision denominator). Markdown has no grammar and scans in
//! graph/md.rs.

use crate::scan::lang::Lang;

/// How to pull the specifier string out of a matched node.
pub enum Specifier {
    /// Text of `child_by_field_name(field)`, quotes trimmed — a
    /// node without the field is simply not a site (so TS
    /// `export … from "x"` matches and a plain export does not).
    Field(&'static str),
    /// Text of the `name` field, but only when the node has NO
    /// `body` field — `mod foo;` is a reference to another file,
    /// `mod foo { … }` defines it inline and references nothing.
    NameIfNoBody,
    /// One site per import target child (Python `import a.b, c as d`:
    /// each dotted_name / aliased_import name is its own site).
    EachImportTarget,
    /// `Field`, but only for a STAR export — a `*` token or a
    /// `namespace_export` child beside the source (`export * from`,
    /// `export * as ns from`); an `export_clause` form is not one.
    /// Listed ahead of the plain `Field` entry of the same node kind:
    /// the walker takes the first entry that emits, so one statement
    /// opens one site under the more specific label.
    FieldIfStar(&'static str),
    /// A fixed specifier the node carries as a keyword, not a field:
    /// Python's `from __future__ import …` names the module
    /// `__future__` by a token the grammar keeps anonymous (plan v2.17
    /// L round step 8, O27), and the reference is as real as any
    /// other module import — a stdlib module the ladder answers
    /// External.
    Literal(&'static str),
    /// A Java import (plan v2.30 step 3): the declaration names its
    /// target by an UNFIELDED child — the first `scoped_identifier` or
    /// `identifier` — and says `.*` by an `asterisk` child. The spec
    /// runs from the anonymous `static` token when there is one to the
    /// end of the name (`static a.b.C.m`), so the ladder reads the last
    /// segment as a member and the spec stays a substring of its line.
    /// `star` picks the form the entry matches — the star entry listed
    /// first, as FieldIfStar's is.
    FirstNamed { star: bool },
}

/// (tree-sitter node kind, stable doc/wire label, specifier source).
pub struct SiteKind {
    pub node: &'static str,
    pub label: &'static str,
    pub via: Specifier,
}

/// One table row, in the struct's field order. A macro, not a const
/// fn: a row must stay a constant expression for a match arm's `&[…]`
/// to be promoted to `'static`, and a const fn call never is.
macro_rules! site {
    ($node:expr, $label:expr, $via:expr $(,)?) => {
        SiteKind {
            node: $node,
            label: $label,
            via: $via,
        }
    };
}

/// The anon `import` keyword token and foreign_import's inner token
/// share this kind name (the 3k D11 collision class, AST-probed
/// 2026-08-14) but carry no `module` field, so the Field specifier
/// drops them by construction. Housed outside sites() for the E01
/// fn-length line — a lone table, not the 3k statics trap (that
/// extraction PAIRED two isomorphic tables into a new T2 block).
const HASKELL: [SiteKind; 1] = [site!("import", "import", Specifier::Field("module"))];

/// `from __future__ import …` is its own grammar node without a
/// module field; the site names the literal module (plan v2.17 L
/// round step 8, O27). A named const, like the Haskell table: the
/// two per-language arms it would otherwise lengthen rhymed under
/// the clone gate.
const FUTURE_IMPORT: SiteKind = site!(
    "future_import_statement",
    "import_from",
    Specifier::Literal("__future__")
);

/// `import fs = require("./b")` hangs `source` off this child clause,
/// not off the statement (step 8, O26); `import X = A.B.C` is an
/// `import_alias` naming a namespace, not a module — the file it may
/// reach is already referenced by the `import * as A` that bound
/// `A`, so it opens no site by construction.
const IMPORT_REQUIRE: SiteKind = site!(
    "import_require_clause",
    "import",
    Specifier::Field("source")
);

/// The C family's one site (plan v2.30 step 2): `#include`, whose
/// `path` field is a `string_literal` or a `system_lib_string` (both
/// probed). The quoted form loses its quotes like every string
/// specifier and the system form keeps its angle brackets — the
/// delimiter is the search order the ladder reads (ladder/c.rs). A
/// macro-spelled include (`#include HEADER`) carries an identifier in
/// the field and is a site whose spec no rung can answer — the
/// honest ledger row, never a guess.
const INCLUDE: [SiteKind; 1] = [site!(
    "preproc_include",
    "include",
    Specifier::Field("path")
)];

/// Java's import declaration (plan v2.30 step 3): the star form under
/// its own label, as TS's `export *` is. Java's other site kind,
/// `type_ref`, is no table row: which names count excludes the ones the
/// file itself declares — a file-level fact — so graph/sites/java.rs
/// runs a pass of its own.
const JAVA: [SiteKind; 2] = [
    site!(
        "import_declaration",
        "import_star",
        Specifier::FirstNamed { star: true }
    ),
    site!(
        "import_declaration",
        "import",
        Specifier::FirstNamed { star: false }
    ),
];

/// R's one site that is no call: `pkg::name` names its package whatever
/// the right side selects (plan v2.30 step 4) — the `library` site a
/// qualified call opens in place of a call site of its own.
const R: [SiteKind; 1] = [site!(
    "namespace_operator",
    "library",
    Specifier::Field("lhs")
)];

/// A call naming its target by an argument (plan v2.30 step 4): Lua's
/// `require` and file loaders, R's `source` and `library` family —
/// neither language has an import statement. Its own table, not a
/// SiteKind row: which node is a call and where its callee hangs are
/// the grammar's facts (LangSpec::call_kinds / call_fields, the
/// spelling the recursion arcs read), so a row names only what varies
/// — the bare callee names, and the argument naming the target: the
/// one passed as `formal` (R matches names first; a Lua call passes
/// none) or else the first unnamed one. That argument must be a string
/// literal — or an identifier, where the callee reads one unevaluated:
/// `unquoted` names the argument that turns that off
/// (graph/sites/call.rs).
pub struct CallSite {
    pub label: &'static str,
    pub callees: &'static [&'static str],
    pub formal: &'static str,
    pub unquoted: Option<&'static str>,
}

/// Lua: a module by `require` (a dotted name, the package.path search),
/// a file by `dofile` / `loadfile` (a path).
const LUA_CALLS: [CallSite; 2] = [
    CallSite {
        label: "require",
        callees: &["require"],
        formal: "",
        unquoted: None,
    },
    CallSite {
        label: "load",
        callees: &["dofile", "loadfile"],
        formal: "",
        unquoted: None,
    },
];

/// R: a file by `source` (its first formal is `file`), a package by the
/// `library` family (`package`). `library` and `require` read a bare
/// name as the package unless the call passes `character.only`;
/// `requireNamespace` and `loadNamespace` evaluate the argument, so a
/// bare name there is a variable (R's own help pages, base `library`
/// and `ns-load`).
const R_CALLS: [CallSite; 3] = [
    CallSite {
        label: "source",
        callees: &["source", "sys.source"],
        formal: "file",
        unquoted: None,
    },
    CallSite {
        label: "library",
        callees: &["library", "require"],
        formal: "package",
        unquoted: Some("character.only"),
    },
    CallSite {
        label: "library",
        callees: &["requireNamespace", "loadNamespace"],
        formal: "package",
        unquoted: None,
    },
];

/// The calls that open a site in one language (CallSite).
pub fn calls(lang: Lang) -> &'static [CallSite] {
    match lang {
        Lang::Lua => &LUA_CALLS,
        Lang::R => &R_CALLS,
        _ => &[],
    }
}

/// The site vocabulary of one language, its calls' aside (`calls`).
/// Labels are frozen doc/wire identity — renaming one is a contract
/// change.
pub fn sites(lang: Lang) -> &'static [SiteKind] {
    match lang {
        Lang::Python => &[
            site!("import_statement", "import", Specifier::EachImportTarget),
            site!(
                "import_from_statement",
                "import_from",
                Specifier::Field("module_name")
            ),
            FUTURE_IMPORT,
        ],
        Lang::TypeScript | Lang::Tsx => &[
            site!("import_statement", "import", Specifier::Field("source")),
            IMPORT_REQUIRE,
            // the star form first (plan v2.17 L round piece (5)): a
            // re-export target is the mounts table's bit 0
            site!(
                "export_statement",
                "export_star",
                Specifier::FieldIfStar("source")
            ),
            site!(
                "export_statement",
                "export_from",
                Specifier::Field("source")
            ),
        ],
        Lang::Rust => &[
            site!("use_declaration", "use", Specifier::Field("argument")),
            site!("mod_item", "mod_decl", Specifier::NameIfNoBody),
        ],
        Lang::Go => &[site!("import_spec", "import", Specifier::Field("path"))],
        Lang::Haskell => &HASKELL,
        Lang::C | Lang::Cpp => &INCLUDE,
        Lang::Java => &JAVA,
        // every Lua site is a call (`calls`)
        Lang::Lua => &[],
        Lang::R => &R,
        // Markdown scans line-wise in graph/md.rs (no grammar); the
        // sentinel is never walked, and the scan-only arm (plan
        // v2.5) is never indexed — no site vocabulary either way.
        _ => &[],
    }
}
