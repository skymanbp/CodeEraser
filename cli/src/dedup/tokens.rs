//! Leaf-token extraction + T2 normalization: identifiers → ID,
//! literals → LIT (pieces of ONE literal node collapse into one, and
//! a string, char or user-defined literal node is one token whole —
//! `whole_literal`), comments and the import family skipped as whole
//! subtrees (`skipped`), a docstring holding no code skipped like a
//! comment (rev 6), everything else kept as its kind text. Classification rules derive from leaf-kind AST probes
//! of all four grammars, not guesses. Known M2 stances (documented,
//! deliberate): booleans/None are NOT collapsed (their identity is
//! usually semantic, unlike numbers/strings); Go `blank_identifier`
//! normalizes to ID like any identifier.

use crate::docdup::spec::{doc_spec, docstring};
use crate::scan::ast;
use crate::scan::lang::Lang;
use crate::scan::loads;
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
/// 5: the import family gives no token (v2.35 imports lane): the
/// package's `[scan] import_kinds` declarations and `[scan] loads`
/// statements are skipped whole like a comment, so a run may continue
/// across an import one side lacks.
/// 6: a docstring gives no token (the node docdup reads as one,
/// `docdup::spec::docstring`: the body's first statement past any
/// comment, when a bare string; here only one whose string holds no
/// code): it was one LIT spanning every line of the docstring, so at
/// a file head it joined the statements below the imports into one
/// run and the block named lines it never compared; code equal but
/// for a docstring on one side now matches. Every other language's
/// stream is unchanged.
pub const TOKENIZER_REV: i64 = 6;

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
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&grammar).context("set_language")?;
    let tree = parser.parse(src, None).context("parse")?;
    Ok(tokenize(tree.root_node(), src, lang))
}

/// Normalized token stream of a whole parsed file. Literal pieces
/// merge ONLY within the same parent literal node (attack-review D3:
/// lexical-adjacency merging swallowed whole statements — a Python
/// attribute docstring after a string assignment vanished and its
/// lines ballooned the previous LIT's span).
pub fn tokenize(root: Node<'_>, src: &[u8], lang: Lang) -> Vec<Token> {
    let spec = crate::scan::spec::spec(lang);
    let hosts = doc_spec(lang).docstring_hosts;
    let mut out: Vec<Token> = Vec::new();
    let mut lit_parent: Option<usize> = None;
    // the docstrings of the hosts walked so far, each skipped when met
    let mut docstrings: Vec<usize> = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if skipped(node, spec) || met(&mut docstrings, node) {
            continue; // lit_parent intentionally unchanged
        }
        if whole_literal(node, spec) {
            lit_parent = None;
            out.push(token(&Class::Lit, node.kind(), node));
            continue;
        }
        if node.child_count() > 0 {
            // a host's docstring and a statement list's load statements
            // are read where the host is in hand, so the walk never
            // asks a node for its parent
            let doc =
                docstring(node, hosts, spec.comment_kinds).filter(|s| literal_pieces(*s, spec));
            docstrings.extend(doc.map(|s| s.id()));
            push_children(&mut stack, node, src, lang, spec);
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

/// A comment, or a declaration of the import family
/// (`loads::import_declaration`). Both give no token (v2.35 imports
/// lane: the standing treatment of a comment, extended to name plumbing
/// — the PMD CPD Java lexer and SonarQube's Java statement chunker
/// discard package and import statements the same way).
fn skipped(node: Node<'_>, spec: &LangSpec) -> bool {
    spec.comment_kinds.contains(&node.kind()) || loads::import_declaration(node, spec)
}

/// Whether `node` is a docstring its host put on the list; a docstring
/// is met once, so it leaves the list.
fn met(docstrings: &mut Vec<usize>, node: Node<'_>) -> bool {
    let at = docstrings.iter().position(|id| *id == node.id());
    at.map(|i| docstrings.swap_remove(i)).is_some()
}

/// Push `node`'s children, last first; a statement list's load
/// statements are name plumbing too (scan/loads.rs) and stay off.
fn push_children<'t>(
    stack: &mut Vec<Node<'t>>,
    node: Node<'t>,
    src: &[u8],
    lang: Lang,
    spec: &LangSpec,
) {
    let lists = spec.loads.statement_lists.contains(&node.kind());
    stack.extend(
        ast::children(node)
            .into_iter()
            .rev()
            .filter(|c| !(lists && loads::is_load(*c, src, lang, spec))),
    );
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
    node.child_count() > 0
        && (kind.ends_with("string_literal")
            || matches!(kind, "char_literal" | "user_defined_literal"))
        && literal_pieces(node, spec)
}

/// Every named descendant of `node` is a literal piece: the string
/// holds no code (a whole literal's test, and a docstring's).
fn literal_pieces(node: Node<'_>, spec: &LangSpec) -> bool {
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

/// FNV-1a 64: tiny, dependency-free, stable across runs (the index, the
/// baseline and the T3 cache persist it). The crate's one spelling.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    fnv1a_from(0xcbf2_9ce4_8422_2325, bytes)
}

/// FNV-1a 64 over the fields' bytes with one NUL between two of them —
/// the §7.2 member identity primitive (score/baseline.rs), the config
/// digest and the T3 tree text: equal to `fnv1a` of the joined bytes,
/// without joining them.
pub fn fnv1a_fields(fields: &[&[u8]]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325;
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            h = fnv1a_from(h, &[0]);
        }
        h = fnv1a_from(h, f);
    }
    h
}

fn fnv1a_from(mut h: u64, bytes: &[u8]) -> u64 {
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}
