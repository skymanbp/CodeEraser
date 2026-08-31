//! Call edges between the units of ONE parse unit — the syntax half of
//! the recursion increment (ADR-008 fourth instalment, plan v2.23).
//! Cycles over these edges are the core's judgment; this side only
//! states who calls whom, and only where the callee is provable.
//!
//! The reading is deliberately NOT "the unit's own name appears in its
//! body". Measured on the crosscheck corpora, that reading fires on
//! `DirEntry::path` calling `self.dent.path()` and on `Request.prepare`
//! calling `p.prepare(...)` — same spelling, a different type. A callee
//! resolves two ways instead, both against the units this file itself
//! declares: a bare name matched WHOLE, or a member selected off the
//! caller's own receiver, matched with the receiver prefix stripped.
//! Anything else mints nothing, and a name owned by two callables
//! mints nothing either. With no semantic model the error direction is
//! undercount, never a wrong +1 — a wrong one would flow into the
//! score and the size gate, an absent one only leaves a point unpaid.
//!
//! A callable is not always one unit: Haskell gives every equation of
//! `f` its own `function` node (divergence D7), and those equations
//! are one function, not an ambiguity. So names are owned by GROUPS
//! keyed by (parent, name) — equations share a parent, while a
//! `where`-local `go` and a top-level `go` do not, and stay two
//! callables that cancel each other out.

use super::ast;
use super::functions::FnUnit;
use super::metrics::own_nodes;
use super::spec::LangSpec;
use std::collections::HashMap;
use tree_sitter::Node;

/// The callee's field name — one spelling across all five grammars
/// (probed against the pinned versions, not recalled).
const CALLEE: &str = "function";

/// Edges `(caller, callee)` as indices into `units`, sorted and
/// deduplicated. Self-edges are kept: direct recursion is a cycle of
/// length one and the core reads it straight off the arc set.
pub fn edges(units: &[FnUnit<'_>], src: &[u8], spec: &LangSpec) -> Vec<(usize, usize)> {
    if spec.call_kinds.is_empty() {
        return Vec::new();
    }
    let named = Named::of(units);
    let mut out = Vec::new();
    for (from, unit) in units.iter().enumerate() {
        let mine = receiver_binding(unit.node, src);
        for node in own_nodes(unit.node, spec) {
            if !spec.call_kinds.contains(&node.kind()) {
                continue;
            }
            let Some(group) = target(node, src, spec, mine.as_deref(), &named) else {
                continue;
            };
            out.extend(named.groups[group].iter().map(|&to| (from, to)));
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// The callables this file declares and the names they answer to,
/// whole and receiver-stripped. A name owned by two groups is dropped
/// rather than disambiguated — that one rule keeps the measured false
/// positives out.
struct Named {
    groups: Vec<Vec<usize>>,
    whole: HashMap<String, usize>,
    base: HashMap<String, usize>,
}

impl Named {
    fn of(units: &[FnUnit<'_>]) -> Self {
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut names: Vec<&str> = Vec::new();
        let mut seat: HashMap<(usize, &str), usize> = HashMap::new();
        for (i, unit) in units.iter().enumerate() {
            let parent = unit.node.parent().map_or(0, |p| p.id());
            match seat.get(&(parent, unit.name.as_str())) {
                Some(&g) => groups[g].push(i),
                None => {
                    seat.insert((parent, unit.name.as_str()), groups.len());
                    names.push(&unit.name);
                    groups.push(vec![i]);
                }
            }
        }
        Named {
            whole: unique(names.iter().copied()),
            base: unique(names.iter().map(|n| base_name(n))),
            groups,
        }
    }
}

/// Keys owned by exactly one group; a repeat erases its key for good.
fn unique<'a>(names: impl Iterator<Item = &'a str>) -> HashMap<String, usize> {
    let mut seen: HashMap<&str, Option<usize>> = HashMap::new();
    for (group, name) in names.enumerate() {
        seen.entry(name)
            .and_modify(|slot| *slot = None)
            .or_insert(Some(group));
    }
    seen.into_iter()
        .filter_map(|(name, slot)| Some((name.to_string(), slot?)))
        .collect()
}

/// `(T) add` → `add` — the receiver qualification functions::name_of
/// puts on Go methods. Every other spelling is its own base.
fn base_name(name: &str) -> &str {
    name.strip_prefix('(')
        .and_then(|rest| rest.split_once(") "))
        .map_or(name, |(_, base)| base)
}

/// The group a call reaches, or None when nothing proves one.
fn target(
    call: Node<'_>,
    src: &[u8],
    spec: &LangSpec,
    mine: Option<&str>,
    named: &Named,
) -> Option<usize> {
    let callee = call.child_by_field_name(CALLEE)?;
    if spec.call_name_kinds.contains(&callee.kind()) {
        return named.whole.get(text(callee, src)?).copied();
    }
    if !spec.call_member_kinds.contains(&callee.kind()) {
        return None;
    }
    let kids = ast::named_children(callee);
    let (object, member) = (*kids.first()?, *kids.last()?);
    if object.id() == member.id() {
        return None;
    }
    let obj = text(object, src)?;
    if !spec.call_self_words.contains(&obj) && mine != Some(obj) {
        return None;
    }
    named.base.get(text(member, src)?).copied()
}

/// A Go method's receiver BINDING (`t` in `func (t *T) g()`): the one
/// object name besides the self words that stands for the caller's own
/// type. functions::name_of deliberately takes the receiver's TYPE for
/// identity, so the binding is read here rather than shared.
fn receiver_binding(node: Node<'_>, src: &[u8]) -> Option<String> {
    let recv = node.child_by_field_name("receiver")?;
    let decl = ast::named_children(recv)
        .into_iter()
        .find(|c| c.kind() == "parameter_declaration")?;
    text(decl.child_by_field_name("name")?, src).map(str::to_string)
}

fn text<'s>(node: Node<'_>, src: &'s [u8]) -> Option<&'s str> {
    node.utf8_text(src).ok()
}

#[cfg(test)]
#[path = "../../tests/unit/scan/calls.rs"]
mod tests;
