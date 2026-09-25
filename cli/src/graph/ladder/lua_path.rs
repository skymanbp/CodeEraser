//! The templates a Lua project writes into its own `package.path` (plan
//! v2.30 step 4; design booklet §8 row Lua). `require "a.b"` asks each
//! template of the path in turn, `?` standing for the module name with
//! its dots turned into slashes, and loads the first file that exists
//! (Lua 5.4 manual §6.3, `package.searchpath`). The path is a run-time
//! value, but a project that assigns it literal text —
//! `package.path = "frontend/?.lua;" .. package.path` — has written its
//! search directories down. The walk reads every Lua file this way on
//! every run (dedup/walkidx.rs) and hands the union to the Lua ladder
//! (Scope::lua), which reads no file itself.
//!
//! Only literal text counts: each string operand of the assigned
//! value's `..` chain, parentheses looked through. A call
//! (`string.format("%s/?.lua", dir)`) or a variable states no template
//! in the text. A template is kept when it is relative — to the working
//! directory, which the ladder takes to be the tree root, the
//! convention its load rung shares — names one `?` and ends `.lua`; an
//! absolute one (`/usr/share/lua/5.1/?.lua`) names no file of the tree.

use crate::scan::ast;
use crate::scan::lang::Lang;
use tree_sitter::Node;

/// One template as written: the directory before its `?` (`""` = the
/// working directory, the trailing `/` dropped) and what follows the
/// `?` (`.lua`, `/init.lua`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Template {
    pub dir: String,
    pub suffix: String,
}

/// Every literal template one file assigns to `package.path`, in
/// document order. A file that never spells `package.path` is not
/// parsed.
pub fn read(text: &str) -> Vec<Template> {
    if !text.contains("package.path") {
        return Vec::new();
    }
    ast::with_tree(text, Lang::Lua, |tree| {
        let src = text.as_bytes();
        let mut out = Vec::new();
        for node in ast::preorder(tree.root_node(), |_| true, ast::named_children) {
            if node.kind() == "assignment_statement" {
                assigned(node, src, &mut out);
            }
        }
        out
    })
}

/// The templates of one assignment: each target spelled `package.path`
/// — the prefilter's own spelling, so no file reads differently for
/// what else it holds — takes the value in its own position (Lua
/// assigns the i-th value to the i-th variable).
fn assigned(node: Node<'_>, src: &[u8], out: &mut Vec<Template>) {
    let list = |kind: &str| {
        ast::named_children(node)
            .into_iter()
            .find(|c| c.kind() == kind)
            .map_or_else(Vec::new, ast::entries)
    };
    let values = list("expression_list");
    for (at, var) in list("variable_list").iter().enumerate() {
        if var.utf8_text(src) != Ok("package.path") {
            continue;
        }
        for text in values.get(at).map_or_else(Vec::new, |v| literals(*v, src)) {
            out.extend(text.split(';').filter_map(template));
        }
    }
}

/// The literal text a value states: a string's content, or each string
/// operand of a `..` chain, parentheses looked through; anything else
/// states none.
fn literals<'s>(node: Node<'_>, src: &'s [u8]) -> Vec<&'s str> {
    match node.kind() {
        "string" => node
            .child_by_field_name("content")
            .and_then(|c| c.utf8_text(src).ok())
            .into_iter()
            .collect(),
        "parenthesized_expression" => ast::named_children(node)
            .into_iter()
            .flat_map(|n| literals(n, src))
            .collect(),
        "binary_expression" if ast::operator_text(node, src) == Some("..") => ["left", "right"]
            .iter()
            .filter_map(|f| node.child_by_field_name(f))
            .flat_map(|n| literals(n, src))
            .collect(),
        _ => Vec::new(),
    }
}

/// One `;`-separated piece as a template: relative (no leading `/` or
/// `~`, no drive or variable), its `./` prefixes dropped, a directory
/// that is empty or ends `/`, exactly one `?`, and a `.lua` ending. A
/// backslash is a Windows separator — an escaped one (`.\\?.lua` in
/// the source, the string's text being read undecoded) first becomes
/// the one backslash the string holds.
fn template(piece: &str) -> Option<Template> {
    let piece = piece.replace("\\\\", "\\").replace('\\', "/");
    let mut rest = piece.as_str();
    while let Some(tail) = rest.strip_prefix("./") {
        rest = tail;
    }
    let (dir, suffix) = rest.split_once('?')?;
    let relative = !(dir.starts_with(['/', '~']) || dir.contains([':', '$']));
    let shaped = (dir.is_empty() || dir.ends_with('/')) && !suffix.contains('?');
    (relative && shaped && suffix.ends_with(".lua")).then(|| Template {
        dir: dir.trim_end_matches('/').to_string(),
        suffix: suffix.to_string(),
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/lua_path.rs"]
mod tests;
