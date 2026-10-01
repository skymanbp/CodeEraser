//! Unit subtrees in the clone/1 wire form (design vol.2 §2.2):
//! postorder kind labels + leftmost-leaf-descendant indices, selected
//! by the SAME predicate as `struct_fp::unit_seq` (named nodes whose
//! 1-based line range sits inside the unit span) — so a built tree's
//! node count EQUALS the cached unitsig.nodes, and the driver asserts
//! that per unit instead of assuming two walks agree. A span whose
//! maximal selected nodes are not exactly one is a forest — a
//! ledgered outcome, never a guessed root.

use super::tree_text::{Tokens, joined_text};
use crate::dedup::{struct_fp, tokens};
use crate::scan::ast;
use crate::scan::lang::Lang;
use tree_sitter::Node;

/// One wire-ready tree. `lab` holds raw fnv1a kind codes — the
/// request-local DENSE mapping happens at wire time, per request. The
/// columns after them are merge/1's (plan v2.31 step 7, booklet §6.2;
/// merge generation 2 ruling R1), filled by the same walk: a leaf's
/// source-text hash (0 on an internal node), each node's position
/// class, each node's byte span (the face reads a hole's text back
/// through it), and each node's own-token and token-text hashes
/// (tree_text.rs). clone/1 never sends them — its request is `lab` and
/// `lld` alone, and its cache keys read nothing else.
#[derive(Default, Clone)]
pub struct UnitTree {
    pub lab: Vec<u64>,
    pub lld: Vec<i64>,
    pub leaf: Vec<u64>,
    pub slot: Vec<u8>,
    pub spans: Vec<(usize, usize)>,
    pub own: Vec<u64>,
    pub text: Vec<u64>,
}

/// Whether a tree carries the grammar's extras (comments): clone/1's
/// judgment does — its trees are the unit_seq selection, node for node —
/// and merge/1's do not (step-7 ruling 3): the token runs behind a T1/T2
/// family hold no comment, so a comment on one member only must not
/// make the members two shapes. Without, an extra is never a top node
/// and its subtree is never emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extras {
    With,
    Without,
}

impl Extras {
    /// Whether a walk that found `walked` nodes agrees with the cached
    /// unit signature's `cached`: exactly with extras, at most without
    /// (the comments left out).
    pub fn agrees(self, walked: i64, cached: i64) -> bool {
        match self {
            Extras::With => walked == cached,
            Extras::Without => walked <= cached,
        }
    }
}

