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
//! file-level edge (picking would invent a path). Since step 8 the
//! same file also answers the crate rung's tie-break (`owns`), so
//! the hashed projection covers every top-level fact a walk
//! consults: the pub-use surface, the private use bindings and the
//! item names (the step-8 review: the hash had stopped at the surface
//! while `owns` read the other two).

use super::Scope;
use super::rs_tree::cached_tree;
use std::rc::Rc;

/// The full path a single unambiguous top-level `pub use` of `f`
/// binds `name` to, with the declaration's row (the hop site's own
/// line, so its namespace is read at the file's top level) — None =
/// no bind (undefined name, defined locally, ambiguous, or no
/// parsable surface).
pub(super) fn binds_to(scope: &Scope, f: &str, name: &str) -> Option<(Vec<String>, usize)> {
    let parsed = cached_tree(scope, f);
    let (text, tree) = parsed.as_ref().as_ref()?;
    if defines_toplevel(tree, text, name) {
        return None;
    }
    let entries = surface(scope, f, text, tree);
    let mut hits = entries.iter().filter(|(n, _, _)| n == name);
    let hit = hits.next()?;
    if hits.next().is_some() {
        return None; // ambiguous re-export: keep the file-level edge
    }
    Some((hit.1.clone(), hit.2))
}

/// Whether `f` holds `name` in its top-level namespace — defined
/// there, or imported by any top-level `use` (private ones too: a
/// `crate::Thing` from a module resolves through a root's private
/// import just as well). The crate rung's tie-break between two roots
/// of one package (rs_use.rs, step 8).
pub(super) fn owns(scope: &Scope, f: &str, name: &str) -> bool {
    let parsed = cached_tree(scope, f);
    let Some((text, tree)) = parsed.as_ref().as_ref() else {
        return false;
    };
    defines_toplevel(tree, text, name)
        || use_entries(tree, text, false)
            .iter()
            .any(|(n, _, _)| n == name)
}

/// The flattened pub-use surface, memoized per sweep (the md_slugs
/// shape): (bound name, full path segments, row) per leaf entry.
fn surface(scope: &Scope, f: &str, text: &str, tree: &tree_sitter::Tree) -> Rc<Vec<Entry>> {
    scope
        .memo
        .cached("rs_pubuse", f, || pub_entries(tree, text))
}

/// (bound name, full path segments, the declaration's row).
type Entry = (String, Vec<String>, usize);

/// ONE iteration over the top-level pub use declarations — shared by
/// the surface and its hash so the two projections cannot drift (a
/// second inline copy of this loop was the census's catch).
fn pub_entries(tree: &tree_sitter::Tree, text: &str) -> Vec<Entry> {
    use_entries(tree, text, true)
}

/// The top-level use declarations flattened — every one, or the pub
/// ones alone (the re-export surface).
fn use_entries(tree: &tree_sitter::Tree, text: &str, pub_only: bool) -> Vec<Entry> {
    let mut out = Vec::new();
    for item in crate::scan::ast::children(tree.root_node()) {
        if item.kind() != "use_declaration" || (pub_only && !is_pub(item, text)) {
            continue;
        }
        if let Some(arg) = item.child_by_field_name("argument") {
            flatten(arg, text, "", item.start_position().row, &mut out);
        }
    }
    out
}

/// Recursive flatten of one use tree: nested brace groups expand,
/// `as` binds the ALIAS, `self` in a list binds the module name,
/// globs are skipped whole (an unknown name set is never followed).
fn flatten(node: tree_sitter::Node, src: &str, prefix: &str, row: usize, out: &mut Vec<Entry>) {
    let text = |n: tree_sitter::Node| n.utf8_text(src.as_bytes()).unwrap_or("").to_string();
    match node.kind() {
        "use_list" => {
            for c in crate::scan::ast::children(node) {
                flatten(c, src, prefix, row, out);
            }
        }
        "scoped_use_list" => {
            let p = node
                .child_by_field_name("path")
                .map(text)
                .unwrap_or_default();
            let joined = join(prefix, &p);
            if let Some(list) = node.child_by_field_name("list") {
                flatten(list, src, &joined, row, out);
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
                out.push((alias, split(&join(prefix, &p)), row));
            }
        }
        "use_wildcard" => {}
        "identifier" | "scoped_identifier" | "crate" | "super" | "self" => {
            let mut segs = split(&join(prefix, &text(node)));
            if segs.last().is_some_and(|s| s == "self") {
                segs.pop(); // `{self, …}` binds the module itself
            }
            if let Some(last) = segs.last() {
                out.push((last.clone(), segs.clone(), row));
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

/// The names F defines at top level, ONE iteration (the module's own
/// discipline): the definition-wins refusal and the crate rung's
/// tie-break both read it, and the hash folds it.
fn toplevel_defs(tree: &tree_sitter::Tree, src: &str) -> Vec<String> {
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
        .filter(|c| DEFS.contains(&c.kind()))
        .filter_map(|c| c.child_by_field_name("name"))
        .filter_map(|n| n.utf8_text(src.as_bytes()).ok())
        .map(str::to_string)
        .collect()
}

/// F defining `name` at top level wins over any re-export of the
/// same name — the walk's own "items INSIDE that file" invariant.
fn defines_toplevel(tree: &tree_sitter::Tree, src: &str, name: &str) -> bool {
    toplevel_defs(tree, src).iter().any(|n| n == name)
}

fn join(prefix: &str, tail: &str) -> String {
    if prefix.is_empty() {
        tail.to_string()
    } else {
        format!("{prefix}::{tail}")
    }
}

/// Path segments; the global form (`::foo::Bar`) keeps an EMPTY first
/// segment — the marker the hop reads to walk the path as a crate
/// name and never as a local module (rs_use.rs, the step-8 review).
fn split(path: &str) -> Vec<String> {
    let mut segs: Vec<String> = path
        .split("::")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if path.trim_start().starts_with("::") {
        segs.insert(0, String::new());
    }
    segs
}

/// The surface folded to one resolve_key input (the md slug_hash
/// sibling): the hashed projection IS the consulted projection —
/// the pub-use bindings (names and full paths, document order), then
/// every top-level use binding's name and every top-level item name,
/// the two facts `owns` and `binds_to` also read. A private-use or
/// item rename costs one spurious sweep; a missing fact would cost a
/// permanently wrong edge — keys.rs's own trade for the TS facts.
pub fn pubuse_hash(text: &str) -> u64 {
    let Some(grammar) = crate::scan::lang::Lang::Rust.grammar() else {
        return 0;
    };
    let Some(tree) = crate::scan::ast::parse(text, &grammar) else {
        return 0;
    };
    let mut buf = Vec::new();
    for (n, p, _) in pub_entries(&tree, text) {
        buf.extend_from_slice(n.as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(p.join("::").as_bytes());
        buf.push(b'\n');
    }
    let bindings = use_entries(&tree, text, false)
        .into_iter()
        .map(|(n, _, _)| n);
    for name in std::iter::once(String::new()).chain(bindings) {
        buf.extend_from_slice(name.as_bytes());
        buf.push(b'\n');
    }
    for name in std::iter::once(String::new()).chain(toplevel_defs(&tree, text)) {
        buf.extend_from_slice(name.as_bytes());
        buf.push(b'\n');
    }
    crate::dedup::tokens::fnv1a(&buf)
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/rs_reexport.rs"]
mod tests;
