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
//! `K::m()` and — inside a member of `K` — a bare `m()` look up; the
//! owner is what Owner says it is per grammar. A
//! key naming two callables reaches neither, unless the grammar
//! overloads and the call's argument count admits exactly one of them
//! (`pick`): that one rule keeps the measured false positives out.
//!
//! A callable is not always one unit: Haskell gives every equation of
//! `f` its own `function` node (divergence D7), and those equations
//! are one function, not an ambiguity. So names are owned by GROUPS
//! keyed by (container, name) — the container is the unit's parent,
//! or the scope a Lua or R binding lives in (`container_of`):
//! equations share a parent, while a `where`-local `go` and a
//! top-level `go` do not, and stay two callables that cancel each
//! other out. A grammar that overloads
//! (C++, Java — LangSpec::overloads) reads the other way: its
//! same-named units of one scope are different functions, each a group
//! of its own, told apart by the arguments a call passes.

use super::ast;
use super::binding;
use super::functions::{self, FnUnit};
use super::spec::{LangSpec, Overloads};
use std::collections::HashMap;
use tree_sitter::Node;

/// A base-road key: the owner a member answers to (None for a free
/// callable and for every grammar without one) and the name with its
/// receiver / class qualification stripped.
pub(super) type BaseKey = (Option<Owner>, String);

/// Who a member answers to on the base road.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) enum Owner {
    /// The class a C++ declarator spells (functions::owner_of): a
    /// member defined out of class meets its class there, wherever the
    /// definition sits.
    Spelled(String),
    /// The one type body a member sits in, where lexical nesting is the
    /// only owner (Java — LangSpec::owner_kinds). A class has exactly
    /// one body, so the body alone tells an anonymous class, an enum
    /// constant's body and a same-named local class from every other —
    /// none of which a spelled owner can tell apart.
    Body(usize),
    /// The object a Lua or R member-shaped name is selected off (`M` of
    /// `M.f` and `M:f`, `x` of `x$f`; scan/binding.rs). A table is no
    /// class: inside `M.a` a bare `b()` is whichever `b` the block
    /// sees, never `M.b`, so this owner keys the member road alone.
    Table(String),
}

impl Owner {
    /// Whether the owner is a class, whose own members a bare call
    /// inside one of them reaches before any enclosing scope (C++,
    /// Java). A Lua or R table is not.
    pub(super) fn is_class(&self) -> bool {
        !matches!(self, Owner::Table(_))
    }
}

/// A unit's base-road owner (see Owner): None for a free callable and
/// for every grammar that has no owner road. Where the owner is a
/// spelling it is the one functions::owner_of reads, and only its kind
/// is decided here: the object a member-shaped name is selected off
/// is a table, a declarator's class a class.
pub(super) fn owner_key(node: Node<'_>, src: &[u8], spec: &LangSpec) -> Option<Owner> {
    if !spec.owner_kinds.is_empty() {
        return in_member_scope(node, spec).then(|| Owner::Body(container_of(node, src)));
    }
    let owner = functions::owner_of(node, src, spec)?;
    Some(match binding::member_of(node, src) {
        Some(_) => Owner::Table(owner),
        None => Owner::Spelled(owner),
    })
}

/// The argument counts a callable accepts, `(fewest, most)` with
/// `most` None = no upper bound (functions::arity).
type Range = (usize, Option<usize>);

