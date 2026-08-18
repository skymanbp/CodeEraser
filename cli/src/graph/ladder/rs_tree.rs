//! The Rust module-tree machinery (split from rs.rs at the 300-line
//! dogfood gate): one child-lookup throat, the descent that stops at
//! the deepest in-scope module, the multi-anchor walk, and the
//! super-climb over file-level module owners. rs.rs owns the rung
//! POLICY; this file owns the tree MECHANICS.

use super::{Reason, Scope};
use crate::graph::roots;
use std::collections::BTreeSet;

/// Per-sweep parse cache (review MED): every self/super/#[path] site
/// in one file shares one read + parse; the text rides along so the
/// tree's provenance stays in one place.
pub(crate) fn cached_tree(
    scope: &Scope,
    from: &str,
) -> std::rc::Rc<Option<(String, tree_sitter::Tree)>> {
    scope.memo.cached("rs_tree", from, || {
        let text = std::fs::read_to_string(scope.root.join(from)).ok()?;
        let grammar = crate::scan::lang::Lang::Rust.grammar()?;
        let tree = crate::scan::ast::parse(&text, &grammar)?;
        Some((text, tree))
    })
}

/// The root→leaf chain of nodes covering `row` — ONE descent walker
/// shared by the inline-depth count and the #[path] probe (a second
/// copy of this loop was the ratchet's eighteenth catch).
pub(crate) fn covering_chain(tree: &tree_sitter::Tree, row: usize) -> Vec<tree_sitter::Node<'_>> {
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

/// Names of the BODIED mod_items enclosing `row`, outermost first —
/// the inline-module components of a #[path] base (the same chain
/// inline_depth counts; here the names travel).
pub(crate) fn inline_mods(tree: &tree_sitter::Tree, src: &str, row: usize) -> Vec<String> {
    covering_chain(tree, row)
        .iter()
        .filter(|c| c.kind() == "mod_item" && c.child_by_field_name("body").is_some())
        .filter_map(|c| c.child_by_field_name("name"))
        .filter_map(|n| n.utf8_text(src.as_bytes()).ok())
        .map(str::to_string)
        .collect()
}

/// The mod_item starting exactly at `row` whose name field is
/// `name` — a projection of the shared covering chain.
pub(crate) fn mod_item_at<'t>(
    tree: &'t tree_sitter::Tree,
    src: &str,
    row: usize,
    name: &str,
) -> Option<tree_sitter::Node<'t>> {
    covering_chain(tree, row).into_iter().find(|c| {
        c.kind() == "mod_item"
            && c.start_position().row == row
            && c.child_by_field_name("name")
                .and_then(|n| n.utf8_text(src.as_bytes()).ok())
                == Some(name)
    })
}

