//! Reference sites a CALL opens (plan v2.30 step 4; booklet §8): Lua
//! loads a module by calling `require "a.b"` and runs a file with
//! `dofile` / `loadfile`, R sources a file with `source("x.R")` and
//! attaches a package with `library(pkg)` — no import statement exists
//! in either language. A call is a site only when its callee is spelled
//! as one of a row's bare names (graph/spec.rs CallSite) and the
//! argument naming the target is literal text: a computed argument
//! (`require(prefix .. name)`) holds no target in the text, so it opens
//! no site — naming what it would load is a guess (the ladder's rule).
//! A qualified callee (`base::source("x.R")`) is no site either: the
//! form is rare, and the package it names is a `library` site of its
//! own. A protected call is the call it protects: Lua's
//! `pcall(require, "a.b")` loads `a.b` (graph/spec.rs LUA_PROTECTED).

use crate::graph::spec::{calls, protected};
use crate::scan::ast;
use crate::scan::lang::Lang;
use tree_sitter::Node;

/// The site `node` opens as a call: its row's label, the node it sits on
/// — the argument, whose line holds the spec even when the call spans
/// several (the Python import target's precedent) — and the spec. The
/// call is one the grammar spells (LangSpec::call_kinds), its callee
/// read through LangSpec::call_fields as a bare name
/// (call_name_kinds) — the recursion arcs' own reading — and one of
/// the row's `callees`; a protected call is read as the call it
/// protects (`unprotected`). R matches a named argument before a
/// positional one, so the argument named `formal` wins over the first
/// unnamed one; a Lua argument list has no names. The argument must be
/// a string literal — its content, so R's raw `r"(x.R)"` and Lua's long
/// `[[x.lua]]` read like any quoted one, cut at a line break like every
/// spec — or an identifier where the callee reads one unevaluated
/// (`unquoted` is Some: R's `library(pkg)`) and the call does not pass
/// that flag as anything but a literal `FALSE` (`library(p,
/// character.only = TRUE)` loads whatever `p` holds).
pub(super) fn site<'t>(
    node: Node<'t>,
    src: &[u8],
    lang: Lang,
) -> Option<(&'static str, Node<'t>, String)> {
    let (rows, grammar) = (calls(lang), crate::scan::spec::spec(lang));
    if rows.is_empty() || !grammar.call_kinds.contains(&node.kind()) {
        return None;
    }
    let callee = node
        .child_by_field_name(grammar.call_fields.0)
        .filter(|c| grammar.call_name_kinds.contains(&c.kind()))?;
    let args = ast::entries(node.child_by_field_name("arguments")?);
    let (name, args) = unprotected(callee, &args, lang, src)?;
    let row = rows.iter().find(|r| r.callees.contains(&name))?;
    let arg = args
        .iter()
        .find(|a| formal_is(a, row.formal, src))
        .or_else(|| args.iter().find(|a| formal_of(a).is_none()))?;
    let value = if arg.kind() == "argument" {
        arg.child_by_field_name("value")?
    } else {
        *arg
    };
    let text = match value.kind() {
        "string" => value
            .child_by_field_name("content")
            .map_or(Some(""), |c| c.utf8_text(src).ok())?,
        "identifier" if reads_names(args, row.unquoted, src) => value.utf8_text(src).ok()?,
        _ => return None,
    };
    let spec = text.split('\n').next().unwrap_or("").to_string();
    Some((row.label, value, spec))
}

/// A call's callee name and arguments, a protected call's being those
/// of the call it protects (spec::protected): the wrapper's first
/// argument is the function, its text matched against the rows' bare
/// names like any callee's (`pcall(m.require, "x")` spells none), and
/// the function's own arguments follow the wrapper's leading ones —
/// `pcall(require, "x")` is `require("x")`.
fn unprotected<'a, 't>(
    callee: Node<'t>,
    args: &'a [Node<'t>],
    lang: Lang,
    src: &'a [u8],
) -> Option<(&'a str, &'a [Node<'t>])> {
    let name = callee.utf8_text(src).ok()?;
    let Some((_, leading)) = protected(lang).iter().find(|(w, _)| *w == name) else {
        return Some((name, args));
    };
    Some((args.first()?.utf8_text(src).ok()?, args.get(*leading..)?))
}

/// Whether the call reads an identifier argument as a name: the callee
/// reads one unevaluated (`unquoted` is Some) and no argument passes
/// that flag as anything but a literal `FALSE`.
fn reads_names(args: &[Node<'_>], unquoted: Option<&str>, src: &[u8]) -> bool {
    unquoted.is_some_and(|flag| {
        !args.iter().any(|a| {
            formal_is(a, flag, src)
                && a.child_by_field_name("value")
                    .is_none_or(|v| v.kind() != "false")
        })
    })
}

/// The name an R argument is passed under (`file = "x.R"`); a Lua
/// argument list, and a positional R argument, has none.
fn formal_of<'t>(arg: &Node<'t>) -> Option<Node<'t>> {
    (arg.kind() == "argument")
        .then(|| arg.child_by_field_name("name"))
        .flatten()
}

fn formal_is(arg: &Node<'_>, name: &str, src: &[u8]) -> bool {
    formal_of(arg).is_some_and(|n| n.utf8_text(src) == Ok(name))
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/sites/call.rs"]
mod tests;
