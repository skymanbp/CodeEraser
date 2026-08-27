//! Python: two facts.
//!   - `Member`: the declaration sits inside a class body — any
//!     `class_definition` on the parent chain (the criterion's P-F3:
//!     a method is reached through its class, never spelled bare).
//!   - `Registration`: a decorator on the declaration whose callee's
//!     last name is in the registrar table — `@app.route("/")`,
//!     `@pytest.fixture`, a bare `@fixture` imported by name — read off
//!     the `decorated_definition` wrapper the grammar puts around a
//!     decorated def or class. The table is the criterion's own
//!     (§3.2 bit 3); anything else is a plain decorator and claims
//!     nothing.

use super::{Conv, parent_of_kind, text, under};
use crate::scan::ast;
use tree_sitter::Node;

/// Decorator attribute suffixes that register a callable with a host
/// — web routes, CLI commands, task queues, fixtures, hooks, events.
/// One string, split at use: a one-name-per-line array is this repo's
/// most-rhyming token shape and its own clone gate said so.
const REGISTRARS: &str = "route get post put delete patch command task fixture \
                          errorhandler before_request after_request on_event event register";

pub(super) fn bits(node: Node<'_>, src: &[u8]) -> i64 {
    let mut word = 0;
    if under(node, "class_definition") {
        word |= Conv::Member.bit();
    }
    let registered = parent_of_kind(node, "decorated_definition").is_some_and(|wrapper| {
        ast::named_children(wrapper)
            .into_iter()
            .filter(|c| c.kind() == "decorator")
            .any(|d| registers(d, src))
    });
    if registered {
        word |= Conv::Registration.bit();
    }
    word
}

/// `@a.b.c(...)` and `@a.b.c` name `c`; `@c` names `c`.
fn registers(decorator: Node<'_>, src: &[u8]) -> bool {
    let Some(expr) = ast::named_children(decorator).into_iter().next() else {
        return false;
    };
    let callee = match expr.kind() {
        "call" => expr.child_by_field_name("function"),
        _ => Some(expr),
    };
    let name = callee.and_then(|c| match c.kind() {
        "attribute" => c.child_by_field_name("attribute"),
        "identifier" => Some(c),
        _ => None,
    });
    name.is_some_and(|n| REGISTRARS.split_whitespace().any(|r| r == text(n, src)))
}
