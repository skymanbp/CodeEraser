//! The statement class of the merge family's slot tables read off a
//! flow table (plan v2.31 step 7; design booklet §6.2), beside the
//! contract in slot.rs: every FlowSpec field that names a statement
//! form, and scan's if kinds. Split from slot.rs when the second
//! generation's target and part sets pushed it past the 300-line line.

use crate::scan::lang::Lang;
use std::collections::HashSet;

/// The statement forms a flow table decides (and scan's if kinds).
/// Every field naming a statement kind is read; the fields that name an
/// expression, a part or a mix of both are not — `assigns` and
/// `update_kinds` (an expression in most grammars), `macros`, and
/// `dynamic_kinds` (C's `#if` beside `gnu_asm_expression`), whose
/// statement kinds sit in a container and are held there anyway.
pub(super) fn flow_statements(
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