/// One top node of a fragment span (merge/1): its tree, its 1-based
/// inclusive line range — the lines a trimmed run is priced by — and
/// its token stream (tree_text.rs), which a run's synthetic root joins.
#[derive(Clone)]
pub struct Top {
    pub tree: UnitTree,
    pub lines: (usize, usize),
    pub stream: Vec<u8>,
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
pub fn file_trees(text: &str, lang: Lang, spans: &[(usize, usize)], extras: Extras) -> Vec<Built> {
    ast::with_tree(text, lang, |tree| {
        spans
            .iter()
            .map(|&(s, e)| {
                let tops = tops(tree.root_node(), s, e, extras);
                match tops[..] {
                    [_] => Built::Tree(tree_of(&tops, text, lang, extras)),
                    _ => Built::Forest(tops.len()),
                }
            })
            .collect()
    })
}

/// A fragment's tree (merge/1, step-7 ruling 4): a span's maximal
/// selected nodes, however many, under one synthetic root — kind
/// `ce:fragment`, leaf hash 0, position class 4 (other), its span the
/// run's. None where the span selects nothing (or the text does not
/// parse); per span its top nodes with their lines, from ONE parse of
/// the file — the caller trims them to the members' common shape
/// (merge/groups_trim.rs) and hangs the kept run with `fragment_of`.
pub fn file_fragments(
    text: &str,
    lang: Lang,
    spans: &[(usize, usize)],
    extras: Extras,
) -> Vec<Option<Vec<Top>>> {
    ast::parse_lang(text, lang).map_or_else(
        || spans.iter().map(|_| None).collect(),
        |tree| {
            spans
                .iter()
                .map(|&(s, e)| {
                    let tops = tops(tree.root_node(), s, e, extras);
                    let trees: Vec<Top> = tops
                        .iter()
                        .map(|&top| Top {
                            tree: tree_of(&[top], text, lang, extras),
                            lines: lines(top),
                            stream: Tokens::of(top, text, extras).stream(top),
                        })
                        .collect();
                    (!trees.is_empty()).then_some(trees)
                })
                .collect()
        },
    )
}

/// A run of top trees under one synthetic root — kind `ce:fragment`,
/// leaf hash 0, position class 4 (other), its span the first top's
/// start to the last top's end, no own token, its text the tops' token
/// streams joined — and the run's lines, the first top's first line to
/// the last top's last; None for an empty run.
pub fn fragment_of(tops: &[Top]) -> Option<(UnitTree, (usize, usize))> {
    let (first, last) = (tops.first()?, tops.last()?);
    let span = (first.tree.spans.last()?.0, last.tree.spans.last()?.1);
    let run = (first.lines.0, last.lines.1);
    let mut t = UnitTree::default();
    for Top { tree: top, .. } in tops {
        let base = t.lab.len() as i64;
        t.lab.extend(&top.lab);
        t.lld.extend(top.lld.iter().map(|l| l + base));
        t.leaf.extend(&top.leaf);
        t.slot.extend(&top.slot);
        t.spans.extend(&top.spans);
        t.own.extend(&top.own);
        t.text.extend(&top.text);
    }
    let streams: Vec<&[u8]> = tops.iter().map(|top| top.stream.as_slice()).collect();
    t.lab.push(struct_fp::kind_code("ce:fragment"));
    t.lld.push(0);
    t.leaf.push(0);
    t.slot.push(4);
    t.spans.push(span);
    t.own.push(0);
    t.text.push(joined_text(&streams));
    Some((t, run))
}

fn tops(root: Node, start: usize, end: usize, extras: Extras) -> Vec<Node> {
    let mut out = Vec::new();
    maximal(root, start, end, extras, &mut out);
    out
}

/// The postorder run of `tops` with every column: the walk emits `lab`
/// and `lld`, and its visitor fills the rest at the same node — the one
/// traversal clone/1 and merge/1 share (the token columns read one
/// token walk per top, tree_text.rs).
fn tree_of(tops: &[Node], text: &str, lang: Lang, extras: Extras) -> UnitTree {
    let classes = crate::merge::slot::classes(lang);
    let mut t = UnitTree::default();
    let (mut leaf, mut slot, mut spans) = (Vec::new(), Vec::new(), Vec::new());
    let (mut own, mut texts) = (Vec::new(), Vec::new());
    for &top in tops {
        let toks = Tokens::of(top, text, extras);
        emit(top, &mut t, extras, &mut |n: Node, is_leaf: bool| {
            let bytes = &text.as_bytes()[n.byte_range()];
            leaf.push(if is_leaf { tokens::fnv1a(bytes) } else { 0 });
            slot.push(classes.map_or(4, |c| crate::merge::slot::slot_of(c, n)));
            spans.push((n.start_byte(), n.end_byte()));
            own.push(toks.own(n));
            texts.push(toks.text(n));
        });
    }
    (t.leaf, t.slot, t.spans, t.own, t.text) = (leaf, slot, spans, own, texts);
    t
}

/// The maximal selected nodes: named nodes inside [start, end] whose
/// nearest named ancestor is not. Containment is the unit_seq
/// predicate verbatim; descent stops at a hit (position nesting makes
/// the whole subtree selected) and at ranges that cannot intersect.
/// Explicit stack like every other walker in this crate (M5-close
/// review LOW: call-stack recursion made a pathologically deep AST a
/// process abort instead of a judgment). Without extras, an extra is
/// never a top node (nor is anything under it).
fn maximal<'t>(node: Node<'t>, start: usize, end: usize, extras: Extras, out: &mut Vec<Node<'t>>) {
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        let (s, e) = lines(n);
        if e < start || s > end || skipped(n, extras) {
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
/// first-child-return threading, derived instead of threaded. The
/// visitor sees each node as it is emitted, with whether it is a leaf
/// (its lld is its own index): the columns of a caller that wants more
/// than the shape ride this one walk instead of a second. Without
/// extras, an extra child's subtree is skipped whole.
fn emit<'t>(
    node: Node<'t>,
    t: &mut UnitTree,
    extras: Extras,
    on_node: &mut impl FnMut(Node<'t>, bool),
) {
    let mut stack = vec![(node, None)];
    while let Some((n, entered_at)) = stack.pop() {
        let Some(base) = entered_at else {
            stack.push((n, Some(t.lab.len() as i64)));
            let (kids, _) = kids(n, extras);
            stack.extend(kids.into_iter().rev().map(|k| (k, None)));
            continue;
        };
        let idx = t.lab.len() as i64;
        t.lab.push(struct_fp::kind_code(n.kind()));
        t.lld.push(if base < idx { base } else { idx });
        on_node(n, base >= idx);
    }
}

/// A node's children as the tree sees them, looking through anonymous
/// intermediates: its named children (the tree's next nodes) and its
/// anonymous tokens (merge/1's `own` column, tree_text.rs). Today's
/// grammars make anonymous nodes terminals, so the descent costs
/// nothing — it keeps the selected SET identical to the spine's
/// all-children walk by construction, not by grammar accident.
pub(super) fn kids(node: Node, extras: Extras) -> (Vec<Node>, Vec<Node>) {
    let (mut named, mut own) = (Vec::new(), Vec::new());
    let mut stack: Vec<Node> = ast::children(node).into_iter().rev().collect();
    while let Some(c) = stack.pop() {
        if skipped(c, extras) {
            continue;
        }
        if c.is_named() {
            named.push(c);
        } else {
            if c.child_count() == 0 {
                own.push(c);
            }
            stack.extend(ast::children(c).into_iter().rev());
        }
    }
    (named, own)
}

/// An extra (a comment) left out of a tree built without extras.
pub(super) fn skipped(node: Node, extras: Extras) -> bool {
    extras == Extras::Without && node.is_extra()
}

#[cfg(test)]
#[path = "../../../tests/unit/dedup/t3/tree.rs"]
mod tests;
