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
    /// Text of `child_by_field_name(field)`, quotes trimmed.
    Field(&'static str),
    /// Like Field, but the node is only a site when the field exists
    /// (TS `export … from "x"` vs a plain export).
    FieldIfPresent(&'static str),
    /// Text of the `name` field, but only when the node has NO
    /// `body` field — `mod foo;` is a reference to another file,
    /// `mod foo { … }` defines it inline and references nothing.
    NameIfNoBody,
    /// One site per import target child (Python `import a.b, c as d`:
    /// each dotted_name / aliased_import name is its own site).
    EachImportTarget,
}

/// (tree-sitter node kind, stable doc/wire label, specifier source).
pub struct SiteKind {
    pub node: &'static str,
    pub label: &'static str,
    pub via: Specifier,
}

/// The anon `import` keyword token and foreign_import's inner token
/// share this kind name (the 3k D11 collision class, AST-probed
/// 2026-08-14) but carry no `module` field, so the Field specifier
/// drops them by construction. Housed outside sites() for the E01
/// fn-length line — a lone table, not the 3k statics trap (that
/// extraction PAIRED two isomorphic tables into a new T2 block).
const HASKELL: [SiteKind; 1] = [SiteKind {
    node: "import",
    label: "import",
    via: Specifier::Field("module"),
}];

/// The site vocabulary of one language. Labels are frozen doc/wire
/// identity — renaming one is a contract change.
pub fn sites(lang: Lang) -> &'static [SiteKind] {
    match lang {
        Lang::Python => &[
            SiteKind {
                node: "import_statement",
                label: "import",
                via: Specifier::EachImportTarget,
            },
            SiteKind {
                node: "import_from_statement",
                label: "import_from",
                via: Specifier::Field("module_name"),
            },
        ],
        Lang::TypeScript | Lang::Tsx => &[
            SiteKind {
                node: "import_statement",
                label: "import",
                via: Specifier::Field("source"),
            },
            SiteKind {
                node: "export_statement",
                label: "export_from",
                via: Specifier::FieldIfPresent("source"),
            },
        ],
        Lang::Rust => &[
            SiteKind {
                node: "use_declaration",
                label: "use",
                via: Specifier::Field("argument"),
            },
            SiteKind {
                node: "mod_item",
                label: "mod_decl",
                via: Specifier::NameIfNoBody,
            },
        ],
        Lang::Go => &[SiteKind {
            node: "import_spec",
            label: "import",
            via: Specifier::Field("path"),
        }],
        Lang::Haskell => &HASKELL,
        // Markdown scans line-wise in graph/md.rs (no grammar); the
        // sentinel is never walked, and the scan-only arm (plan
        // v2.5) is never indexed — no site vocabulary either way.
        _ => &[],
    }
}
