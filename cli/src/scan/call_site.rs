//! Reference sites a CALL opens (plan v2.30 step 4; booklet §8): Lua
//! loads a module by calling `require "a.b"` and runs a file with
//! `dofile` / `loadfile`, R sources a file with `source("x.R")` and
//! attaches a package with `library(pkg)` — no import statement exists
//! in either language. A call is a site only when its callee is spelled
//! as one of a row's names (scan/call_table.rs CallSite) — bare, or
//! qualified by the row's package (`base::source("x.R")`, plan v2.30
//! step 5b; the `base::` opens its own `library` site beside it) — and
//! the argument naming the target is literal text. A computed argument
//! (`require(prefix .. name)`) holds no target in the text — naming what
//! it would load is a guess (the ladder's rule) — so it opens a site of
//! the row's `computed` kind instead (`dynamic`, lane v235_graphtrust):
//! the resolve family refuses it as dynamic, and the unresolved ledger
//! keeps the language from vouching for a file only such a call reaches.
//! Python's `importlib.import_module` / `__import__` and TypeScript's
//! `require` / `import(…)` read the same way beside their statements.
//! A protected call is the call it protects: Lua's `pcall(require,
//! "a.b")` loads `a.b` (CE.Lang.Lua `protected`, the package's
//! `calls.protected`).

use super::ast;
use super::call_table::{CallSite, calls, formals, protected};
use super::lang::Lang;
use super::spec::LangSpec;
use tree_sitter::Node;

/// The site `node` opens as a call: its row's label, the node it sits on
/// — the argument, whose line holds the spec even when the call spans
/// several (the Python import target's precedent) — and the spec. The
/// call is one the grammar spells (LangSpec::call_kinds), its callee
/// read through LangSpec::call_fields (`callee`) and one of a row's
/// `callees`; a protected call is read as the call it protects
/// (`unprotected`). R matches a named argument before a positional one
/// — the name exactly, or as a prefix no other formal before `...`
/// shares (`formal_matches`) — so the argument naming the callee's
/// first formal wins over the first unnamed one; a Lua argument list
/// has no names. The argument must be a string literal — its content,
/// so R's raw `r"(x.R)"` and Lua's long `[[x.lua]]` read like any quoted
/// one, cut at a line break like every spec — or an identifier where
/// the callee reads one unevaluated (`unquoted` is Some: R's
/// `library(pkg)`) and the call does not pass that flag as anything but
/// a literal `FALSE` (`library(p, character.only = TRUE)` loads whatever
/// `p` holds). Any other argument opens the row's `computed` site, its
/// spec the argument's first line, or none when the row names no kind.
pub(crate) fn site<'t>(
    node: Node<'t>,
    src: &[u8],
    lang: Lang,
) -> Option<(&'static str, Node<'t>, String)> {
    let (rows, grammar) = (calls(lang), crate::scan::spec::spec(lang));
    if rows.is_empty() || !grammar.call_kinds.contains(&node.kind()) {
        return None;
    }
    let (package, name) = callee(
        node.child_by_field_name(grammar.call_fields.0)?,
        grammar,
        src,
    )?;
    let args = ast::entries(node.child_by_field_name("arguments")?);
    let (name, args) = unprotected(name, &args, lang, src)?;
    let row = rows
        .iter()
        .find(|r| r.callees.contains(&name) && package.is_none_or(|p| p == r.package))?;
    let formals = formals(name);
    let value = target_arg(args, formals, src)?;
    let first = |text: &str| text.split('\n').next().unwrap_or("").to_string();
    match literal(value, reads_names(args, row, formals, src), src) {
        Some(text) => Some((row.label, value, first(text))),
        None => Some((
            row.computed?,
            value,
            first(value.utf8_text(src).ok()?).trim().to_string(),
        )),
    }
}

/// Whether `node` is a call the call-site table opens a site for — the
/// load a load statement holds (scan/loads.rs). One reading with the
/// graph's call sites (graph/sites.rs reads `site`).
pub(crate) fn opens_site(node: Node<'_>, src: &[u8], lang: Lang) -> bool {
    site(node, src, lang).is_some()
}

/// The argument naming the target, unwrapped to its value: the one
/// addressing the callee's first formal by name (`formal_is`), else
/// the first unnamed one — R's positional matching after the named
/// arguments are taken; a Lua list has no names.
fn target_arg<'t>(args: &[Node<'t>], formals: &[&str], src: &[u8]) -> Option<Node<'t>> {
    let target = formals.first().copied().unwrap_or("");
    let arg = args
        .iter()
        .find(|a| formal_is(a, target, formals, src))
        .or_else(|| args.iter().find(|a| formal_of(a).is_none()))?;
    if arg.kind() == "argument" {
        arg.child_by_field_name("value")
    } else {
        Some(*arg)
    }
}

