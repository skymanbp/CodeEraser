//! The path-joining steps several ladders share (plan v2.30 step 4):
//! C's declared roots, Lua's search directories and R's declared roots
//! judge a set of candidates the same way, and Lua's `dofile` and R's
//! `source` read a path the same way — two copies of each were the
//! clone gate's to find.

use super::{Outcome, Reason, Rung, Scope};
use crate::graph::roots;
use std::collections::BTreeSet;

/// A rung's distinct in-scope candidates as its answer: none leaves the
/// next rung to ask (None), one resolves at `rung`, two or more is one
/// name in two places — ambiguous_root, for the rung named no order.
pub(super) fn one_of(hits: BTreeSet<String>, rung: Rung) -> Option<Outcome> {
    let mut hits = hits.into_iter();
    match (hits.next(), hits.next()) {
        (None, _) => None,
        (Some(path), None) => Some(Outcome::Resolved { path, rung }),
        _ => Some(Outcome::Unresolved(Reason::AmbiguousRoot)),
    }
}

/// The in-scope files a spec names under each directory one language
/// declares in `[graph.search_roots]`.
pub(super) fn declared(lang: &str, spec: &str, scope: &Scope) -> BTreeSet<String> {
    scope
        .search_roots
        .get(lang)
        .into_iter()
        .flatten()
        .filter_map(|dir| roots::join_rel(dir, spec))
        .filter(|p| scope.files.contains(p))
        .collect()
}

/// A path as a script's working directory reads it: beside the file
/// that names it, then under the tree root (a script runs from its own
/// directory or from the project root) — the first in scope. An
/// absolute path, a home-relative one or one with a drive names no file
/// of the tree.
pub(super) fn beside_or_root(from: &str, spec: &str, scope: &Scope) -> Option<String> {
    if spec.starts_with(['/', '~']) || spec.contains(':') {
        return None;
    }
    [roots::parent_dir(from), String::new()]
        .iter()
        .filter_map(|dir| roots::join_rel(dir, spec))
        .find(|p| scope.files.contains(p))
}
