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
//! Resolution is SCOPED, never file-wide. A member off the receiver
//! reaches the caller's OWN container and nothing else: two classes in
//! one file whose methods happen to spell each other's names are not a
//! cycle, and a file-wide lookup minted exactly that pair of arcs. A
//! bare name reaches a callable whose container is an ANCESTOR of the
//! call site — what the caller can lexically see — so a module-level
//! call can never land on a class method it has no way to reach. Both
//! roads keep the undercount direction: no proof, no edge.
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

/// What a caller brings to a resolution: the container its own
/// declaration sits in, and the receiver name standing for its own
/// type. One record rather than two more parameters — the fn-params
/// line this repo sets is five.
struct Caller<'a> {
    container: usize,
    receiver: Option<&'a str>,
}

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
        let receiver = receiver_binding(unit.node, src);
        let caller = Caller {
            container: container_of(unit.node),
            receiver: receiver.as_deref(),
        };
        for node in own_nodes(unit.node, spec) {
            if !spec.call_kinds.contains(&node.kind()) {
                continue;
            }
            let Some(group) = target(node, src, spec, &caller, &named) else {
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
    containers: Vec<usize>,
    whole: HashMap<String, usize>,
    base: HashMap<String, usize>,
}

impl Named {
    fn of(units: &[FnUnit<'_>]) -> Self {
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut containers: Vec<usize> = Vec::new();
        let mut names: Vec<&str> = Vec::new();
        let mut seat: HashMap<(usize, &str), usize> = HashMap::new();
        for (i, unit) in units.iter().enumerate() {
            let parent = container_of(unit.node);
            match seat.get(&(parent, unit.name.as_str())) {
                Some(&g) => groups[g].push(i),
                None => {
                    seat.insert((parent, unit.name.as_str()), groups.len());
                    names.push(&unit.name);
                    containers.push(parent);
                    groups.push(vec![i]);
                }
            }
        }
        Named {
            whole: unique(names.iter().copied()),
            base: unique(names.iter().map(|n| base_name(n))),
            containers,
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
    caller: &Caller<'_>,
    named: &Named,
) -> Option<usize> {
    let callee = call.child_by_field_name(CALLEE)?;
    if spec.call_name_kinds.contains(&callee.kind()) {
        let group = *named.whole.get(text(callee, src)?)?;
        return sees(call, named.containers[group]).then_some(group);
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
    if !spec.call_self_words.contains(&obj) && caller.receiver != Some(obj) {
        return None;
    }
    let group = *named.base.get(text(member, src)?)?;
    (named.containers[group] == caller.container).then_some(group)
}

/// The node a declaration is declared IN — the identity a scope is
/// keyed by. Every unit sits inside its file, so the 0 a parentless
/// node would answer is a value no container ever takes, and `sees`
/// answers false for it: the safe direction.
fn container_of(node: Node<'_>) -> usize {
    node.parent().map_or(0, |p| p.id())
}

/// Whether a call site can lexically see a callable declared in
/// `container`: true exactly when that container is the call's own
/// node or one of its ancestors.
fn sees(call: Node<'_>, container: usize) -> bool {
    let mut here = Some(call);
    while let Some(node) = here {
        if node.id() == container {
            return true;
        }
        here = node.parent();
    }
    false
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
