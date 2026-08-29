//! Unit subtrees in the clone/1 wire form (design vol.2 §2.2):
//! postorder kind labels + leftmost-leaf-descendant indices, selected
//! by the SAME predicate as `struct_fp::unit_seq` (named nodes whose
//! 1-based line range sits inside the unit span) — so a built tree's
//! node count EQUALS the cached unitsig.nodes, and the driver asserts
//! that per unit instead of assuming two walks agree. A span whose
//! maximal selected nodes are not exactly one is a forest — a
//! ledgered outcome, never a guessed root.

use crate::dedup::struct_fp;
use crate::scan::ast;
use crate::scan::lang::Lang;
use tree_sitter::Node;

/// One wire-ready tree. `lab` holds raw fnv1a kind codes — the
/// request-local DENSE mapping happens at wire time, per request.
pub struct UnitTree {
    pub lab: Vec<u64>,
    pub lld: Vec<i64>,
}

/// One span's outcome.
pub enum Built {
    Tree(UnitTree),
    /// The span's maximal selected nodes number != 1 (their count).
    Forest(usize),
}

/// Build every span's tree from ONE parse of the file. Empty result =
/// no grammar or parse failure (the `with_tree` contract); the caller
/// treats a short result as drift, since cached unitsig rows exist
/// only for texts that parsed.
pub fn file_trees(text: &str, lang: Lang, spans: &[(usize, usize)]) -> Vec<Built> {
    ast::with_tree(text, lang, |tree| {
        spans
            .iter()
            .map(|&(s, e)| build(tree.root_node(), s, e))
            .collect()
    })
}

fn build(root: Node, start: usize, end: usize) -> Built {
    let mut tops = Vec::new();
    maximal(root, start, end, &mut tops);
    match tops[..] {
        [one] => {
            let mut t = UnitTree {
                lab: Vec::new(),
                lld: Vec::new(),
            };
            emit(one, &mut t);
            Built::Tree(t)
        }
        _ => Built::Forest(tops.len()),
    }
}

/// The maximal selected nodes: named nodes inside [start, end] whose
/// nearest named ancestor is not. Containment is the unit_seq
/// predicate verbatim; descent stops at a hit (position nesting makes
/// the whole subtree selected) and at ranges that cannot intersect.
/// Explicit stack like every other walker in this crate (M5-close
/// review LOW: call-stack recursion made a pathologically deep AST a
/// process abort instead of a judgment).
fn maximal<'t>(node: Node<'t>, start: usize, end: usize, out: &mut Vec<Node<'t>>) {
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        let (s, e) = lines(n);
        if e < start || s > end {
            continue;
        }
        if n.is_named() && start <= s && e <= end {
            out.push(n);
            continue;
        }
        stack.extend(ast::children(n).into_iter().rev());
    }
}

/// 1-based inclusive line range — the `fourclass::units` convention
/// `struct_fp::spine` also encodes.
fn lines(node: Node) -> (usize, usize) {
    (node.start_position().row + 1, node.end_position().row + 1)
}

/// Postorder emission of `node`'s named subtree (children first, left
/// to right, nearest-named flattening across anonymous intermediates
/// — the spine's selection, with structure kept). Two-phase explicit
/// stack; a node's lld is the postorder index of its leftmost leaf,
/// which is exactly `lab.len()` at ENTER time whenever the subtree
/// emits anything before the node itself — the recursive
/// first-child-return threading, derived instead of threaded.
fn emit(node: Node, t: &mut UnitTree) {
    let mut stack = vec![(node, None)];
    while let Some((n, entered_at)) = stack.pop() {
        let Some(base) = entered_at else {
            stack.push((n, Some(t.lab.len() as i64)));
            let kids = named_kids(n);
            stack.extend(kids.into_iter().rev().map(|k| (k, None)));
            continue;
        };
        let idx = t.lab.len() as i64;
        t.lab.push(struct_fp::kind_code(n.kind()));
        t.lld.push(if base < idx { base } else { idx });
    }
}

/// A node's named children, looking through anonymous intermediates
/// (today's grammars make anonymous nodes terminals, so the descent
/// costs nothing — it keeps the selected SET identical to the spine's
/// all-children walk by construction, not by grammar accident).
fn named_kids(node: Node) -> Vec<Node> {
    let mut out = Vec::new();
    let mut stack: Vec<Node> = ast::children(node).into_iter().rev().collect();
    while let Some(c) = stack.pop() {
        if c.is_named() {
            out.push(c);
        } else {
            stack.extend(ast::children(c).into_iter().rev());
        }
    }
    out
}

#[cfg(test)]
#[path = "../../../tests/unit/dedup/t3/tree.rs"]
mod tests;
