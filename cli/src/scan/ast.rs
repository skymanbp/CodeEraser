//! Tiny AST helpers shared by scan modules. Tree-sitter 0.27 returns
//! u32 child counts and usize named-child counts; both accessors take
//! u32 indices. Keep the count conversions in these shared walks.

use tree_sitter::Node;

/// Parse one document — the shared prologue of every AST consumer
/// (unit segmentation, graph site detection). None = grammar refused
/// or parse failed; callers degrade to "no structure".
pub fn parse(text: &str, grammar: &tree_sitter::Language) -> Option<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(grammar).ok()?;
    parser.parse(text, None)
}

/// The grammar lookup + parse in one throat — every "facts of one
/// file" extractor (unitcache, docdup) opens with exactly this, and
/// the repo's own ratchet caught the pair re-instantiating it. None =
/// size-only language or parse failure.
pub fn parse_lang(text: &str, lang: crate::scan::lang::Lang) -> Option<tree_sitter::Tree> {
    parse(text, &lang.grammar()?)
}

/// Parse-or-empty: run the extractor on the tree, or return no facts
/// (the guard idiom itself was the second thing the ratchet caught
/// the extractor pair sharing).
pub fn with_tree<T>(
    text: &str,
    lang: crate::scan::lang::Lang,
    extract: impl FnOnce(tree_sitter::Tree) -> Vec<T>,
) -> Vec<T> {
    parse_lang(text, lang).map_or_else(Vec::new, extract)
}

/// One child walk under either accessor — the two spellings below
/// differed in nothing but which pair they called, and the ratchet
/// charged for the repetition.
/// The cast is safe: a single file cannot hold > u32::MAX AST children.
fn kids<'t>(count: usize, at: impl Fn(u32) -> Option<Node<'t>>) -> Vec<Node<'t>> {
    (0..count).filter_map(|i| at(i as u32)).collect()
}

pub fn children<'t>(node: Node<'t>) -> Vec<Node<'t>> {
    kids(node.child_count() as usize, |i| node.child(i))
}

pub fn named_children<'t>(node: Node<'t>) -> Vec<Node<'t>> {
    kids(node.named_child_count(), |i| node.named_child(i))
}

/// A list's entries — its named children, comments aside: a parameter
/// list's parameters, an argument list's arguments. One reading for
/// both, because overload resolution compares the two counts.
pub fn entries<'t>(list: Node<'t>) -> Vec<Node<'t>> {
    named_children(list)
        .into_iter()
        .filter(|c| !c.kind().contains("comment"))
        .collect()
}

/// The parent chain, innermost first, ending at the file root — the
/// one ancestor walk. The scan layer reads it for a Java member's
/// owner (functions::owner_of); the visibility climbs and the mention
/// category word (mention/conv) read the same chain for enclosing
/// classes, ambient blocks and Rust attributes.
pub fn ancestors(node: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    std::iter::successors(node.parent(), |n| n.parent())
}

/// Pre-order nodes under `root`: `kids` yields a node's children and
/// `enter` prunes a subtree (the root itself is never asked). The one
/// walk unit extraction and the per-unit metric walk drive — they
/// differed in nothing but their two closures, and the ratchet
/// charged for the repeated loop.
pub fn preorder<'t>(
    root: Node<'t>,
    enter: impl Fn(Node<'t>) -> bool,
    kids: impl Fn(Node<'t>) -> Vec<Node<'t>>,
) -> Vec<Node<'t>> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.id() != root.id() && !enter(node) {
            continue;
        }
        out.push(node);
        stack.extend(kids(node).into_iter().rev());
    }
    out
}

/// Text of a node's `operator` field, if any (uniform across grammars:
/// python boolean_operator, ts/rust/go binary_expression all expose it).
pub fn operator_text<'s>(node: Node<'_>, src: &'s [u8]) -> Option<&'s str> {
    let op = node.child_by_field_name("operator")?;
    op.utf8_text(src).ok()
}
