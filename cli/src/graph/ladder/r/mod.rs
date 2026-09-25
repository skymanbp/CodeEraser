//! R rungs (plan v2.30 step 4; design booklet §8 row R). R has no
//! import statement: `source("x.R")` runs a file, and `library(pkg)`,
//! `requireNamespace("pkg")` and `pkg::name` reach a package
//! (graph/sites/call.rs). The sites walk:
//!   R1 `source`: the path beside the sourcing file, then under the
//!      tree root — a path is relative to R's working directory, the
//!      project root by the convention RStudio projects and `Rscript`
//!      runs from the root share — the first hit; then the declared
//!      `[graph.search_roots] r` directories, two of which answering two
//!      different files is ambiguous_root;
//!   R2 `library`: the directory of the in-scope DESCRIPTION whose
//!      `Package:` field names the package (description.rs) — a package
//!      is its directory (ResolvedPackage), its code the `R/` files
//!      under it, which is all its node reaches (nodes::contain reads
//!      deadcode/targets.rs); two such is ambiguous_workspace;
//!   R3 External: a package no in-scope DESCRIPTION declares — base R,
//!      CRAN and Bioconductor live outside the corpus by construction,
//!      so no name table is needed — and a `source` of a URL.
//! A `source` no rung holds is out_of_scope: a file outside the tree, or
//! a path spelled for a working directory the text does not name.

pub mod description;

use super::{Outcome, Reason, Scope, Site, paths};
use description::Description;

pub fn resolve(site: &Site, scope: &Scope) -> Outcome {
    let (from, spec) = (site.from, site.spec);
    match site.kind {
        "source" if spec.contains("://") => Outcome::External { rung: 3 },
        "source" => paths::beside_or_root(from, spec, scope)
            .map(|path| Outcome::Resolved { path, rung: 1 })
            .or_else(|| paths::one_of(paths::declared("r", spec, scope), 1))
            .unwrap_or(Outcome::Unresolved(Reason::OutOfScope)),
        "library" => package(spec, scope),
        _ => Outcome::Unresolved(Reason::Unsupported),
    }
}

/// R2: the one in-scope package of that name, or External when the tree
/// declares none.
fn package(spec: &str, scope: &Scope) -> Outcome {
    let found: Vec<Description> = super::members(scope, "DESCRIPTION", description::parse, |d| {
        d.package == spec
    });
    match found.as_slice() {
        [] => Outcome::External { rung: 3 },
        [one] => Outcome::ResolvedPackage {
            dir: one.dir.clone(),
            rung: 2,
        },
        _ => Outcome::Unresolved(Reason::AmbiguousWorkspace),
    }
}
