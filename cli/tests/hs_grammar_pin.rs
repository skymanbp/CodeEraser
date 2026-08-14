//! M5-3k grammar spike, pinned (design §8.4): tree-sitter-haskell's
//! availability under the core 0.26 ABI was an UNVERIFIED assumption
//! in the locked design — this test is the spike's conclusion made
//! permanent. If a dependency bump ever breaks the language ABI,
//! set_language fails here loudly instead of degrading scan output.

#[test]
fn haskell_grammar_speaks_the_pinned_core_abi() {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_haskell::LANGUAGE.into())
        .expect("tree-sitter-haskell ABI incompatible with pinned tree-sitter core");
    let src = "module M where\n\nf :: Int -> Int\nf x = if x > 0 then x else negate x\n";
    let tree = parser.parse(src, None).expect("parse");
    let root = tree.root_node();
    assert_eq!(root.kind(), "haskell");
    assert!(!root.has_error(), "grammar mis-parses plain Haskell");
    // the two node kinds the scan spec will anchor on: the bind and
    // its conditional body must be visible, else function extraction
    // and CoC have nothing to stand on
    let mut kinds = Vec::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        kinds.push(n.kind());
        for i in (0..n.child_count() as u32).rev() {
            stack.push(n.child(i).unwrap());
        }
    }
    assert!(kinds.contains(&"function"), "no function node: {kinds:?}");
    assert!(
        kinds.contains(&"conditional"),
        "no conditional node: {kinds:?}"
    );
}