/// `Some(target)` when `item` is `#[path = "target"]` — the value
/// field's string_content, so the quotes never travel.
pub(crate) fn attr_path_value(item: tree_sitter::Node, src: &str) -> Option<String> {
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

/// Walk one segment list from every anchor; distinct terminals from
/// different anchors are ambiguous_root (the Python cross-root
/// stance), a double hit at one step is ambiguous_paths, an empty
/// anchor set (climbed above the crate root) is out of scope.
pub(crate) fn walk_all(
    anchors: Vec<String>,
    segs: &[&str],
    rung: u8,
    roots_set: &BTreeSet<String>,
    files: &BTreeSet<String>,
) -> super::Outcome {
    let mut hits = BTreeSet::new();
    for anchor in anchors {
        match descend(&anchor, segs, roots_set, files) {
            Ok(target) => {
                hits.insert(target);
            }
            Err(reason) => return super::Outcome::Unresolved(reason),
        }
    }
    match hits.len() {
        0 => super::Outcome::Unresolved(Reason::OutOfScope),
        1 => super::Outcome::Resolved {
            path: hits.pop_first().expect("len checked"),
            rung,
        },
        _ => super::Outcome::Unresolved(Reason::AmbiguousRoot),
    }
}

/// Descend the convention tree; stopping early is not failure — the
/// remaining segments live inside the deepest matched file.
pub(crate) fn descend(
    anchor: &str,
    segs: &[&str],
    roots_set: &BTreeSet<String>,
    files: &BTreeSet<String>,
) -> Result<String, Reason> {
    let mut cur = anchor.to_string();
    for seg in segs {
        match child(&cur, seg, roots_set, files) {
            Child::One(next) => cur = next,
            Child::Both => return Err(Reason::AmbiguousPaths),
            Child::None => break,
        }
    }
    Ok(cur)
}

pub(crate) enum Child {
    One(String),
    Both,
    None,
}

/// The child-module lookup throat: dir/name.rs | dir/name/mod.rs,
/// both present is rustc's own E0761 ambiguity.
pub(crate) fn child(
    file: &str,
    name: &str,
    roots_set: &BTreeSet<String>,
    files: &BTreeSet<String>,
) -> Child {
    let dir = child_dir(file, roots_set);
    let plain = roots::join_dir(&dir, &format!("{name}.rs"));
    let modrs = roots::join_dir(&dir, &format!("{name}/mod.rs"));
    match (files.contains(&plain), files.contains(&modrs)) {
        (true, true) => Child::Both,
        (true, false) => Child::One(plain),
        (false, true) => Child::One(modrs),
        (false, false) => Child::None,
    }
}

/// A crate root or a mod.rs parents children in its OWN directory;
/// any other module file parents them under dir/<stem>/ (2018 style).
/// pub(crate): the #[path] rung's INLINE-context base is exactly
/// this rule plus the enclosing inline mod names (rustc reference).
pub(crate) fn child_dir(file: &str, roots_set: &BTreeSet<String>) -> String {
    let dir = roots::parent_dir(file);
    if roots_set.contains(file) || is_mod_rs(file) {
        return dir;
    }
    let stem = file
        .rsplit('/')
        .next()
        .unwrap_or(file)
        .trim_end_matches(".rs");
    roots::join_dir(&dir, stem)
}

fn is_mod_rs(file: &str) -> bool {
    file == "mod.rs" || file.ends_with("/mod.rs")
}

/// The crate roots whose module tree can contain `from`: itself when
/// it IS a root, else the roots whose directory is the deepest
/// prefix (src/bin/x/helper.rs belongs to bin x, not to the lib that
/// also covers src/ — deepest-wins is Cargo semantics, not a pick).
pub(crate) fn covering_roots(from: &str, roots_set: &BTreeSet<String>) -> Vec<String> {
    if roots_set.contains(from) {
        return vec![from.to_string()];
    }
    let mut best: Vec<String> = Vec::new();
    let mut best_len = 0usize;
    for root in roots_set {
        let dir = roots::parent_dir(root);
        if !(dir.is_empty() || from.starts_with(&format!("{dir}/"))) {
            continue;
        }
        if best.is_empty() || dir.len() > best_len {
            best = vec![root.clone()];
            best_len = dir.len();
        } else if dir.len() == best_len {
            best.push(root.clone());
        }
    }
    best
}

/// k×super: each step maps every anchor to the file(s) owning its
/// parent directory; a crate root has no parent and drops out. All
/// owners of one directory share one child directory, so divergent
/// climbs can only differ at the terminal — walk_all's check.
pub(crate) fn climb(
    from: &str,
    ups: usize,
    roots_set: &BTreeSet<String>,
    files: &BTreeSet<String>,
) -> Vec<String> {
    let mut cur = BTreeSet::from([from.to_string()]);
    for _ in 0..ups {
        let mut next = BTreeSet::new();
        for f in cur.iter().filter(|f| !roots_set.contains(*f)) {
            let dir = if is_mod_rs(f) {
                roots::parent_dir(&roots::parent_dir(f))
            } else {
                roots::parent_dir(f)
            };
            next.extend(owners(&dir, roots_set, files));
        }
        cur = next;
        if cur.is_empty() {
            break;
        }
    }
    cur.into_iter().collect()
}

/// Files whose child directory is `dir`: dir/mod.rs, the sibling
/// <dir>.rs, and any crate root sitting directly in dir.
fn owners(dir: &str, roots_set: &BTreeSet<String>, files: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let modrs = roots::join_dir(dir, "mod.rs");
    if files.contains(&modrs) {
        out.insert(modrs);
    }
    let sibling = format!("{dir}.rs");
    if !dir.is_empty() && files.contains(&sibling) {
        out.insert(sibling);
    }
    for root in roots_set {
        if roots::parent_dir(root) == dir {
            out.insert(root.clone());
        }
    }
    out
}
