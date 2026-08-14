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
fn maximal<'t>(node: Node<'t>, start: usize, end: usize, out: &mut Vec<Node<'t>>) {
    let (s, e) = lines(node);
    if e < start || s > end {
        return;
    }
    if node.is_named() && start <= s && e <= end {
        out.push(node);
        return;
    }
    for child in ast::children(node) {
        maximal(child, start, end, out);
    }
}

/// 1-based inclusive line range — the `fourclass::units` convention
/// `struct_fp::spine` also encodes.
fn lines(node: Node) -> (usize, usize) {
    (node.start_position().row + 1, node.end_position().row + 1)
}

/// Postorder emission of `node`'s named subtree (children first, left
/// to right, nearest-named flattening across anonymous intermediates
/// — the spine's selection, with structure kept). Returns the emitted
/// node's lld VALUE: its leftmost leaf's postorder index.
fn emit(node: Node, t: &mut UnitTree) -> i64 {
    let mut first = None;
    for kid in named_kids(node) {
        let l = emit(kid, t);
        first.get_or_insert(l);
    }
    let idx = t.lab.len() as i64;
    let l = first.unwrap_or(idx);
    t.lab.push(struct_fp::kind_code(node.kind()));
    t.lld.push(l);
    l
}

/// A node's named children, looking through anonymous intermediates
/// (today's grammars make anonymous nodes terminals, so the recursion
/// costs nothing — it keeps the selected SET identical to the spine's
/// all-children walk by construction, not by grammar accident).
fn named_kids(node: Node) -> Vec<Node> {
    let mut out = Vec::new();
    for child in ast::children(node) {
        if child.is_named() {
            out.push(child);
        } else {
            out.extend(named_kids(child));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dedup::unitcache;
    use crate::fourclass::units;

    /// The same-source counterfactual (3e): tree walk and unitsig
    /// fact walk select by one predicate, so over a real source every
    /// unit's built tree must carry exactly unit_facts' node count —
    /// and satisfy the wire contract the core machine-checks.
    #[test]
    fn tree_nodes_equal_unitsig_nodes_per_unit() {
        let text = "fn a(x: i64) -> i64 {\n    let y = x + 1;\n    y * 2\n}\n\
                    \nfn b() {\n    for i in 0..3 {\n        println!(\"{i}\");\n    }\n}\n";
        let facts = unitcache::unit_facts(text, Lang::Rust);
        assert!(!facts.is_empty());
        let segs = units::segments(text, Lang::Rust);
        let spans: Vec<_> = units::with_nth(&segs)
            .iter()
            .map(|(u, _)| (u.start_line, u.end_line))
            .collect();
        let built = file_trees(text, Lang::Rust, &spans);
        assert_eq!(built.len(), facts.len());
        for (f, b) in facts.iter().zip(&built) {
            let Built::Tree(t) = b else {
                panic!("{}: function units are single-rooted", f.key);
            };
            assert_eq!(t.lab.len() as i64, f.nodes, "{}", f.key);
            assert_eq!(*t.lld.last().expect("nonempty"), 0, "{}: root lld", f.key);
            for (i, &l) in t.lld.iter().enumerate() {
                assert!(0 <= l && l <= i as i64, "{}: lld[{i}] = {l}", f.key);
            }
        }
    }

    /// A span holding two sibling items has no single root — the
    /// outcome is a ledgered Forest, never a guessed root.
    #[test]
    fn sibling_span_is_a_forest() {
        let text = "fn a() {}\nfn b() {}\nfn c() {}\n";
        let built = file_trees(text, Lang::Rust, &[(1, 2)]);
        assert!(matches!(built[0], Built::Forest(2)));
    }
}
