//! The position-class tables of the merge family (plan v2.31 step 7;
//! design booklet docs/reference/analysis-track.md §6.2): the `slot`
//! column merge/1 reads beside each tree node — a statement (0), an
//! expression (1), a type (2), a declared name (3) or anything else
//! (4). Read beside the flow tables (flow/spec.rs): the statement
//! class is read off them and never restated: a node whose kind is one of
//! the table's statement forms, or whose parent is one of its
//! containers, stands in a statement position. What the flow tables do
//! not say is said here, one table per language: the expression kinds,
//! the type kinds, the (declaring kind, field) pairs where a declared
//! name sits, and — named, so the coverage leg can tell a decision from
//! an omission — the kinds that are "other". A kind in no set is also
//! other: an unknown position can hold no parameter, the safe side. A
//! language with no flow table (Haskell, step-7 ruling 8) names its own
//! statement forms and containers here; a language with one never does.
//!
//! A table is TOML text in pieces (TSX is TypeScript's plus the JSX
//! kinds, C++ is C's plus its own); each piece reads alone and the
//! pieces' lists join, so a shared piece never names a kind the other
//! grammar lacks.

use super::slot_c::{C, C_ONLY, CPP};
use super::slot_hs::HASKELL;
use super::slot_java::JAVA;
use super::slot_launch::{GO, PYTHON, RUST};
use super::slot_lua::LUA;
use super::slot_r::R;
use super::slot_ts::{JSX, TS_ONLY, TYPESCRIPT};
use crate::scan::lang::Lang;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use tree_sitter::Node;

/// The four sets a table holds (§6.2's position classes, less the
/// statement class the flow tables own).
#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SlotSpec {
    /// Kinds in an expression position: a parameter can stand there.
    pub expr_kinds: Vec<String>,
    /// Kinds in a type position.
    pub type_kinds: Vec<String>,
    /// (declaring kind, field) where the declared name sits: a
    /// function's, a class's, a variable's, a parameter's.
    pub name_fields: Vec<(String, String)>,
    /// Kinds named "other": containers of no statement, lists,
    /// modifiers, comments, imports, the unit's own definition.
    pub other_kinds: Vec<String>,
    /// A language without a flow table: its statement forms (empty
    /// where the flow table owns them).
    pub stmt_kinds: Vec<String>,
    /// A language without a flow table: the kinds whose children stand
    /// in a statement position (empty where the flow table owns them).
    pub container_kinds: Vec<String>,
}

/// The pieces of every table, by the language it answers.
const TABLES: [(Lang, &[&str]); 11] = [
    (Lang::Python, &[PYTHON]),
    (Lang::TypeScript, &[TYPESCRIPT, TS_ONLY]),
    (Lang::Tsx, &[TYPESCRIPT, JSX]),
    (Lang::Rust, &[RUST]),
    (Lang::Go, &[GO]),
    (Lang::C, &[C, C_ONLY]),
    (Lang::Cpp, &[C, CPP]),
    (Lang::Java, &[JAVA]),
    (Lang::Lua, &[LUA]),
    (Lang::R, &[R]),
    (Lang::Haskell, &[HASKELL]),
];

/// Every table read once, with its classes compiled. The text is this
/// crate's own: a piece that does not read is a build defect the unit
/// legs name.
static READ: LazyLock<HashMap<Lang, SlotSpec>> = LazyLock::new(|| {
    TABLES
        .iter()
        .map(|(lang, pieces)| (*lang, pieces.iter().fold(SlotSpec::default(), join)))
        .collect()
});

fn join(mut acc: SlotSpec, piece: &&str) -> SlotSpec {
    let more: SlotSpec =
        toml::from_str(piece).unwrap_or_else(|e| panic!("slot table piece does not read: {e}"));
    acc.expr_kinds.extend(more.expr_kinds);
    acc.type_kinds.extend(more.type_kinds);
    acc.name_fields.extend(more.name_fields);
    acc.other_kinds.extend(more.other_kinds);
    acc.stmt_kinds.extend(more.stmt_kinds);
    acc.container_kinds.extend(more.container_kinds);
    acc
}

/// A language's slot table: the flow family's languages, whose
/// statement class the flow tables give, and Haskell, whose table
/// gives its own; Markdown and HTML have none (no function to fold).
pub fn slot_spec(lang: Lang) -> Option<&'static SlotSpec> {
    READ.get(&lang)
}

/// One language's classes as sets: the statement forms and containers
/// read off its flow table (and scan's if kinds) — or, with no flow
/// table, off its slot table's own two lists — the rest off its slot
/// table.
pub struct Classes {
    statement: HashSet<&'static str>,
    container: HashSet<&'static str>,
    types: HashSet<&'static str>,
    exprs: HashSet<&'static str>,
    names: HashSet<(&'static str, &'static str)>,
}

