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
//! One thing the ancestor rule alone gets wrong, and the reason a
//! member scope is named separately: inside `impl Drop { fn drop(..)
//! { drop(x) } }` the callable `drop` IS an ancestor's child, yet the
//! bare `drop` is the prelude's free function — a method answers to a
//! receiver, never to its own name alone. Measured on this repository
//! before the rule existed, that one shape charged a recursion point
//! to a `Drop` impl that recurses nowhere.
//!
//! The same reading applies to a name an IMPORT binds: inside
//! `fn symlink { use ...::symlink; symlink(src, dst) }` the bare call
//! is the imported function, because a `use` item is a lexical binding
//! and the innermost one wins. The crosscheck corpus held exactly that
//! shape, and it was the only unit in four corpora the increment moved.
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
use std::collections::{HashMap, HashSet};
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
    shadowed: HashSet<&'a str>,
}

/// Edges `(caller, callee)` as indices into `units`, sorted and
/// deduplicated. Self-edges are kept: direct recursion is a cycle of
/// length one and the core reads it straight off the arc set.
pub fn edges(units: &[FnUnit<'_>], src: &[u8], spec: &LangSpec) -> Vec<(usize, usize)> {
    if spec.call_kinds.is_empty() {
        return Vec::new();
    }
    let named = Named::of(units, spec);
    let mut out = Vec::new();
    for (from, unit) in units.iter().enumerate() {
        let receiver = receiver_binding(unit.node, src);
        let own = own_nodes(unit.node, spec);
        let caller = Caller {
            container: container_of(unit.node),
            receiver: receiver.as_deref(),
            shadowed: shadowed(&own, src, spec),
        };
        for node in own {
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

/// Whether a declaration sits in a type's member body — see
/// LangSpec::call_member_scopes for why both levels are read.
fn in_member_scope(node: Node<'_>, spec: &LangSpec) -> bool {
    node.parent().is_some_and(|container| {
        spec.call_member_scopes.contains(&container.kind())
            || container
                .parent()
                .is_some_and(|owner| spec.call_member_scopes.contains(&owner.kind()))
    })
}

impl Named {
    fn of(units: &[FnUnit<'_>], spec: &LangSpec) -> Self {
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut containers: Vec<usize> = Vec::new();
        let mut names: Vec<(&str, bool)> = Vec::new();
        let mut seat: HashMap<(usize, &str), usize> = HashMap::new();
        for (i, unit) in units.iter().enumerate() {
            let parent = container_of(unit.node);
            match seat.get(&(parent, unit.name.as_str())) {
                Some(&g) => groups[g].push(i),
                None => {
                    seat.insert((parent, unit.name.as_str()), groups.len());
                    names.push((&unit.name, in_member_scope(unit.node, spec)));
                    containers.push(parent);
                    groups.push(vec![i]);
                }
            }
        }
        let seats = names.iter().enumerate();
        Named {
            // a member is absent from the bare road entirely, so a
            // top-level name is not cancelled by a method spelling it
            whole: unique(
                seats
                    .clone()
                    .filter(|(_, (_, m))| !m)
                    .map(|(g, (n, _))| (*n, g)),
            ),
            base: unique(seats.map(|(g, (n, _))| (base_name(n), g))),
            containers,
            groups,
        }
    }
}

/// Keys owned by exactly one group; a repeat erases its key for good.
fn unique<'a>(named: impl Iterator<Item = (&'a str, usize)>) -> HashMap<String, usize> {
    let mut seen: HashMap<&str, Option<usize>> = HashMap::new();
    for (name, group) in named {
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
        let name = text(callee, src)?;
        if caller.shadowed.contains(name) {
            return None;
        }
        let group = *named.whole.get(name)?;
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

/// Names an import binds inside a unit's own body — see
/// LangSpec::call_import_kinds. Every name spelled anywhere under such
/// an item counts: with no resolver, an alias, a list member and a
/// path segment are indistinguishable, and vetoing one name too many
/// only costs an edge, which is the direction this module errs in.
fn shadowed<'s>(own: &[Node<'_>], src: &'s [u8], spec: &LangSpec) -> HashSet<&'s str> {
    let mut out = HashSet::new();
    let imports = own
        .iter()
        .filter(|n| spec.call_import_kinds.contains(&n.kind()));
    for import in imports {
        let mut stack = vec![*import];
        while let Some(node) = stack.pop() {
            if spec.call_name_kinds.contains(&node.kind()) {
                out.extend(text(node, src));
            }
            stack.extend(ast::children(node));
        }
    }
    out
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
