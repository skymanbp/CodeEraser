//! C / C++ rungs (plan v2.30 step 2; the compile database completed
//! in step 5b, item 14; design booklet §8 row C / C++). The site is a
//! `preproc_include`, and its spec keeps the delimiter form the
//! detector left it — `x.h` for `"x.h"` (quotes trimmed like every
//! string specifier), `<x.h>` for the system form — because the form
//! IS the search order the language defines (C17 §6.10.2):
//!   R1 the including file's own directory, for the quoted form only —
//!      unless every compile of the file passes `-I-`, which "inhibits
//!      the use of the directory of the current file directory as the
//!      first search directory" (GCC);
//!   R2 the declared roots, `[graph.search_roots] c` (C++ shares the
//!      key), each directory joined with the spec — two directories
//!      holding two different files is ambiguous_root: the declaration
//!      is the authority here, and it named no order;
//!   R3 the compile database (c_index.rs): the chains the file
//!      compiles under — its own entries in the databases clangd would
//!      find (compdb_find.rs), else the chains of the translation units
//!      whose include closure reaches it, else the `compile_flags.txt`
//!      whose directory is the nearest holding any database — each
//!      searched in the preprocessor's class order (c_search.rs: the
//!      quoted form asks the `-iquote` class first, both forms the `-I`
//!      then the system class; cl's quoted form asks the include
//!      stack's directories before `/I`), first hit per chain; two
//!      chains answering two files is ambiguous_root — a name the build
//!      compiles as two files;
//!   R4 External: the system form with no hit is a toolchain or system
//!      header. The quoted form with no hit is out_of_scope.
//! NEVER a basename search of the tree: two `util.h` in one repository
//! are two files, and picking one would invent an edge (the register's
//! "no basename search" row).

use super::{Outcome, Reason, Scope, c_index, c_search, paths};
use crate::graph::roots;
use std::collections::BTreeSet;

pub fn resolve(from: &str, spec: &str, scope: &Scope) -> Outcome {
    let (name, system) = c_search::form(spec);
    if name.is_empty() {
        return Outcome::Unresolved(Reason::Empty);
    }
    let idx = c_index::index(scope);
    let chains = idx.chains_of(from);
    let own = !system && (chains.is_empty() || chains.iter().any(|c| idx.chains[*c].own_dir));
    own.then(|| beside(from, name, scope))
        .flatten()
        .or_else(|| declared_rung(name, scope))
        .or_else(|| database_rung(&idx, &chains, from, name, system, scope))
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
    paths::one_of(paths::declared("c", name, scope), 2)
}

/// R3: the first hit along each chain the file compiles under; one
/// distinct file resolves, two refuse.
fn database_rung(
    idx: &c_index::Index,
    chains: &BTreeSet<usize>,
    from: &str,
    name: &str,
    system: bool,
    scope: &Scope,
) -> Option<Outcome> {
    let hits: BTreeSet<String> = chains
        .iter()
        .flat_map(|c| {
            c_search::along(
                &idx.chains[*c],
                system,
                name,
                &idx.stack(*c, from),
                scope.files,
            )
        })
        .collect();
    paths::one_of(hits, 3)
}