pub(super) struct Named {
    pub(super) groups: Vec<Vec<usize>>,
    pub(super) containers: Vec<usize>,
    /// The groups each key names — its seats, narrowed by `pick`.
    pub(super) whole: HashMap<String, Vec<usize>>,
    pub(super) base: HashMap<BaseKey, Vec<usize>>,
    ranges: Vec<Range>,
    /// Where each group's name becomes visible (Lua `local`), None =
    /// throughout its container.
    from: Vec<Option<usize>>,
    overloads: Option<&'static Overloads>,
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
/// keyed by: its parent, or, for a Lua or R function value, the scope
/// its binding lives in (scan/binding.rs) — the statement around the
/// value is no scope a call can sit in. Every unit sits inside its
/// file, so the 0 a parentless node would answer is a value no
/// container ever takes, and calls::sees answers false for it: the
/// safe direction.
pub(super) fn container_of(node: Node<'_>, src: &[u8]) -> usize {
    match binding::of(node, src) {
        Some(b) => b.scope.id(),
        None => node.parent().map_or(0, |p| p.id()),
    }
}

impl Named {
    pub(super) fn of(units: &[FnUnit<'_>], src: &[u8], spec: &LangSpec) -> Self {
        let mut named = Named {
            groups: Vec::new(),
            containers: Vec::new(),
            whole: HashMap::new(),
            base: HashMap::new(),
            ranges: Vec::new(),
            from: Vec::new(),
            overloads: spec.overloads,
        };
        let unreachable = spec.overloads.map_or(&[][..], |o| o.unreachable);
        let mut seat: HashMap<(usize, &str), usize> = HashMap::new();
        for (i, unit) in units.iter().enumerate() {
            if unreachable.contains(&unit.node.kind()) {
                continue;
            }
            let key = (container_of(unit.node, src), unit.name.as_str());
            match seat.get(&key) {
                Some(&g) if spec.overloads.is_none() => named.groups[g].push(i),
                _ => {
                    seat.insert(key, named.groups.len());
                    named.seat(unit, i, src, spec);
                }
            }
        }
        named
    }

    /// A new callable for `unit`: its group, container, range and
    /// visibility, and the keys it answers to. A member is absent from
    /// the bare road entirely, so a top-level name is not cancelled by a
    /// method spelling it.
    fn seat(&mut self, unit: &FnUnit<'_>, i: usize, src: &[u8], spec: &LangSpec) {
        let group = self.groups.len();
        self.groups.push(vec![i]);
        self.containers.push(container_of(unit.node, src));
        self.ranges.push(functions::arity(unit.node, src, spec));
        self.from
            .push(binding::of(unit.node, src).and_then(|b| b.from));
        if !in_member_scope(unit.node, spec) {
            self.whole.entry(unit.name.clone()).or_default().push(group);
        }
        let owner = owner_key(unit.node, src, spec);
        let base = binding::member_of(unit.node, src)
            .map_or_else(|| base_name(&unit.name).to_string(), |(_, member)| member);
        self.base.entry((owner, base)).or_default().push(group);
    }

    /// Whether `call` sits where the group's name is visible: past a Lua
    /// local's declaration (scan/binding.rs). Every other name is
    /// visible throughout its container.
    pub(super) fn visible(&self, group: usize, call: Node<'_>) -> bool {
        self.from[group].is_none_or(|at| call.start_byte() >= at)
    }

    /// The one group among `seats` a call reaches: those whose range
    /// admits the call's argument count, when exactly one does. Where
    /// the grammar does not overload every range admits every call, so
    /// a key names one callable or none — two callables spelling one
    /// key cancel out.
    pub(super) fn pick(&self, seats: &[usize], call: Node<'_>) -> Option<usize> {
        let args = self.overloads.and_then(|o| arguments(call, o));
        let mut fit = seats
            .iter()
            .copied()
            .filter(|&g| admits(self.ranges[g], args));
        let first = fit.next()?;
        fit.next().is_none().then_some(first)
    }
}

/// Whether a range admits a call passing `args` arguments; an unknown
/// count admits every range.
fn admits((fewest, most): Range, args: Option<usize>) -> bool {
    args.is_none_or(|n| fewest <= n && most.is_none_or(|m| n <= m))
}

/// How many arguments a call passes — its `arguments` list's entries —
/// or None when no reader can tell: no list, or an entry spreading an
/// unknown count (Overloads::spread).
fn arguments(call: Node<'_>, overloads: &Overloads) -> Option<usize> {
    let args = ast::entries(call.child_by_field_name("arguments")?);
    let spread = args.iter().any(|a| overloads.spread.contains(&a.kind()));
    (!spread).then_some(args.len())
}

/// `(T) add` → `add` — the receiver qualification functions::name_of
/// puts on Go methods — and `K::b` → `b`, the class qualification it
/// puts on C++ members. A Lua or R member-shaped name splits off the
/// AST instead (binding::member_of): an R name may hold dots of its
/// own. Every other spelling is its own base.
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
