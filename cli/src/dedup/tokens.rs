//! Leaf-token extraction + T2 normalization: identifiers → ID,
//! literals → LIT (pieces of ONE literal node collapse into one, and
//! a string, char or user-defined literal node is one token whole —
//! `whole_literal`), comments skipped as whole subtrees, everything
//! else kept as its kind text. Classification rules derive from leaf-kind AST probes
//! of all four grammars, not guesses. Known M2 stances (documented,
//! deliberate): booleans/None are NOT collapsed (their identity is
//! usually semantic, unlike numbers/strings); Go `blank_identifier`
//! normalizes to ID like any identifier.

use crate::scan::ast;
use crate::scan::lang::Lang;
use crate::scan::spec::LangSpec;
use anyhow::{Context, Result};
use tree_sitter::Node;

/// Bump when parser or normalization semantics change: the meta key
/// invalidates parser-derived tables while retaining stamped trend
/// history; stale fingerprints never mix with new measurements.
/// 3: tree-sitter 0.27 — a new parser is a new instrument, so every
/// parser-derived row rebuilds (the five-corpus replay moved nothing).
/// 4: a string, char or user-defined literal node is one LIT whole
/// (plan v2.30 step 5b): a C++ raw string's `R"(` … `)"` pieces and
/// the C family's encoding prefixes (`L"`, `u8"`) were tokens of
/// their own, so a raw literal weighed five where a plain one weighed
/// one; every other grammar's pieces already merged, and their
/// streams are byte-identical across the bump.
pub const TOKENIZER_REV: i64 = 4;

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

/// Normalized token stream of a whole parsed file. Literal pieces
/// merge ONLY within the same parent literal node (attack-review D3:
/// lexical-adjacency merging swallowed whole statements — a Python
/// attribute docstring after a string assignment vanished and its
/// lines ballooned the previous LIT's span).
pub fn tokenize(root: Node<'_>, spec: &LangSpec) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::new();
    let mut lit_parent: Option<usize> = None;
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if spec.comment_kinds.contains(&node.kind()) {
            continue; // lit_parent intentionally unchanged
        }
        if whole_literal(node, spec) {
            lit_parent = None;
            out.push(token(&Class::Lit, node.kind(), node));
            continue;
        }
        if node.child_count() > 0 {
            stack.extend(ast::children(node).into_iter().rev());
            continue;
        }
        match classify(node.kind(), spec) {
            Class::Lit => {
                let parent = node.parent().map(|p| p.id());
                if parent.is_some() && parent == lit_parent {
                    extend_last(&mut out, node);
                } else {
                    lit_parent = parent;
                    out.push(token(&Class::Lit, node.kind(), node));
                }
            }
            class => {
                lit_parent = None;
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
        Class::Text => kind.as_bytes(),
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

/// A literal node read whole as ONE token: a string, char or
/// user-defined literal (C++ [lex.string], [lex.ext]) none of whose
/// named descendants is anything but a literal piece. Its anonymous
/// delimiter tokens — `"`, the C family's `L"` / `u8"` prefixes, the
/// raw string's `R"(` and `)"` — are the literal's own spelling, so
/// they never surface as tokens of their own (plan v2.30 step 5b; a
/// raw string used to weigh five). A literal holding code — a Java
/// `\{expr}` interpolation — fails the test and keeps the piece-merge
/// reading, and a boolean or composite literal is not string-shaped
/// (the M2 stance keeps booleans apart). A leaf is never whole here:
/// it is classified below like any leaf, so its piece-merge reading
/// with a same-parent neighbour is what it was.
fn whole_literal(node: Node<'_>, spec: &LangSpec) -> bool {
    let kind = node.kind();
    if node.child_count() == 0
        || !(kind.ends_with("string_literal")
            || matches!(kind, "char_literal" | "user_defined_literal"))
    {
        return false;
    }
    let mut stack = ast::named_children(node);
    while let Some(piece) = stack.pop() {
        if !is_literal(piece.kind(), spec) {
            return false;
        }
        stack.extend(ast::named_children(piece));
    }
    true
}

fn classify(kind: &str, spec: &LangSpec) -> Class {
    if kind.ends_with("identifier") {
        return Class::Id;
    }
    if is_literal(kind, spec) {
        return Class::Lit;
    }
    Class::Text
}

/// Literal pieces across the grammars (probe-derived): whole literals,
/// string content/fragment/start/end pieces, escape sequences, the
/// C++ raw string's delimiter and the user-defined literal's suffix,
/// and the per-language anonymous delimiter tokens. Precondition:
/// `kind` is a LEAF kind (composite kinds like Go `composite_literal`
/// never reach here — tokenize recurses first) or a named descendant
/// of a string-shaped literal (whole_literal asks the same question
/// of every piece).
pub(crate) fn is_literal(kind: &str, spec: &LangSpec) -> bool {
    kind.ends_with("literal")
        || matches!(
            kind,
            "integer"
                | "float"
                | "number"
                | "escape_sequence"
                | "raw_string_delimiter"
                | "literal_suffix"
        )
        || (kind.contains("string")
            && (kind.ends_with("content")
                || kind.ends_with("fragment")
                || kind.ends_with("start")
                || kind.ends_with("end")))
        || spec.literal_delims.contains(&kind)
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
