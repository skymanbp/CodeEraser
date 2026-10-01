//! Per-language site-kind tables for the graph subsystem (design
//! brief §1/§4, git history: docs/reviews/2026-08-12-m5-2-graph-
//! design.md): which tree-sitter node kinds open a SITE and where the
//! specifier string lives. The tables are the core's since plan v2.32
//! step 2 — each language's `sites` list in CE.Lang.<Language>, read
//! off `tables/1` (crate::tables); this file keeps the shapes. Separate
//! from `scan/spec.rs`: its fn_kinds drives the M1 metrics and the
//! dogfood ratchet, a separate concern.
//!
//! Sites are resolution-FREE: detection needs only these tables, so
//! the sample universe freezes before any resolver exists (the
//! instrument-first spine — the resolver must never choose its own
//! precision denominator). Markdown has no grammar and scans in
//! graph/md.rs.

use crate::scan::lang::Lang;
use crate::tables::leak::leaked;

/// The calls that open a site — CallSite and its tables — live beside
/// the node-kind tables in spec/calls.rs, split out on the file-length
/// line (plan v2.30 step 5b); one door for both vocabularies.
mod calls;
pub use calls::{CallSite, calls, formals, protected};

/// How to pull the specifier string out of a matched node. Read
/// adjacently tagged (`{"form": "field", "arg": "source"}`, a unit
/// form without `arg`) — the tables/1 shape (VERSIONING 7.7.0).
#[derive(serde::Deserialize)]
#[serde(tag = "form", content = "arg", rename_all = "snake_case")]
pub enum Specifier {
    /// Text of `child_by_field_name(field)`, quotes trimmed — a
    /// node without the field is simply not a site (so TS
    /// `export … from "x"` matches and a plain export does not).
    Field(String),
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
    FieldIfStar(String),
    /// A fixed specifier the node carries as a keyword, not a field:
    /// Python's `from __future__ import …` names the module
    /// `__future__` by a token the grammar keeps anonymous (plan v2.17
    /// L round step 8, O27), and the reference is as real as any
    /// other module import — a stdlib module the ladder answers
    /// External.
    Literal(String),
    /// A Java import (plan v2.30 step 3): the declaration names its
    /// target by an UNFIELDED child — the first `scoped_identifier` or
    /// `identifier` — and says `.*` by an `asterisk` child. The spec
    /// runs from the anonymous `static` token when there is one to the
    /// end of the name (`static a.b.C.m`), so the ladder reads the last
    /// segment as a member and the spec stays a substring of its line.
    /// `star` picks the form the entry matches — the star entry listed
    /// first, as FieldIfStar's is.
    FirstNamed { star: bool },
    /// A Rust `use` (plan v2.30 step 5b): a group with no path before
    /// its brace — `use {a::b, c::d};` — opens one site per entry, each
    /// a complete path on its own line; any other argument shape is one
    /// site spelled by the argument's first line (graph/sites.rs).
    UseTargets,
    /// Text from the start of an optional `from` field to the end of
    /// the `to` field, never quote-trimmed: Haskell's `import "pkg" M`
    /// (PackageImports, plan v2.30 step 5b) keeps the package in front
    /// of the module, and the quotes are what tell the two apart.
    Spanned { from: String, to: String },
}

leaked! {
    /// (tree-sitter node kind, stable doc/wire label, specifier source).
    pub struct SiteKind {
        pub node: &'static str,
        pub label: &'static str,
        pub via: Specifier,
    }
}

/// The site vocabulary of one language, its calls' aside (`calls`).
/// Labels are frozen doc/wire identity — renaming one is a contract
/// change. Lua's sites are all calls; Java's `type_ref` and HTML's
/// (element, attribute) pairs are passes of their own
/// (graph/sites/java.rs, graph/sites/html.rs); Markdown scans line-wise
/// in graph/md.rs; a language the package does not judge has none.
pub fn sites(lang: Lang) -> &'static [SiteKind] {
    crate::tables::get()
        .sites
        .get(lang)
        .map_or(&[], Vec::as_slice)
}
