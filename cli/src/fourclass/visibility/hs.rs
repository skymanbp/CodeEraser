//! Haskell: a `module M (a, b) where` header exports exactly its list,
//! and a header without one exports everything — everything TOP-LEVEL
//! (plan v2.17 L round piece (2), criteria H2/H4): a `where`-bound
//! helper that shares a name with an exported binding is not exported
//! by that name, and class default bodies and instance bodies are
//! members of their class, not module bindings. No header at all is
//! the abbreviated `module Main(main) where` (Haskell 2010 §5.1), read
//! the way GHC reads it (`headerless_exports`). The list is read from
//! the file's own header, lexed by hs_lex.rs, and no import is
//! followed.

use super::{ancestors, hs_lex, name_text, root_of, text};
use crate::scan::ast;
use tree_sitter::Node;

/// Kinds that close the module scope: a binding under any of them is
/// local to an equation, a `let`, an instance or a class.
const SCOPED: [&str; 6] = [
    "function",
    "bind",
    "local_binds",
    "let",
    "instance",
    "class",
];

pub(super) fn exported(node: Node<'_>, src: &[u8]) -> bool {
    let Some(name) = name_text(node, src) else {
        return false;
    };
    let owner = associated_class(node, src);
    if owner.is_none() && ancestors(node).any(|a| SCOPED.contains(&a.kind())) {
        return false;
    }
    let root = root_of(node);
    let Some(header) = ast::named_children(root)
        .into_iter()
        .find(|c| c.kind() == "header")
    else {
        return headerless_exports(root, src, &name);
    };
    let Some(list) = header.child_by_field_name("exports") else {
        return true;
    };
    let module = header.child_by_field_name("module").map(|m| text(m, src));
    hs_lex::entries(&hs_lex::strip_comments(&text(list, src)))
        .iter()
        .any(|entry| {
            names(entry, &name, module.as_deref())
                || owner
                    .as_deref()
                    .is_some_and(|class| member_entry(entry, class, &name))
        })
}

/// The class an associated `type`/`data` family is declared in — the
/// one class member that is a unit (a method signature is none, a
/// default body is a member binding): GHC exports an associated
/// family with its class, through `C(..)` or a listed `C(Fam)`, so
/// the class entry speaks for it (the step-8 review's counterexample
/// read every such family as private).
fn associated_class(node: Node<'_>, src: &[u8]) -> Option<String> {
    if !matches!(node.kind(), "type_family" | "data_family") {
        return None;
    }
    let class = ancestors(node).find(|a| a.kind() == "class")?;
    name_text(class, src)
}

/// `C(..)` names every member of C; `C(Fam, m)` names the listed
/// ones (an ExplicitNamespaces `type Fam` inside the list too).
fn member_entry(entry: &str, class: &str, member: &str) -> bool {
    let entry = entry.trim();
    let Some(open) = entry.find('(') else {
        return false;
    };
    if head(entry) != class || !entry.ends_with(')') {
        return false;
    }
    let inner = &entry[open + 1..entry.len() - 1];
    inner.trim() == ".."
        || inner
            .split(',')
            .any(|m| m.trim().trim_start_matches("type ").trim() == member)
}

/// The abbreviated header `module Main(main) where`, as GHC 9.10.3
/// applies it: a header-less file WITH a top-level `main` exports
/// `main` alone (`import Main (helper)` is refused, GHC-61689), and one
/// WITHOUT exports every top-level binding (the same import is
/// accepted) — ruled by the compiler, not read off the report.
fn headerless_exports(root: Node<'_>, src: &[u8], name: &str) -> bool {
    name == "main" || !top_level_bindings(root, src).any(|n| n == "main")
}

fn top_level_bindings<'a>(root: Node<'a>, src: &'a [u8]) -> impl Iterator<Item = String> + 'a {
    root.child_by_field_name("declarations")
        .into_iter()
        .flat_map(ast::named_children)
        .filter(|d| matches!(d.kind(), "function" | "bind"))
        .filter_map(move |d| name_text(d, src))
}

/// One export entry names the binding when it is the name itself —
/// bare, in operator parens, or qualified by THIS module's own name
/// (`M.foo` inside `module M` is the local `foo`; `Q.foo` under a
/// foreign qualifier re-exports an import and never names a local
/// binding) — or when it is `module M` for M this very module, which
/// re-exports everything M defines. Layout is inert inside the list,
/// so any whitespace may separate the keyword from the module name. A
/// `T(..)` / `C(m)` entry names the type or class T / C (a unit since
/// L round step 8) and never a top-level binding called `m`; an
/// ExplicitNamespaces `type T` / `pattern P` entry names T / P.
fn names(entry: &str, name: &str, module: Option<&str>) -> bool {
    let mut words = entry.split_whitespace();
    if words.next() == Some("module") {
        return module.is_some() && words.next() == module && words.next().is_none();
    }
    let bare = unparen(head(entry));
    let local = module
        .and_then(|m| bare.strip_prefix(m)?.strip_prefix('.'))
        .unwrap_or(bare);
    local == unparen(name)
}

/// The entry's own name: a namespace keyword dropped, a trailing
/// member list (`T(..)`, `C(m, n)`) cut. An operator entry opens with
/// its paren and closes at the first `)` — parens are Haskell
/// `special`, never part of a symbol — so a type operator's
/// `(:^:)(..)` cuts to `(:^:)` and `(<+>)` stays whole for `unparen`
/// (the step-8 review: the member list used to stay attached and an
/// exported type operator read private).
fn head(entry: &str) -> &str {
    let entry = entry.trim();
    let entry = ["type ", "pattern "]
        .iter()
        .find_map(|kw| entry.strip_prefix(kw))
        .unwrap_or(entry)
        .trim_start();
    let cut = if entry.starts_with('(') {
        entry.find(')').map(|i| i + 1)
    } else {
        entry.find('(')
    };
    cut.map_or(entry, |i| &entry[..i]).trim_end()
}

fn unparen(s: &str) -> &str {
    let s = s.trim();
    s.strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
        .map_or(s, str::trim)
}
