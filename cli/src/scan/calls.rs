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
//! declares (the index is scan/callees.rs): a bare name matched WHOLE,
//! or a member selected off the caller's own receiver, matched with
//! the receiver prefix stripped. Anything else mints nothing, and a
//! name owned by two callables mints nothing either. With no semantic
//! model the error direction is undercount, never a wrong +1 — a wrong
//! one would flow into the score and the size gate, an absent one only
//! leaves a point unpaid.
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
//! C++ (plan v2.30 step 2) adds one thing the container rule cannot
//! express: a member may be DEFINED out of its class (`void K::b()`),
//! so the members of one class do not share a container, and inside a
//! member a bare `m()` reaches the class's own `m` before anything the
//! enclosing scopes declare. Both roads therefore carry the OWNER the
//! unit answers to (functions::owner_of — the class chain of an
//! in-body definition, the spelled qualifier of an out-of-class one):
//! the member road matches `this->m()`, `K::m()` and, inside a member
//! of `K`, a bare `m()` against (owner, base) — undercount still: a
//! qualifier naming any other class, a call off another object, or a
//! virtual override in some other file all mint nothing. Once the
//! class declares the name, the enclosing scopes are never consulted —
//! class scope hides them, even where the class's own seats cannot
//! tell which one the call reaches.
//!
//! Overloads (C++, Java — LangSpec::overloads) split one name into
//! several callables, and a call reaches the one whose argument range
//! admits its count (callees::Named::pick). Reading them as one
//! callable — the step-2 C++ reading — charged a delegating overload
//! (`f(double d) { f((int)d); }`) a recursion point it does not have.
//!
//! Java (plan v2.30 step 3) hangs the receiver off the call itself
//! (LangSpec::call_fields), and its member road keys by the one class
//! BODY a member sits in (callees::Owner): a local class may share its
//! name with another, and an anonymous class has none, yet each body is
//! one class, so `this.m()` and a bare `m()` inside either reach the
//! class's own `m`. `super.m()` is never the caller's own `m` — an
//! override calling its super is Java's commonest delegation, not a
//! recursion.

use super::ast;
use super::callees::{Named, Owner, container_of, owner_key};
use super::functions::{self, FnUnit};
use super::metrics::own_nodes;
use super::spec::LangSpec;
use std::collections::HashSet;
use tree_sitter::Node;

/// What a caller brings to a resolution: the container its own
/// declaration sits in, the receiver name standing for its own type,
/// the names its own imports bind, and (C++, Java) the owner its
/// member road keys by and the class name a qualifier must spell to
/// name it. One record rather than more parameters — the fn-params
/// line this repo sets is five.
struct Caller<'a> {
    container: usize,
    receiver: Option<&'a str>,
    shadowed: HashSet<&'a str>,
    owner: Option<Owner>,
    class: Option<String>,
}

/// What a call spells: a bare name, or an object and the member
/// selected off it.
enum Callee<'t> {
    Bare(Node<'t>),
    Member(Node<'t>, Node<'t>),
}

/// Edges `(caller, callee)` as indices into `units`, sorted and
/// deduplicated. Self-edges are kept: direct recursion is a cycle of
/// length one and the core reads it straight off the arc set.
pub fn edges(units: &[FnUnit<'_>], src: &[u8], spec: &LangSpec) -> Vec<(usize, usize)> {
    if spec.call_kinds.is_empty() {
        return Vec::new();
    }
    let named = Named::of(units, src, spec);
    let mut out = Vec::new();
    for (from, unit) in units.iter().enumerate() {
        let receiver = receiver_binding(unit.node, src);
        let own = own_nodes(unit.node, src, spec);
        let caller = Caller {
            container: container_of(unit.node),
            receiver: receiver.as_deref(),
            shadowed: shadowed(&own, src, spec),
            owner: owner_key(unit.node, src, spec),
            class: functions::owner_of(unit.node, src, spec),
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

/// The group a call reaches, or None when nothing proves one.
fn target(
    call: Node<'_>,
    src: &[u8],
    spec: &LangSpec,
    caller: &Caller<'_>,
    named: &Named,
) -> Option<usize> {
    match callee(call, spec)? {
        Callee::Bare(name) => bare(call, text(name, src)?, caller, named),
        Callee::Member(object, member) => {
            let obj = text(object, src)?;
            let own_type = spec.call_self_words.contains(&obj)
                || caller.receiver == Some(obj)
                || owner_tail(caller) == Some(obj);
            if !own_type {
                return None;
            }
            let key = (caller.owner.clone(), text(member, src)?.to_string());
            own(named.pick(named.base.get(&key)?, call)?, caller, named)
        }
    }
}

/// What a call spells, read through LangSpec::call_fields: a receiver
/// field on the call itself makes it a member call (Java), otherwise
/// the callee node's own shape decides.
fn callee<'t>(call: Node<'t>, spec: &LangSpec) -> Option<Callee<'t>> {
    let (callee_field, receiver_field) = spec.call_fields;
    let callee = call.child_by_field_name(callee_field)?;
    if let Some(object) = receiver_field.and_then(|f| call.child_by_field_name(f)) {
        return Some(Callee::Member(object, callee));
    }
    if spec.call_name_kinds.contains(&callee.kind()) {
        return Some(Callee::Bare(callee));
    }
    if !spec.call_member_kinds.contains(&callee.kind()) {
        return None;
    }
    let kids = ast::named_children(callee);
    let (object, member) = (*kids.first()?, *kids.last()?);
    (object.id() != member.id()).then_some(Callee::Member(object, member))
}

/// The bare road. A name an import binds is not provably the local
/// callable; inside a member the class's own members come first
/// (class scope precedes every enclosing scope, and a C++ member
/// defined out of class is not lexically visible at all); otherwise a
/// whole name reaches a callable the call site can see.
fn bare(call: Node<'_>, name: &str, caller: &Caller<'_>, named: &Named) -> Option<usize> {
    if caller.shadowed.contains(name) {
        return None;
    }
    if caller.owner.is_some()
        && let Some(seats) = named.base.get(&(caller.owner.clone(), name.to_string()))
    {
        return own(named.pick(seats, call)?, caller, named);
    }
    let group = named.pick(named.whole.get(name)?, call)?;
    sees(call, named.containers[group]).then_some(group)
}

/// A callee of the caller's own type: an owner key proves it (the seats
/// it names are the caller's class's own, by construction); without
/// one, the callee must sit in the caller's own container.
fn own(group: usize, caller: &Caller<'_>, named: &Named) -> Option<usize> {
    (caller.owner.is_some() || named.containers[group] == caller.container).then_some(group)
}

/// `K::m()` / `K.m()` inside a member of `K`: the qualifier names the
/// caller's own class by its unqualified name — the innermost segment
/// of a C++ owner chain (`Outer::Inner` answers to `Inner`). Any other
/// qualifier (a base class, a namespace) proves nothing here.
fn owner_tail<'c>(caller: &'c Caller<'_>) -> Option<&'c str> {
    caller.class.as_deref().and_then(|o| o.rsplit("::").next())
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
#[cfg(test)]
#[path = "../../tests/unit/scan/calls_scope.rs"]
mod tests_scope;
