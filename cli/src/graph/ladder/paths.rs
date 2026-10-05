//! A rung's candidates judged as one answer (plan v2.30 step 4), the
//! step Haskell's search directories take. C's, Lua's and R's copies of
//! this module's steps moved into the core with their ladders
//! (`CE.Resolve.Answer`, `CE.Resolve.World`; plan v2.33 W2a and
//! W2-text stage B).

use super::{Outcome, Reason, Rung};
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
