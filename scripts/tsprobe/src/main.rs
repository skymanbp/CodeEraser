//! AST probe for the language-expansion design booklet
//! (docs/reference/language-expansion.md §10): parse one snippet with
//! one of the six pinned grammars and print the tree with field
//! names, so every kind and field the booklet's tables cite can be
//! re-read from a transcript instead of recalled. Not a product
//! surface: nothing in cli/ or core/ reads it, and the tests
//! submodule does not mount it.
//!
//! Usage: `cargo run --release -- <c|cpp|lua|java|r|html> <file>`.
//! The header line carries the grammar's ABI (tree-sitter 0.27 accepts
//! 13–15), whether the parse holds any ERROR node, and the kind count.

use std::fmt::Write as _;
use tree_sitter::{Node, Parser};
use tree_sitter_language::LanguageFn;

/// One grammar per name — the six the booklet pins.
fn grammar(name: &str) -> Option<LanguageFn> {
    Some(match name {
        "c" => tree_sitter_c::LANGUAGE,
        "cpp" => tree_sitter_cpp::LANGUAGE,
        "lua" => tree_sitter_lua::LANGUAGE,
        "java" => tree_sitter_java::LANGUAGE,
        "r" => tree_sitter_r::LANGUAGE,
        "html" => tree_sitter_html::LANGUAGE,
        _ => return None,
    })
}

/// Where a transcript line sits: its depth, and the field name the
/// parent reached the node through (None for a positional child).
struct Slot<'a> {
    depth: usize,
    field: Option<&'a str>,
}

/// One node per line: the node's 1-based start line, then `field: kind`
/// for named nodes, the kind quoted for anonymous tokens, `!!ERROR` on
/// error or missing nodes, and the first forty characters of a leaf's
/// text in angle brackets. The line prefix is what lets a transcript of
/// a whole header be read against the source it came from (the step-2
/// crosscheck reads fmt's format.h this way).
fn dump(out: &mut String, node: Node<'_>, src: &[u8], slot: Slot<'_>) {
    let indent = format!("{:5} {}", node.start_position().row + 1, "  ".repeat(slot.depth));
    let field = slot.field.map(|f| format!("{f}: ")).unwrap_or_default();
    let kind = if node.is_named() {
        node.kind().to_string()
    } else {
        format!("{:?}", node.kind())
    };
    let err = if node.is_error() || node.is_missing() {
        " !!ERROR"
    } else {
        ""
    };
    if node.child_count() == 0 {
        let text: String = String::from_utf8_lossy(&src[node.byte_range()])
            .chars()
            .take(40)
            .collect();
        let text = text.replace('\n', "\\n");
        let _ = writeln!(out, "{indent}{field}{kind}{err}  <{text}>");
        return;
    }
    let _ = writeln!(out, "{indent}{field}{kind}{err}");
    let mut cursor = node.walk();
    for (i, child) in node.children(&mut cursor).enumerate() {
        let slot = Slot {
            depth: slot.depth + 1,
            field: node.field_name_for_child(i as u32),
        };
        dump(out, child, src, slot);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, name, path] = args.as_slice() else {
        eprintln!("usage: tsprobe <c|cpp|lua|java|r|html> <file>");
        std::process::exit(2);
    };
    let Some(language) = grammar(name) else {
        eprintln!("unknown grammar {name}");
        std::process::exit(2);
    };
    let language = tree_sitter::Language::from(language);
    let mut parser = Parser::new();
    parser.set_language(&language).expect("grammar loads");
    let src = std::fs::read(path).expect("snippet readable");
    let tree = parser.parse(&src, None).expect("parse");
    let root = tree.root_node();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "== {name}: abi={} has_error={} node_kinds={}",
        language.abi_version(),
        root.has_error(),
        language.node_kind_count()
    );
    dump(
        &mut out,
        root,
        &src,
        Slot {
            depth: 0,
            field: None,
        },
    );
    print!("{out}");
}
