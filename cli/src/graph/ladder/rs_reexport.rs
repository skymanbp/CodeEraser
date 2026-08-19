//! The Rust re-export surface (§4 R5 as amended 2026-08-18: `pub
//! use` binds ≤1 hop to the DEFINITION file). This module owns the
//! surface FACTS of one file — which names its top-level pub uses
//! bind, and to which full paths; the hop itself runs in rs.rs
//! through the bind-free walk, so the ≤1 bound is structural.
//! Refusals are the honest half: `use_wildcard` binds an unknown
//! name set (never followed), `pub extern crate` is a different
//! grammar node and stays invisible (a chained façade terminates at
//! the middle file), a name F defines ITSELF at top level never
//! hops (definition wins), and >1 matching entry keeps the
//! file-level edge (picking would invent a path).

use super::Scope;
use super::rs_tree::cached_tree;
use std::rc::Rc;

/// The full path a single unambiguous top-level `pub use` of `f`
/// binds `name` to — None = no bind (undefined name, defined
/// locally, ambiguous, or no parsable surface).
pub(super) fn binds_to(scope: &Scope, f: &str, name: &str) -> Option<Vec<String>> {
    let parsed = cached_tree(scope, f);
    let (text, tree) = parsed.as_ref().as_ref()?;
    if defines_toplevel(tree, text, name) {
        return None;
    }
    let entries = surface(scope, f, text, tree);
    let mut hits = entries.iter().filter(|(n, _)| n == name);
    let hit = hits.next()?;
    if hits.next().is_some() {
        return None; // ambiguous re-export: keep the file-level edge
    }
    Some(hit.1.clone())
}

/// The flattened pub-use surface, memoized per sweep (the md_slugs
/// shape): (bound name, full path segments) per leaf entry.
fn surface(scope: &Scope, f: &str, text: &str, tree: &tree_sitter::Tree) -> Rc<Vec<Entry>> {
    scope
        .memo
        .cached("rs_pubuse", f, || pub_entries(tree, text))
}

type Entry = (String, Vec<String>);

/// ONE iteration over the top-level pub use declarations — shared by
/// the surface and its hash so the two projections cannot drift (a
/// second inline copy of this loop was the census's catch).
fn pub_entries(tree: &tree_sitter::Tree, text: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    for item in crate::scan::ast::children(tree.root_node()) {
        if item.kind() != "use_declaration" || !is_pub(item, text) {
            continue;
        }
        if let Some(arg) = item.child_by_field_name("argument") {
            flatten(arg, text, "", &mut out);
        }
    }
    out
}

/// Recursive flatten of one use tree: nested brace groups expand,
/// `as` binds the ALIAS, `self` in a list binds the module name,
/// globs are skipped whole (an unknown name set is never followed).
fn flatten(node: tree_sitter::Node, src: &str, prefix: &str, out: &mut Vec<Entry>) {
    let text = |n: tree_sitter::Node| n.utf8_text(src.as_bytes()).unwrap_or("").to_string();
    match node.kind() {
        "use_list" => {
            for c in crate::scan::ast::children(node) {
                flatten(c, src, prefix, out);
            }
        }
        "scoped_use_list" => {
            let p = node
                .child_by_field_name("path")
                .map(text)
                .unwrap_or_default();
            let joined = join(prefix, &p);
            if let Some(list) = node.child_by_field_name("list") {
                flatten(list, src, &joined, out);
            }
        }
        "use_as_clause" => {
            let p = node
                .child_by_field_name("path")
                .map(text)
                .unwrap_or_default();
            let alias = node
                .child_by_field_name("alias")
                .map(text)
                .unwrap_or_default();
            if !alias.is_empty() {
                out.push((alias, split(&join(prefix, &p))));
            }
        }
        "use_wildcard" => {}
        "identifier" | "scoped_identifier" | "crate" | "super" | "self" => {
            let mut segs = split(&join(prefix, &text(node)));
            if segs.last().is_some_and(|s| s == "self") {
                segs.pop(); // `{self, …}` binds the module itself
            }
            if let Some(last) = segs.last() {
                out.push((last.clone(), segs.clone()));
            }
        }
        _ => {}
    }
}

/// pub / pub(crate) visibility only — a private use re-exports
/// nothing, and pub(self)/pub(in …) are not a consumer surface.
fn is_pub(item: tree_sitter::Node, src: &str) -> bool {
    crate::scan::ast::children(item)
        .into_iter()
        .find(|c| c.kind() == "visibility_modifier")
        .and_then(|v| v.utf8_text(src.as_bytes()).ok())
        .is_some_and(|t| t == "pub" || t == "pub(crate)")
}

/// F defining `name` at top level wins over any re-export of the
/// same name — the walk's own "items INSIDE that file" invariant.
fn defines_toplevel(tree: &tree_sitter::Tree, src: &str, name: &str) -> bool {
    const DEFS: [&str; 10] = [
        "struct_item",
        "enum_item",
        "function_item",
        "trait_item",
        "type_item",
        "const_item",
        "static_item",
        "mod_item",
        "union_item",
        "macro_definition",
    ];
    crate::scan::ast::children(tree.root_node())
        .into_iter()
        .any(|c| {
            DEFS.contains(&c.kind())
                && c.child_by_field_name("name")
                    .and_then(|n| n.utf8_text(src.as_bytes()).ok())
                    == Some(name)
        })
}

fn join(prefix: &str, tail: &str) -> String {
    if prefix.is_empty() {
        tail.to_string()
    } else {
        format!("{prefix}::{tail}")
    }
}

fn split(path: &str) -> Vec<String> {
    path.split("::")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// The surface folded to one resolve_key input (the md slug_hash
/// sibling): the hashed projection IS the consulted projection —
/// bound names and full paths, document order.
pub fn pubuse_hash(text: &str) -> u64 {
    let Some(grammar) = crate::scan::lang::Lang::Rust.grammar() else {
        return 0;
    };
    let Some(tree) = crate::scan::ast::parse(text, &grammar) else {
        return 0;
    };
    let mut buf = Vec::new();
    for (n, p) in pub_entries(&tree, text) {
        buf.extend_from_slice(n.as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(p.join("::").as_bytes());
        buf.push(b'\n');
    }
    crate::dedup::tokens::fnv1a(&buf)
}

#[cfg(test)]
mod tests {
    use super::pubuse_hash;

    /// The hashed projection and the consulted projection MOVE
    /// TOGETHER (the md_tests coupling stance): a surface edit shifts
    /// the key and re-fires the sweep; a body edit never does — the
    /// only road the cross-file staleness class returns for the
    /// binder is these two drifting apart.
    #[test]
    fn hash_moves_exactly_when_the_bound_surface_moves() {
        let pairs: &[(&str, bool, &str, &str)] = &[
            (
                "body-only edit holds",
                true,
                "pub use crate::a::X;\nfn f() {}\n",
                "pub use crate::a::X;\nfn g() {}\n",
            ),
            (
                "surface edit moves",
                false,
                "pub use crate::a::X;\n",
                "pub use crate::b::X;\n",
            ),
            (
                "alias change moves",
                false,
                "pub use crate::a::X as Y;\n",
                "pub use crate::a::X as Z;\n",
            ),
            (
                "a private use is no surface",
                true,
                "use crate::a::X;\n",
                "use crate::b::X;\n",
            ),
        ];
        for (why, same, a, b) in pairs {
            assert_eq!(pubuse_hash(a) == pubuse_hash(b), *same, "{why}");
        }
    }
}
