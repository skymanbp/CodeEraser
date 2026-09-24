//! The callables one parse unit declares and the names they answer
//! to — the index scan/calls.rs resolves every call site against.
//! Split from calls.rs when the C++ owner road pushed that file past
//! the 300-line line (plan v2.30 step 2); the rules stay as written,
//! only the seat is new.
//!
//! Two roads, two keys. WHOLE: a bare name matched against the unit's
//! full spelling, members excluded (a method answers to a receiver,
//! never to its own name alone). BASE: (owner, name with the receiver
//! / class qualification stripped), which is what `this->m()`,
//! `K::m()` and — inside a member of `K` — a bare `m()` look up. A
//! key owned by two groups is dropped rather than disambiguated: that
//! one rule keeps the measured false positives out.
//!
//! A callable is not always one unit: Haskell gives every equation of
//! `f` its own `function` node (divergence D7), and those equations
//! are one function, not an ambiguity. So names are owned by GROUPS
//! keyed by (parent, name) — equations share a parent, while a
//! `where`-local `go` and a top-level `go` do not, and stay two
//! callables that cancel each other out.

use super::functions::{self, FnUnit};
use super::spec::LangSpec;
use std::collections::HashMap;
use std::hash::Hash;
use tree_sitter::Node;

/// A base-road key: the owner a member answers to (None for a free
/// callable and for every grammar without one) and the name with its
/// receiver / class qualification stripped.
pub(super) type BaseKey = (Option<String>, String);

pub(super) struct Named {
    pub(super) groups: Vec<Vec<usize>>,
    pub(super) containers: Vec<usize>,
    pub(super) whole: HashMap<String, usize>,
    pub(super) base: HashMap<BaseKey, usize>,
}

/// Whether a declaration sits in a type's member body — see
/// LangSpec::call_member_scopes for why both levels are read. Crate-
/// visible: the similar bag's shape word (similar/bag.rs) asks the
/// same question of the same node, and asked it once more verbatim
/// before the dedup guard sent it here.
pub(crate) fn in_member_scope(node: Node<'_>, spec: &LangSpec) -> bool {
    node.parent().is_some_and(|container| {
        spec.call_member_scopes.contains(&container.kind())
            || container
                .parent()
                .is_some_and(|owner| spec.call_member_scopes.contains(&owner.kind()))
    })
}

/// The node a declaration is declared IN — the identity a scope is
/// keyed by. Every unit sits inside its file, so the 0 a parentless
/// node would answer is a value no container ever takes, and
/// calls::sees answers false for it: the safe direction.
pub(super) fn container_of(node: Node<'_>) -> usize {
    node.parent().map_or(0, |p| p.id())
}

impl Named {
    pub(super) fn of(units: &[FnUnit<'_>], src: &[u8], spec: &LangSpec) -> Self {
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut containers: Vec<usize> = Vec::new();
        let mut seats: Vec<(&str, bool, Option<String>)> = Vec::new();
        let mut seat: HashMap<(usize, &str), usize> = HashMap::new();
        for (i, unit) in units.iter().enumerate() {
            let parent = container_of(unit.node);
            match seat.get(&(parent, unit.name.as_str())) {
                Some(&g) => groups[g].push(i),
                None => {
                    seat.insert((parent, unit.name.as_str()), groups.len());
                    seats.push((
                        &unit.name,
                        in_member_scope(unit.node, spec),
                        functions::owner_of(unit.node, src),
                    ));
                    containers.push(parent);
                    groups.push(vec![i]);
                }
            }
        }
        let rows = seats.iter().enumerate();
        Named {
            // a member is absent from the bare road entirely, so a
            // top-level name is not cancelled by a method spelling it
            whole: unique(
                rows.clone()
                    .filter(|(_, (_, member, _))| !member)
                    .map(|(g, (n, ..))| (n.to_string(), g)),
            ),
            base: unique(
                rows.map(|(g, (n, _, owner))| ((owner.clone(), base_name(n).to_string()), g)),
            ),
            containers,
            groups,
        }
    }
}

/// Keys owned by exactly one group; a repeat erases its key for good.
fn unique<K: Eq + Hash>(named: impl Iterator<Item = (K, usize)>) -> HashMap<K, usize> {
    let mut seen: HashMap<K, Option<usize>> = HashMap::new();
    for (key, group) in named {
        seen.entry(key)
            .and_modify(|slot| *slot = None)
            .or_insert(Some(group));
    }
    seen.into_iter()
        .filter_map(|(key, slot)| Some((key, slot?)))
        .collect()
}

/// `(T) add` → `add` — the receiver qualification functions::name_of
/// puts on Go methods — and `K::b` → `b`, the class qualification it
/// puts on C++ members. Every other spelling is its own base.
fn base_name(name: &str) -> &str {
    let name = name
        .strip_prefix('(')
        .and_then(|rest| rest.split_once(") "))
        .map_or(name, |(_, base)| base);
    name.rsplit("::").next().unwrap_or(name)
}

#[cfg(test)]
#[path = "../../tests/unit/scan/callees.rs"]
mod tests;
