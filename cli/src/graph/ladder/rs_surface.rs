//! The top-level surface of one Rust file, read off its tree — the
//! child of rs_reexport.rs, split at the 300 line when plan v2.30 step
//! 5b taught the surface globs and extern crates: every `use`
//! declaration flattened to (bound name, full path, row) entries — a
//! glob under the `*` slot, an `extern crate x [as y]` under its bound
//! name as the crate's global path — and the item names the file
//! defines with their visibility. One iteration each, shared by the
//! parent's questions (`binds_to`, `exports`, `owns`) and the hash that
//! keys the edge cache, so the consulted and the hashed projections
//! cannot drift.

/// The name slot of a glob entry: `*` is no identifier, so it
/// collides with no bound name.
pub(super) const GLOB: &str = "*";

/// (bound name, full path segments, the declaration's row).
pub(super) type Entry = (String, Vec<String>, usize);

/// The top-level use declarations flattened, and the extern crate
/// declarations, each binding its name or alias to the global path of
/// the crate — every one, or the pub ones alone (the re-export
/// surface).
pub(super) fn use_entries(tree: &tree_sitter::Tree, src: &str, pub_only: bool) -> Vec<Entry> {
    let mut out = Vec::new();
    for item in crate::scan::ast::children(tree.root_node()) {
        if pub_only && !is_pub(item, src) {
            continue;
        }
        match item.kind() {
            "use_declaration" => {
                if let Some(arg) = item.child_by_field_name("argument") {
                    flatten(arg, src, "", item.start_position().row, &mut out);
                }
            }
            "extern_crate_declaration" => extern_crate(item, src, &mut out),
            _ => {}
        }
    }
    out
}

/// `extern crate x;` binds `x`, `extern crate x as y;` binds `y`, to
/// the crate — the global form's path (`::x`, an empty first segment),
/// the marker the hop reads (plan v2.30 step 5b).
fn extern_crate(item: tree_sitter::Node, src: &str, out: &mut Vec<Entry>) {
    let Some(name) = item.child_by_field_name("name").map(|n| text(n, src)) else {
        return;
    };
    let bound = item
        .child_by_field_name("alias")
        .map(|n| text(n, src))
        .unwrap_or_else(|| name.clone());
    out.push((bound, vec![String::new(), name], item.start_position().row));
}

/// Recursive flatten of one use tree: nested brace groups expand,
/// `as` binds the ALIAS, `self` in a list binds the module name, a
/// glob is an entry under the `*` slot whose path is the module it
/// opens (followed only through `exports`, step 5b).
fn flatten(node: tree_sitter::Node, src: &str, prefix: &str, row: usize, out: &mut Vec<Entry>) {
    let field = |name: &str| {
        node.child_by_field_name(name)
            .map(|n| text(n, src))
            .unwrap_or_default()
    };
    match node.kind() {
        "use_list" => {
            for c in crate::scan::ast::children(node) {
                flatten(c, src, prefix, row, out);
            }
        }
        "scoped_use_list" => {
            let joined = join(prefix, &field("path"));
            if let Some(list) = node.child_by_field_name("list") {
                flatten(list, src, &joined, row, out);
            }
        }
        "use_as_clause" => {
            let alias = field("alias");
            if !alias.is_empty() {
                out.push((alias, split(&join(prefix, &field("path"))), row));
            }
        }
        "use_wildcard" => {
            let p = crate::scan::ast::named_children(node)
                .first()
                .map(|n| text(*n, src))
                .unwrap_or_default();
            let segs = split(&join(prefix, &p));
            if !segs.is_empty() {
                out.push((GLOB.to_string(), segs, row));
            }
        }
        "identifier" | "scoped_identifier" | "crate" | "super" | "self" => {
            let mut segs = split(&join(prefix, &text(node, src)));
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

fn text(n: tree_sitter::Node, src: &str) -> String {
    n.utf8_text(src.as_bytes()).unwrap_or("").to_string()
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

/// The names F defines at top level, each with whether it is pub, ONE
/// iteration (the module's own discipline): the definition-wins
/// refusal, the crate rung's tie-break and the glob export read it,
/// and the hash folds it.
pub(super) fn toplevel_defs(tree: &tree_sitter::Tree, src: &str) -> Vec<(String, bool)> {
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
        .filter_map(|c| {
            let name = c
                .child_by_field_name("name")?
                .utf8_text(src.as_bytes())
                .ok()?;
            Some((name.to_string(), is_pub(c, src)))
        })
        .collect()
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
/// name and never as a local module (rs_bind.rs, the step-8 review).
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
