//! Haskell rungs (M5-3l, decision ⑧: the full package includes the
//! graph ladder, under the 2f fixture discipline). A module name is
//! dots-to-slashes under a source root; the roots are CABAL facts.
//! R1 source-root walk: the importer's owning cabal (deepest
//! directory prefix — two at one depth is ambiguous_workspace), the
//! stanzas whose roots hold the importer (none holding it ⇒ every
//! stanza — which component owns the file is then unknowable), one
//! candidate per root; DISTINCT files from different roots is
//! ambiguous_root, never a pick (GHC itself rejects the collision).
//! No cabal owning the importer anchors at the repo root — bare
//! `ghc Main.hs` search semantics. R2 External: the machine-generated
//! global-package-db table (hs_boot.rs), gated by the owner cabal's
//! build-depends — under cabal a package's modules are importable
//! only if declared; with no cabal every db package is default-
//! visible (bare-ghc semantics), so the whole table answers.
//!
//! Stated boundaries (refusals in the ledger, never guesses):
//! cross-package in-corpus imports (a build-depends target that is
//! ANOTHER in-scope cabal) need exposed-modules modeling and land
//! out_of_scope; store-installed deps outside the global db (aeson)
//! likewise; PackageImports package qualifiers are invisible in the
//! spec (the module field carries dots only) so such an import
//! resolves as if unqualified — the extension appears in none of the
//! six corpora.
//!
//! R6/RG10 re-review (3l exit row): a module's export list is a
//! symbol-level fact (AST-probed: header/exports), and the file-tier
//! exported bit stays unset for Haskell exactly as for the five
//! launch languages (deadcode.rs flags_of) — RG10's unref_public
//! class cannot fire for .hs until symbol-level flags land; recorded
//! stance, no new mechanism.

use super::hs_boot::BOOT;
use super::{Outcome, Reason, Scope};
use crate::graph::cabal::{self, Cabal};
use crate::graph::roots;
use std::collections::BTreeSet;

/// Every dot-separated segment opens with an uppercase letter —
/// anything else is not a module name and nothing resolvable was
/// referenced. (Helper-first on purpose: the ladder preamble —
/// use block + resolve signature — is token-identical across the
/// parallel ladder modules under T2 folding, and the validator
/// between them is the structural boundary that keeps the shared
/// run below the clone floor.)
fn module_shaped(spec: &str) -> bool {
    !spec.is_empty()
        && spec
            .split('.')
            .all(|seg| seg.starts_with(|c: char| c.is_ascii_uppercase()))
}

pub fn resolve(from: &str, spec: &str, scope: &Scope) -> Outcome {
    if !module_shaped(spec) {
        return Outcome::Unresolved(Reason::OutOfScope);
    }
    let owner = match owner(from, scope) {
        Ok(owner) => owner,
        Err(reason) => return Outcome::Unresolved(reason),
    };
    let rel = format!("{}.hs", spec.replace('.', "/"));
    let hits: BTreeSet<String> = source_roots(owner.as_ref(), from)
        .iter()
        .filter_map(|root| {
            let path = roots::join_dir(root, &rel);
            scope.files.contains(&path).then_some(path)
        })
        .collect();
    match hits.len() {
        1 => Outcome::Resolved {
            path: hits.into_iter().next().expect("len checked"),
            rung: 1,
        },
        0 => external_rung(spec, owner.as_ref()),
        _ => Outcome::Unresolved(Reason::AmbiguousRoot),
    }
}

/// The cabal whose directory is the deepest prefix of `from`; None
/// is a legitimate anchor (bare ghc), two at one depth refuse.
fn owner(from: &str, scope: &Scope) -> Result<Option<Cabal>, Reason> {
    let mut best: Option<Cabal> = None;
    let mut tied = false;
    for rel in scope.configs.iter().filter(|c| c.ends_with(".cabal")) {
        // per-sweep: one parse per .cabal, not one per SITE per
        // .cabal (review MED)
        let hit = scope
            .memo
            .cached("cabal", rel, || cabal::parse(scope.root, rel));
        let Some(c) = hit.as_ref().clone() else {
            continue;
        };
        if !(c.dir.is_empty() || from.starts_with(&format!("{}/", c.dir))) {
            continue;
        }
        match &best {
            Some(b) if c.dir.len() > b.dir.len() => {
                best = Some(c);
                tied = false;
            }
            Some(b) if c.dir.len() == b.dir.len() => tied = true,
            Some(_) => {}
            None => best = Some(c),
        }
    }
    if tied {
        return Err(Reason::AmbiguousWorkspace);
    }
    Ok(best)
}

/// R1 root set: the owning stanzas' roots (falling back to every
/// stanza), or the repo root without a cabal.
fn source_roots(owner: Option<&Cabal>, from: &str) -> Vec<String> {
    let Some(c) = owner else {
        return vec![String::new()];
    };
    let holds = |roots: &[String]| {
        roots
            .iter()
            .any(|r| r.is_empty() || from.starts_with(&format!("{r}/")))
    };
    let owning: Vec<&cabal::Stanza> = c.stanzas.iter().filter(|s| holds(&s.roots)).collect();
    let picked = if owning.is_empty() {
        c.stanzas.iter().collect()
    } else {
        owning
    };
    let set: BTreeSet<String> = picked
        .iter()
        .flat_map(|s| s.roots.iter().cloned())
        .collect();
    set.into_iter().collect()
}

/// R2: the global-db table, build-depends-gated under a cabal.
fn external_rung(spec: &str, owner: Option<&Cabal>) -> Outcome {
    let hit = BOOT.iter().any(|(pkg, modules)| {
        owner.is_none_or(|c| c.deps.iter().any(|d| d == pkg))
            && modules.split_whitespace().any(|m| m == spec)
    });
    if hit {
        Outcome::External { rung: 2 }
    } else {
        Outcome::Unresolved(Reason::OutOfScope)
    }
}