static CLASSES: LazyLock<HashMap<Lang, Classes>> = LazyLock::new(|| {
    TABLES
        .iter()
        .filter_map(|(lang, _)| Some((*lang, compile(*lang)?)))
        .collect()
});

/// A language's classes, or None where it has no slot table.
pub fn classes(lang: Lang) -> Option<&'static Classes> {
    CLASSES.get(&lang)
}

fn compile(lang: Lang) -> Option<Classes> {
    let slots = slot_spec(lang)?;
    let set = |v: &'static [String]| v.iter().map(String::as_str);
    let (statement, container) = match crate::flow::spec::spec(lang) {
        Some(flow) => (
            flow_statements(lang, flow),
            set(&flow.block_kinds)
                .chain(set(&flow.splice_kinds))
                .collect(),
        ),
        None => (
            set(&slots.stmt_kinds).collect(),
            set(&slots.container_kinds).collect(),
        ),
    };
    Some(Classes {
        statement,
        container,
        types: set(&slots.type_kinds).collect(),
        exprs: set(&slots.expr_kinds).collect(),
        names: slots
            .name_fields
            .iter()
            .map(|(k, f)| (k.as_str(), f.as_str()))
            .collect(),
    })
}

/// The statement forms a flow table decides (and scan's if kinds).
/// Every field naming a statement kind is read; the fields that name an
/// expression, a part or a mix of both are not — `assigns` and
/// `update_kinds` (an expression in most grammars), `macros`, and
/// `dynamic_kinds` (C's `#if` beside `gnu_asm_expression`), whose
/// statement kinds sit in a container and are held there anyway.
fn flow_statements(
    lang: Lang,
    flow: &'static crate::flow::spec::FlowSpec,
) -> HashSet<&'static str> {
    let set = |v: &'static [String]| v.iter().map(String::as_str);
    let firsts = |v: &'static [(String, String)]| v.iter().map(|(k, _)| k.as_str());
    let mut statement: HashSet<&'static str> = crate::scan::spec::spec(lang)
        .if_kinds
        .iter()
        .copied()
        .collect();
    statement.extend(flow.loops.iter().map(|r| r.kind.as_str()));
    statement.extend(flow.switches.iter().map(|r| r.kind.as_str()));
    statement.extend(flow.cases.iter().map(|r| r.kind.as_str()));
    statement.extend(flow.tries.iter().map(|r| r.kind.as_str()));
    statement.extend(flow.catches.iter().map(|r| r.kind.as_str()));
    statement.extend(flow.withs.iter().map(|r| r.kind.as_str()));
    statement.extend(flow.decls.iter().map(|r| r.kind.as_str()));
    for kinds in [
        &flow.return_kinds,
        &flow.throw_kinds,
        &flow.break_kinds,
        &flow.continue_kinds,
        &flow.yield_kinds,
        &flow.fallthrough_kinds,
        &flow.nonlocal_kinds,
        &flow.empty_kinds,
        &flow.wrapper_kinds,
        &flow.elif_kinds,
    ] {
        statement.extend(set(kinds));
    }
    for pairs in [&flow.finally_kinds, &flow.gotos, &flow.else_kinds] {
        statement.extend(firsts(pairs));
    }
    statement.extend(flow.labels.iter().map(|(k, _, _)| k.as_str()));
    statement
}

/// A node's position class (§6.2, step-7 ruling 1): a statement form,
/// or any node a container holds, is 0; else a type kind 2; else a
/// declared name 3; else an expression kind 1; else 4.
pub fn slot_of(c: &Classes, node: Node) -> u8 {
    let parent = node.parent();
    let held = parent.is_some_and(|p| c.container.contains(p.kind()));
    if held || c.statement.contains(node.kind()) {
        0
    } else if c.types.contains(node.kind()) {
        2
    } else if parent
        .is_some_and(|p| field_of(p, node).is_some_and(|f| c.names.contains(&(p.kind(), f))))
    {
        3
    } else if c.exprs.contains(node.kind()) {
        1
    } else {
        4
    }
}

/// The field the parent holds this child on, if any.
fn field_of<'t>(parent: Node<'t>, child: Node<'t>) -> Option<&'t str> {
    let mut cursor = parent.walk();
    if !cursor.goto_first_child() {
        return None;
    }
    loop {
        if cursor.node() == child {
            return cursor.field_name();
        }
        if !cursor.goto_next_sibling() {
            return None;
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/merge/slot.rs"]
mod tests;
