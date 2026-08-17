//! Phase-2 resolver bridge (design §3/§4 wiring): one cached site →
//! ladder dispatch → edge rows. Only in-corpus resolutions become
//! rows — External and Unresolved sites stay ledger-visible as sites
//! without edges (the eval instrument re-runs the ladder itself, so
//! refusal reasons need no storage). A ResolvedPackage lands ONE row
//! against the package node (dir, "") — the design's "fan out to
//! every member file" is the judgment layer's expansion (2g), which
//! also owns the //go:build exclusion flags: a collapsed fan-out
//! could not be undone, a package node can be expanded. The storage
//! codes below are frozen positions like store::KINDS — renaming or
//! reordering is a GRAPH_REV bump.

use super::ladder::{self, Outcome, Scope, Site};
use super::store::{CachedSite, EdgeRow, kind_label};
use crate::scan::lang::Lang;
use std::path::Path;

/// Frozen edge-kind codes: code→code import, doc→doc link, doc→code
/// reference, image asset (excluded from deadcode by kind, design
/// §4 Markdown row), and the SYNTHETIC containment arc deadcode adds
/// from a package node to its member files (reaching a package
/// reaches what it holds — never stored, built at request time).
pub const EDGE_IMPORT: i64 = 0;
pub const EDGE_DOC_LINK: i64 = 1;
pub const EDGE_DOC_REF: i64 = 2;
pub const EDGE_ASSET: i64 = 3;
pub const EDGE_CONTAIN: i64 = 4;

/// Frozen granularity codes.
pub const GRAN_FILE: i64 = 0;
pub const GRAN_PACKAGE: i64 = 1;
pub const GRAN_SECTION: i64 = 2;

/// The resolver callback: dispatch one cached site down its language
/// ladder and shape the outcome into edge rows.
pub fn edges(site: &CachedSite, scope: &Scope) -> Vec<EdgeRow> {
    let Some(lang) = Lang::from_path(Path::new(&site.file)) else {
        return Vec::new();
    };
    let Some(kind) = kind_label(site.kind) else {
        return Vec::new();
    };
    let at = Site {
        kind,
        from: &site.file,
        spec: &site.spec,
        line: usize::try_from(site.line).unwrap_or(1),
    };
    let outcome = ladder::resolve(lang, &at, scope);
    rows(kind, outcome)
}

/// In-corpus outcomes only. dst_unit "" is the file or package node;
/// a slugless section outcome is the file-level anchor degrade.
fn rows(kind: &str, outcome: Outcome) -> Vec<EdgeRow> {
    let (dst_path, dst_unit, rung, granularity) = match outcome {
        Outcome::Resolved { path, rung } => (path, String::new(), rung, GRAN_FILE),
        Outcome::ResolvedPackage { dir, rung } => (dir, String::new(), rung, GRAN_PACKAGE),
        Outcome::ResolvedSection {
            path,
            slug: Some(slug),
            rung,
        } => (path, slug, rung, GRAN_SECTION),
        Outcome::ResolvedSection {
            path,
            slug: None,
            rung,
        } => (path, String::new(), rung, GRAN_FILE),
        Outcome::External { .. } | Outcome::Unresolved(_) => return Vec::new(),
    };
    vec![EdgeRow {
        kind: edge_kind(kind, &dst_path),
        dst_path,
        dst_unit,
        rung: i64::from(rung),
        granularity,
    }]
}

/// Code kinds import; the md kinds split doc_link / doc_ref by
/// target (the md::is_md_path throat, so `.markdown` labels like
/// `.md`), and an image is always an asset.
fn edge_kind(kind: &str, dst: &str) -> i64 {
    match kind {
        "image" => EDGE_ASSET,
        "link" | "ref_link" | "ref_def" | "url" => {
            if crate::graph::md::is_md_path(dst) {
                EDGE_DOC_LINK
            } else {
                EDGE_DOC_REF
            }
        }
        _ => EDGE_IMPORT,
    }
}