/// The text a value node holds as a target: a string's content (`""`
/// when the literal has none), or an identifier's spelling where the
/// call reads names (`reads_name`); any other shape names no target.
fn literal<'a>(value: Node<'_>, reads_name: bool, src: &'a [u8]) -> Option<&'a str> {
    match value.kind() {
        "string" => content(value, src),
        "identifier" if reads_name => value.utf8_text(src).ok(),
        _ => None,
    }
}

/// A string literal's content: its `content` field (Lua, R), else what
/// sits between the delimiters a grammar spells as children of their own
/// (Python's `string_start` / `string_end`, TypeScript's quotes) — none
/// is `""`, one leaf is its text, and anything else (an interpolation, an
/// escape beside the text) is no literal.
fn content<'a>(value: Node<'_>, src: &'a [u8]) -> Option<&'a str> {
    if let Some(c) = value.child_by_field_name("content") {
        return c.utf8_text(src).ok();
    }
    let kids = ast::children(value);
    match kids.get(1..kids.len().saturating_sub(1)).unwrap_or(&[]) {
        [] => Some(""),
        [leaf] if leaf.named_child_count() == 0 => leaf.utf8_text(src).ok(),
        _ => None,
    }
}

/// A callee's package qualifier and bare name: a node of the kinds the
/// recursion arcs read (LangSpec::call_name_kinds) is an unqualified
/// name, and so is a keyword the grammar spells as a node of its own
/// kind (TypeScript's `import(…)`); a member of the kinds those arcs
/// read (LangSpec::call_member_kinds) over two such names — R's
/// `pkg::name`, Python's `importlib.import_module` — is the member under
/// the object. The row's `package` then decides: `m.require` names no
/// row whose package is not `m`. Any other shape (`f()()`) names none.
fn callee<'a>(
    callee: Node<'_>,
    grammar: &LangSpec,
    src: &'a [u8],
) -> Option<(Option<&'a str>, &'a str)> {
    let text = callee.utf8_text(src).ok()?;
    if grammar.call_name_kinds.contains(&callee.kind()) || callee.kind() == text {
        return Some((None, text));
    }
    if !grammar.call_member_kinds.contains(&callee.kind()) {
        return None;
    }
    let kids = ast::named_children(callee);
    let word = |node: &Node<'_>| -> Option<&'a str> {
        grammar
            .call_name_kinds
            .contains(&node.kind())
            .then(|| node.utf8_text(src).ok())
            .flatten()
    };
    let (object, member) = (kids.first()?, kids.last()?);
    (object.id() != member.id()).then_some(())?;
    Some((Some(word(object)?), word(member)?))
}

/// A call's callee name and arguments, a protected call's being those
/// of the call it protects (spec::protected): the wrapper's first
/// argument is the function, its text matched against the rows' bare
/// names like any callee's (`pcall(m.require, "x")` spells none), and
/// the function's own arguments follow the wrapper's leading ones —
/// `pcall(require, "x")` is `require("x")`.
fn unprotected<'a, 't>(
    name: &'a str,
    args: &'a [Node<'t>],
    lang: Lang,
    src: &'a [u8],
) -> Option<(&'a str, &'a [Node<'t>])> {
    let Some((_, leading)) = protected(lang).iter().find(|(w, _)| *w == name) else {
        return Some((name, args));
    };
    Some((args.first()?.utf8_text(src).ok()?, args.get(*leading..)?))
}

/// Whether the call reads an identifier argument as a name: the callee
/// reads one unevaluated (`unquoted` is Some) and no argument passes
/// that flag as anything but a literal `FALSE`.
fn reads_names(args: &[Node<'_>], row: &CallSite, formals: &[&str], src: &[u8]) -> bool {
    row.unquoted.is_some_and(|flag| {
        !args.iter().any(|a| {
            formal_is(a, flag, formals, src)
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

/// Whether the argument is passed under a name that lands on `formal`.
fn formal_is(arg: &Node<'_>, formal: &str, formals: &[&str], src: &[u8]) -> bool {
    formal_of(arg).is_some_and(|n| {
        n.utf8_text(src)
            .is_ok_and(|given| formal_matches(given, formal, formals))
    })
}

/// Whether a name lands on `formal` the way R matches argument names
/// (R Language Definition §4.3.2): exactly, or as a prefix of `formal`
/// that no other formal before `...` shares — `source(fi = "x.R")`
/// names `file`; `source(e = "x.R")` names nothing, since `echo`,
/// `exprs` and `encoding` all begin so (R itself refuses the call). A
/// Lua row lists no formals and a Lua argument carries no name, so the
/// question never arises there.
fn formal_matches(given: &str, formal: &str, formals: &[&str]) -> bool {
    given == formal
        || (formal.starts_with(given)
            && formals.iter().filter(|f| f.starts_with(given)).count() == 1)
}

#[cfg(test)]
#[path = "../../tests/unit/scan/call_site.rs"]
mod tests;
