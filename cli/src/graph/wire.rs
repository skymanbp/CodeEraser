//! Phase-2 resolver bridge (design §3/§4 wiring): cached sites →
//! ladder dispatch (one batch: the core holds four languages' rungs
//! since plan v2.33 wave W2a) → edge rows per site. Only in-corpus resolutions become
//! rows — External and Unresolved sites stay ledger-visible as sites
//! without edges (the eval instrument re-runs the ladder itself, so
//! refusal reasons need no storage). A ResolvedPackage lands ONE row
//! against the package node (dir, "") — the design's "fan out to
//! every member file" is the judgment layer's expansion (2g), which
//! also owns the //go:build exclusion flags: a collapsed fan-out
//! could not be undone, a package node can be expanded. The storage
//! codes below are frozen positions like the store table's kinds — renaming or
//! reordering is a GRAPH_REV bump.

use super::ladder::{self, Outcome, Scope, Site};
use super::owed::{Owed, Resolved};
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
/// The unused reference definition's edge (H1 slice 16, 2.29.0):
/// resolves, travels, and the CORE excludes it from liveness beside
/// the asset kind — user decision D3 as a wire fact instead of a
/// borrowed External outcome.
pub const EDGE_REFDEF_UNUSED: i64 = 5;

/// Frozen granularity codes.
pub const GRAN_FILE: i64 = 0;
pub const GRAN_PACKAGE: i64 = 1;
pub const GRAN_SECTION: i64 = 2;

/// The resolver callback: dispatch the cached sites down their
/// language ladders in one batch and shape each outcome into edge rows
/// (a site with no language or no kind label has none). When the core
/// cannot answer resolve/1, the languages it holds are stored
/// unresolved and their files owed (owed.rs); the rest still resolve.
pub fn edges(sites: &[CachedSite], scope: &Scope) -> anyhow::Result<Resolved> {
    let at: Vec<Option<(Lang, Site)>> = sites
        .iter()
        .map(|site| {
            let lang = Lang::from_path(Path::new(&site.file))?;
            let kind = kind_label(site.kind)?;
            Some((
                lang,
                Site {
                    kind,
                    from: &site.file,
                    spec: &site.spec,
                    line: usize::try_from(site.line).unwrap_or(1),
                },
            ))
        })
        .collect();
    let batch: Vec<(Lang, &Site)> = at.iter().flatten().map(|(l, s)| (*l, s)).collect();
    let (outcomes, owed) = match ladder::resolve_all(&batch, scope) {
        Ok(all) => (all.into_iter().map(Some).collect(), None),
        Err(reason) => without_core(&batch, scope, reason)?,
    };
    let mut outcomes = outcomes.into_iter();
    let rows = at
        .iter()
        .map(|found| match found {
            Some((_, site)) => outcomes
                .next()
                .expect("one outcome per site")
                .map_or_else(Vec::new, |o| rows(site.kind, o)),
            None => Vec::new(),
        })
        .collect();
    Ok(Resolved { rows, owed })
}

/// The batch with no core to answer: this side's rungs answer their
/// languages, the core's languages are none, their files owed.
type Partial = (Vec<Option<Outcome>>, Option<Owed>);

fn without_core(batch: &[(Lang, &Site)], scope: &Scope, reason: String) -> anyhow::Result<Partial> {
    let held = |l: Lang| super::resolve::in_core(l);
    let here: Vec<(Lang, &Site)> = batch.iter().filter(|(l, _)| !held(*l)).copied().collect();
    let mut answered = ladder::resolve_all(&here, scope)
        .map_err(anyhow::Error::msg)?
        .into_iter();
    let files = batch
        .iter()
        .filter(|(l, _)| held(*l))
        .map(|(_, s)| s.from.to_string())
        .collect();
    let outcomes = batch
        .iter()
        .map(|(l, _)| if held(*l) { None } else { answered.next() })
        .collect();
    Ok((outcomes, Some(Owed { reason, files })))
}

/// In-corpus outcomes only. dst_unit "" is the file or package node;
/// a slugless section outcome is the file-level anchor degrade.
fn rows(kind: &str, outcome: Outcome) -> Vec<EdgeRow> {
    // the inert arm first (H1 slice 16): kind is FORCED — an unused
    // definition's target identity rides, its reference-ness does not
    if let Outcome::ResolvedInert { path, rung } = outcome {
        return vec![EdgeRow {
            kind: EDGE_REFDEF_UNUSED,
            dst_path: path,
            dst_unit: String::new(),
            rung: i64::from(rung),
            granularity: GRAN_FILE,
            via_reexport: 0,
        }];
    }
    let (dst_path, dst_unit, rung, granularity, via) = match outcome {
        Outcome::Resolved { path, rung } => (path, String::new(), rung, GRAN_FILE, 0),
        // §4 R5 amendment: same file-level edge, the hop recorded
        Outcome::ResolvedVia { path, rung } => (path, String::new(), rung, GRAN_FILE, 1),
        Outcome::ResolvedPackage { dir, rung } => (dir, String::new(), rung, GRAN_PACKAGE, 0),
        Outcome::ResolvedSection {
            path,
            slug: Some(slug),
            rung,
        } => (path, slug, rung, GRAN_SECTION, 0),
        Outcome::ResolvedSection {
            path,
            slug: None,
            rung,
        } => (path, String::new(), rung, GRAN_FILE, 0),
        // the early arm above consumed this variant; the compiler
        // still wants it named here (the dispatch.rs exhaustiveness
        // posture)
        Outcome::ResolvedInert { .. } => unreachable!("handled by the inert arm above"),
        Outcome::External { .. } | Outcome::Unresolved(_) => return Vec::new(),
    };
    vec![EdgeRow {
        kind: edge_kind(kind, &dst_path),
        dst_path,
        dst_unit,
        rung: i64::from(rung),
        granularity,
        via_reexport: via,
    }]
}

/// Code kinds import; the doc kinds — Markdown's link family and
/// HTML's page references (`href`, a form's `action`) — split
/// doc_link / doc_ref by target (a page, else a code file); an image,
/// a `src` / `srcset` resource and an asset `<link>` are always
/// assets (plan v2.30 step 5, booklet §8).
fn edge_kind(kind: &str, dst: &str) -> i64 {
    match kind {
        "image" | "src" | "srcset" | "link_asset" => EDGE_ASSET,
        "link" | "ref_link" | "ref_def" | "url" | "href" | "action" => {
            if doc_page(dst) {
                EDGE_DOC_LINK
            } else {
                EDGE_DOC_REF
            }
        }
        _ => EDGE_IMPORT,
    }
}

/// A page — a Markdown or HTML document — as an edge target, the
/// doc→doc kind (`.markdown` labels like `.md`: the walker's own
/// extension table answers). md::is_md_path stays the one answer for
/// Markdown itself, which anchor validation asks.
fn doc_page(dst: &str) -> bool {
    matches!(
        Lang::from_path(Path::new(dst)),
        Some(Lang::Markdown | Lang::Html)
    )
}
