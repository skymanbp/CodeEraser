//! Leaf-token extraction + T2 normalization: identifiers → ID,
//! literals → LIT (consecutive literal pieces collapse into one),
//! comments skipped, everything else (keywords/punctuation) kept as
//! its kind text. Classification rules derive from leaf-kind AST
//! probes of all four grammars (2026-08-07), not guesses:
//! every identifier-class kind ends in "identifier"; string pieces
//! surface as start/end/content/fragment kinds plus anonymous quote
//! tokens; numbers are `integer`/`float` (Py), `number` (TS — also
//! the type keyword, deliberately merged: T2 covers type
//! substitution), `*_literal` (Rust/Go).

use crate::scan::ast;
use crate::scan::lang::Lang;
use crate::scan::spec::LangSpec;
use anyhow::{Context, Result};
use tree_sitter::Node;

/// One normalized token, carrying its source span for report mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub hash: u64,
    pub start_line: usize, // 1-based
    pub end_line: usize,   // 1-based inclusive
}

const ID_MARK: &[u8] = b"\x01ID";
const LIT_MARK: &[u8] = b"\x02LIT";

enum Class {
    Id,
    Lit,
    Text,
}

/// Parse + tokenize one source in one call (index refresh and the
/// pair-layer stream provider share this path).
pub fn stream(src: &[u8], lang: Lang) -> Result<Vec<Token>> {
    let grammar = lang
        .grammar()
        .context("size-only language has no token stream")?;
    let sp = crate::scan::spec::spec(lang);
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&grammar).context("set_language")?;
    let tree = parser.parse(src, None).context("parse")?;
    Ok(tokenize(tree.root_node(), sp))
}

/// Normalized token stream of a whole parsed file. Comments are
/// skipped as WHOLE SUBTREES — tree-sitter-rust 0.24 line_comment is
/// not a leaf (found by test: its child token leaked into the
/// stream). Consecutive literal pieces merge into the previous LIT
/// (a whole string is one LIT); a comment neither breaks nor joins
/// a run.
pub fn tokenize(root: Node<'_>, spec: &LangSpec) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::new();
    let mut prev_lit = false;
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if spec.comment_kinds.contains(&node.kind()) {
            continue; // prev_lit intentionally unchanged
        }
        if node.child_count() > 0 {
            stack.extend(ast::children(node).into_iter().rev());
            continue;
        }
        match classify(node.kind()) {
            Class::Lit if prev_lit => extend_last(&mut out, node),
            class => {
                prev_lit = matches!(class, Class::Lit);
                out.push(token(&class, node.kind(), node));
            }
        }
    }
    out
}

fn token(class: &Class, kind: &str, node: Node<'_>) -> Token {
    let bytes = match class {
        Class::Id => ID_MARK,
        Class::Lit => LIT_MARK,
        _ => kind.as_bytes(),
    };
    Token {
        hash: fnv1a(bytes),
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
    }
}

fn extend_last(out: &mut [Token], node: Node<'_>) {
    if let Some(last) = out.last_mut() {
        last.end_line = last.end_line.max(node.end_position().row + 1);
    }
}

fn classify(kind: &str) -> Class {
    if kind.ends_with("identifier") {
        return Class::Id;
    }
    if is_literal(kind) {
        return Class::Lit;
    }
    Class::Text
}

/// Literal pieces across the four grammars (probe-derived): whole
/// literals, string content/fragment/start/end pieces, escape
/// sequences, and the anonymous quote delimiter tokens.
fn is_literal(kind: &str) -> bool {
    kind.ends_with("literal")
        || matches!(kind, "integer" | "float" | "number" | "escape_sequence")
        || (kind.contains("string")
            && (kind.ends_with("content")
                || kind.ends_with("fragment")
                || kind.ends_with("start")
                || kind.ends_with("end")))
        || matches!(kind, "\"" | "'" | "`")
}

/// FNV-1a: tiny, dependency-free, stable across runs (index persists).
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}
