//! Tiny AST helpers shared by scan modules. tree-sitter 0.26 mixes
//! usize (`child_count`) with u32 (`child(i)`) — centralize the cast
//! here instead of sprinkling `as u32` through every walker.

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
    kids(node.child_count(), |i| node.child(i))
}

pub fn named_children<'t>(node: Node<'t>) -> Vec<Node<'t>> {
    kids(node.named_child_count(), |i| node.named_child(i))
}

/// Text of a node's `operator` field, if any (uniform across grammars:
/// python boolean_operator, ts/rust/go binary_expression all expose it).
pub fn operator_text<'s>(node: Node<'_>, src: &'s [u8]) -> Option<&'s str> {
    let op = node.child_by_field_name("operator")?;
    op.utf8_text(src).ok()
}
