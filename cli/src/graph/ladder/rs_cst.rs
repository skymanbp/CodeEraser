//! What the Rust rungs read of a file's syntax tree (plan v2.33 W2-text
//! stage F): the rungs, the Cargo.toml reader and the module-tree walk
//! moved into the core (CE.Resolve.Rs, CE.Resolve.RsTree,
//! CE.Resolve.Cargo); tree-sitter stays on this side, so a file's
//! concrete syntax travels as the two facts the core asks for
//! (resolve/1 fact ops 4 and 5, graph/resolve/request.rs), each computed
//! by the CST walks that stood in rs_tree.rs, rs_use.rs and rs_bind.rs:
//!   `at` — at one row: how many bodied `mod` items enclose it and their
//!   names, outermost first; each `mod` item starting there (on the
//!   covering chain) with its `#[path]` value; the `mod` items of the
//!   namespace enclosing it — the innermost bodied mod's, else the
//!   file's — each bodied or not with its row; each `use` item starting
//!   there with its argument's first line, trimmed, and the whole
//!   argument with its whitespace folded;
//!   `surface` — the top-level item names with whether each is pub, and
//!   every top-level use and extern-crate binding flattened
//!   (rs_surface.rs) with whether its item is pub.
//! What a site's path means, which row is asked and how the answers are
//! read are the core's. `pubuse_hash` folds the surface into the
//! resolve key (dedup/walkidx.rs).

use serde_json::{Value, json};

#[path = "rs_surface.rs"]
mod surface;
use surface::{toplevel_defs, use_entries};

/// A Rust file's text and tree; none when it cannot be read or parsed.
pub(crate) fn parsed(root: &std::path::Path, rel: &str) -> Option<(String, tree_sitter::Tree)> {
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    let grammar = crate::scan::lang::Lang::Rust.grammar()?;
    let tree = crate::scan::ast::parse(&text, &grammar)?;
    Some((text, tree))
}

/// The root→leaf chain of nodes covering `row`.
fn covering_chain(tree: &tree_sitter::Tree, row: usize) -> Vec<tree_sitter::Node<'_>> {
    let (mut node, mut out) = (tree.root_node(), Vec::new());
    'down: loop {
        for c in crate::scan::ast::children(node) {
            if c.start_position().row <= row && row <= c.end_position().row {
                out.push(c);
                node = c;
                continue 'down;
            }
        }
        return out;
    }
}

fn bodied(node: tree_sitter::Node) -> bool {
    node.kind() == "mod_item" && node.child_by_field_name("body").is_some()
}

/// A `mod_item`'s name, none for another node or a nameless one.
fn mod_name(node: tree_sitter::Node, src: &str) -> Option<String> {
    if node.kind() != "mod_item" {
        return None;
    }
    let name = node.child_by_field_name("name")?;
    name.utf8_text(src.as_bytes()).ok().map(str::to_string)
}

/// `Some(target)` when `item` is `#[path = "target"]` — the value
/// field's string_content, so the quotes never travel.
fn attr_path_value(item: tree_sitter::Node, src: &str) -> Option<String> {
    let attr = crate::scan::ast::children(item)
        .into_iter()
        .find(|c| c.kind() == "attribute")?;
    let key = attr.named_child(0)?;
    if key.utf8_text(src.as_bytes()).ok()? != "path" {
        return None;
    }
    let val = attr.child_by_field_name("value")?;
    let content = crate::scan::ast::children(val)
        .into_iter()
        .find(|c| c.kind() == "string_content")?;
    content.utf8_text(src.as_bytes()).ok().map(str::to_string)
}

/// The `#[path]` value among the attribute items stacked before `node`
/// (attributes are preceding siblings of a mod_item), the nearest first.
fn path_attr(node: tree_sitter::Node, src: &str) -> Option<String> {
    let mut prev = node.prev_sibling();
    while let Some(p) = prev {
        if p.kind() != "attribute_item" {
            break;
        }
        if let Some(v) = attr_path_value(p, src) {
            return Some(v);
        }
        prev = p.prev_sibling();
    }
    None
}

/// The `mod` items of the namespace enclosing `row`: the innermost
/// bodied mod's declaration list on the covering chain, else the file's
/// root — `[name, bodied, row]` each, in order.
fn namespace(tree: &tree_sitter::Tree, src: &str, chain: &[tree_sitter::Node]) -> Vec<Value> {
    let ns = chain
        .iter()
        .rev()
        .find(|c| bodied(**c))
        .and_then(|m| m.child_by_field_name("body"))
        .unwrap_or_else(|| tree.root_node());
    crate::scan::ast::children(ns)
        .into_iter()
        .filter_map(|c| {
            Some(json!([
                mod_name(c, src)?,
                bodied(c),
                c.start_position().row
            ]))
        })
        .collect()
}

