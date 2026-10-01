//! Go rungs (design §4 row 4). Go resolves at PACKAGE granularity:
//! an import path names a directory, the node identity is
//! (pkg_dir, ""), and collapsing to one file would be a guess — so
//! this ladder returns ResolvedPackage, and a directory only counts
//! as an importable package while it directly holds an in-scope
//! non-_test.go file (a test-only directory is not a target). R1 the
//! module-prefix walk: the LONGEST in-scope go.mod module path
//! owning the spec wins — longest is how nested modules own their
//! subtrees; two modules declaring one path is ambiguous_workspace.
//! R2 the importer's nearest module's replace directives: a
//! filesystem target (./ or ../ per the spec) maps into the corpus,
//! a module target is rewritten and retried against the module set.
//! R3 External: the stdlib table (machine-generated, importable set
//! only), or a dotted first segment matching no local module — heads
//! without a dot are reserved for the standard library, so a dotless
//! head outside the table is out_of_scope, never External.
//! //go:build-constrained files still count toward package existence
//! (design R4: tag evaluation needs a build configuration we do not
//! have; the build_constrained flag is wiring-level metadata).

use super::{Outcome, Reason, Scope};
use crate::graph::gomod::{self, GoMod};
use crate::graph::roots;
use std::collections::BTreeSet;

pub fn resolve(from: &str, spec: &str, scope: &Scope) -> Outcome {
    let mods = modules(scope);
    if let Some(outcome) = module_rung(spec, &mods, scope) {
        return outcome;
    }
    if let Some(outcome) = replace_rung(from, spec, &mods, scope) {
        return outcome;
    }
    external_rung(spec)
}

/// Every in-scope go.mod surface that declares a module path.
fn modules(scope: &Scope) -> Vec<GoMod> {
    super::members(scope, "go.mod", gomod::parse, |m| m.module.is_some())
}

/// R1: the longest module prefix owns the import; the remainder
/// names a package directory under the module's own directory.
fn module_rung(spec: &str, mods: &[GoMod], scope: &Scope) -> Option<Outcome> {
    let mut best_len = 0;
    let mut dirs = BTreeSet::new();
    for m in mods {
        let Some(module) = m.module.as_deref() else {
            continue;
        };
        let Some(rest) = strip_module(spec, module) else {
            continue;
        };
        let dir = if rest.is_empty() {
            m.dir.clone()
        } else {
            roots::join_dir(&m.dir, rest)
        };
        if module.len() > best_len {
            best_len = module.len();
            dirs = BTreeSet::from([dir]);
        } else if module.len() == best_len {
            dirs.insert(dir);
        }
    }
    match dirs.len() {
        0 => None,
        1 => Some(package(dirs.pop_first().expect("len checked"), scope, 1)),
        _ => Some(Outcome::Unresolved(Reason::AmbiguousWorkspace)),
    }
}

/// Module-path prefix match on a segment boundary; the remainder
/// never keeps a leading slash.
fn strip_module<'a>(spec: &'a str, module: &str) -> Option<&'a str> {
    let rest = spec.strip_prefix(module)?;
    if rest.is_empty() {
        Some(rest)
    } else {
        rest.strip_prefix('/')
    }
}

/// R2: the importer's module's replace directives, longest old wins.
fn replace_rung(from: &str, spec: &str, mods: &[GoMod], scope: &Scope) -> Option<Outcome> {
    let owner = owner(from, mods)?;
    let (rest, new) = owner
        .replaces
        .iter()
        .filter_map(|(old, new)| Some((strip_module(spec, old)?, new)))
        .min_by_key(|(rest, _)| rest.len())?;
    if new.starts_with("./") || new.starts_with("../") {
        let base = roots::join_rel(&owner.dir, new)?;
        let dir = if rest.is_empty() {
            base
        } else {
            roots::join_dir(&base, rest)
        };
        return Some(package(dir, scope, 2));
    }
    let rewritten = if rest.is_empty() {
        new.clone()
    } else {
        format!("{new}/{rest}")
    };
    match module_rung(&rewritten, mods, scope) {
        Some(outcome) => Some(outcome.with_rung(2)),
        None => Some(external_rung(&rewritten)),
    }
}

/// The module whose directory is the deepest prefix of `from`.
fn owner<'a>(from: &str, mods: &'a [GoMod]) -> Option<&'a GoMod> {
    mods.iter()
        .filter(|m| m.dir.is_empty() || from.starts_with(&format!("{}/", m.dir)))
        .max_by_key(|m| m.dir.len())
}

/// A directory is an importable package while it directly holds an
/// in-scope non-test .go file — an O(files) scan every import of one
/// directory used to repay (review MED), now once per sweep.
fn package(dir: String, scope: &Scope, rung: u8) -> Outcome {
    let importable = *scope.memo.cached("go_pkg", &dir, || {
        let prefix = if dir.is_empty() {
            String::new()
        } else {
            format!("{dir}/")
        };
        scope.files.iter().any(|f| {
            f.strip_prefix(&prefix).is_some_and(|rest| {
                rest.ends_with(".go") && !rest.contains('/') && !rest.ends_with("_test.go")
            })
        })
    });
    if importable {
        Outcome::ResolvedPackage { dir, rung }
    } else {
        Outcome::Unresolved(Reason::OutOfScope)
    }
}

/// R3: the stdlib table, or a dotted head with no local match.
fn external_rung(spec: &str) -> Outcome {
    let head = spec.split('/').next().unwrap_or(spec);
    if head.contains('.') || crate::tables::get().ladder.go.std.contains(&spec) {
        return Outcome::External { rung: 3 };
    }
    Outcome::Unresolved(Reason::OutOfScope)
}
