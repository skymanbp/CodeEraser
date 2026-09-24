//! C / C++ rungs (plan v2.30 step 2; design booklet §8 row C / C++).
//! The site is a `preproc_include`, and its spec keeps the delimiter
//! form the detector left it — `x.h` for `"x.h"` (quotes trimmed like
//! every string specifier), `<x.h>` for the system form — because the
//! form IS the search order the language defines (C17 §6.10.2):
//!   R1 the including file's own directory, for the quoted form only;
//!   R2 the declared roots, `[graph.search_roots] c` (C++ shares the
//!      key), each directory joined with the spec — two directories
//!      holding two different files is ambiguous_root: the declaration
//!      is the authority here, and it named no order;
//!   R3 the compile database: the `-I` / `-iquote` / `-isystem`
//!      directories of the including file's OWN compile_commands.json
//!      entry, in invocation order, the FIRST hit winning — that is the
//!      compiler's rule, so no ambiguity exists at this rung; a header
//!      has no entry (a database lists translation units) and reads
//!      nothing here;
//!   R4 External: the system form with no hit is a toolchain or system
//!      header. The quoted form with no hit is out_of_scope.
//! NEVER a basename search of the tree: two `util.h` in one repository
//! are two files, and picking one would invent an edge (the register's
//! "no basename search" row).

use std::collections::BTreeSet;

use super::Scope;
use super::{Outcome, Reason};
use crate::graph::compdb;
use crate::graph::roots;

pub fn resolve(from: &str, spec: &str, scope: &Scope) -> Outcome {
    let (name, system) = match spec.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
        Some(inner) => (inner, true),
        None => (spec, false),
    };
    if name.is_empty() {
        return Outcome::Unresolved(Reason::Empty);
    }
    let own = (!system).then(|| beside(from, name, scope)).flatten();
    own.or_else(|| declared_rung(name, scope))
        .or_else(|| database_rung(from, name, scope))
        .unwrap_or(if system {
            Outcome::External { rung: 4 }
        } else {
            Outcome::Unresolved(Reason::OutOfScope)
        })
}

/// R1: the including file's own directory.
fn beside(from: &str, name: &str, scope: &Scope) -> Option<Outcome> {
    let path = roots::join_rel(&roots::parent_dir(from), name)?;
    scope
        .files
        .contains(&path)
        .then_some(Outcome::Resolved { path, rung: 1 })
}

/// R2: every declared root joined with the name; one distinct hit
/// resolves, two refuse.
fn declared_rung(name: &str, scope: &Scope) -> Option<Outcome> {
    let hits: BTreeSet<String> = scope
        .search_roots
        .get("c")
        .into_iter()
        .flatten()
        .filter_map(|dir| roots::join_rel(dir, name))
        .filter(|p| scope.files.contains(p))
        .collect();
    match hits.len() {
        0 => None,
        1 => Some(Outcome::Resolved {
            path: hits.into_iter().next()?,
            rung: 2,
        }),
        _ => Some(Outcome::Unresolved(Reason::AmbiguousRoot)),
    }
}

/// R3: the including file's own compile entries, first hit in
/// invocation order.
fn database_rung(from: &str, name: &str, scope: &Scope) -> Option<Outcome> {
    let dbs: Vec<compdb::CompDb> =
        super::members(scope, "compile_commands.json", compdb::parse, |_| true);
    let path = dbs
        .iter()
        .flat_map(|db| &db.entries)
        .filter(|(unit, _)| unit == from)
        .flat_map(|(_, dirs)| dirs)
        .filter_map(|dir| roots::join_rel(dir, name))
        .find(|p| scope.files.contains(p))?;
    Some(Outcome::Resolved { path, rung: 3 })
}