/// Each `use` item starting on `row`, in the order the tree is searched
/// (a stack, the root first): its argument's first line, trimmed, and
/// the whole argument with its whitespace folded to single spaces.
fn uses(tree: &tree_sitter::Tree, src: &str, row: usize) -> Vec<Value> {
    let (mut out, mut pending) = (Vec::new(), vec![tree.root_node()]);
    while let Some(node) = pending.pop() {
        if node.start_position().row > row || node.end_position().row < row {
            continue;
        }
        if node.kind() == "use_declaration"
            && node.start_position().row == row
            && let Some(arg) = node.child_by_field_name("argument")
            && let Ok(text) = arg.utf8_text(src.as_bytes())
            && let Some(first) = text.lines().next()
        {
            let folded = text.split_whitespace().collect::<Vec<_>>().join(" ");
            out.push(json!([first.trim(), folded]));
        }
        pending.extend(crate::scan::ast::children(node));
    }
    out
}

/// Fact 4: a file's answers at one 0-based row.
pub(crate) fn at(tree: &tree_sitter::Tree, src: &str, row: usize) -> Value {
    let chain = covering_chain(tree, row);
    let mods: Vec<String> = chain
        .iter()
        .filter(|c| bodied(**c))
        .filter_map(|c| c.child_by_field_name("name"))
        .filter_map(|n| n.utf8_text(src.as_bytes()).ok())
        .map(str::to_string)
        .collect();
    let items: Vec<Value> = chain
        .iter()
        .filter(|c| c.start_position().row == row)
        .filter_map(|c| Some(json!([mod_name(*c, src)?, path_attr(*c, src)])))
        .collect();
    json!({
        "depth": chain.iter().filter(|c| bodied(**c)).count(),
        "mods": mods,
        "items": items,
        "ns": namespace(tree, src, &chain),
        "uses": uses(tree, src, row),
    })
}

/// Fact 5: a file's top-level surface.
pub(crate) fn surface(tree: &tree_sitter::Tree, src: &str) -> Value {
    let defs: Vec<Value> = toplevel_defs(tree, src)
        .into_iter()
        .map(|(name, is_pub)| json!([name, is_pub]))
        .collect();
    let uses: Vec<Value> = use_entries(tree, src)
        .into_iter()
        .map(|((name, path, row), is_pub)| json!([name, path, row, is_pub]))
        .collect();
    json!({ "defs": defs, "uses": uses })
}

/// The surface folded to one resolve_key input (the md slug_hash
/// sibling): the hashed projection IS the consulted projection — the
/// pub-use bindings (names and full paths, document order; a glob under
/// its `*` slot, an extern crate under its bound name), then every
/// top-level use binding's name and every top-level item name with its
/// visibility mark. A private-use or item rename costs one spurious
/// sweep; a missing fact would cost a permanently wrong edge — keys.rs's
/// own trade for the TS facts.
pub fn pubuse_hash(text: &str) -> u64 {
    let Some(grammar) = crate::scan::lang::Lang::Rust.grammar() else {
        return 0;
    };
    let Some(tree) = crate::scan::ast::parse(text, &grammar) else {
        return 0;
    };
    let entries = use_entries(&tree, text);
    let mut buf = Vec::new();
    for ((n, p, _), _) in entries.iter().filter(|(_, is_pub)| *is_pub) {
        buf.extend_from_slice(n.as_bytes());
        buf.push(b'=');
        buf.extend_from_slice(p.join("::").as_bytes());
        buf.push(b'\n');
    }
    let bindings = entries.into_iter().map(|((n, _, _), _)| n);
    for name in std::iter::once(String::new()).chain(bindings) {
        buf.extend_from_slice(name.as_bytes());
        buf.push(b'\n');
    }
    for (name, is_pub) in std::iter::once((String::new(), false)).chain(toplevel_defs(&tree, text))
    {
        buf.extend_from_slice(name.as_bytes());
        buf.push(if is_pub { b'+' } else { b'-' });
        buf.push(b'\n');
    }
    crate::dedup::tokens::fnv1a(&buf)
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/rs_cst.rs"]
mod tests;
