//! What a grammar reads as opaque to names: the spans of every literal
//! (a string's content declares nothing — the tombstone's fifth replay
//! round bound `independent` out of a caveat message and `linux` out of
//! a cfg string) and, when asked, every comment — blanked to spaces with
//! the newlines kept, so a line still counts and a word-bounded search
//! over the result finds code alone. Two readers share it: the
//! tombstone's marked positions (literals only; it surrenders whole
//! comment lines by the docdup segmenter's reading instead, the
//! under-counting side) and the structure family's seam proxy (both —
//! plan v2.30 step 5b item 30: a unit's name inside a neighbour's string
//! or comment counted as a reference a seam would sever). The walk is
//! the dedup tokenizer's own: a whole `string` / `char` node or a
//! literal leaf (`is_literal`), a comment by the LangSpec's kinds; a
//! source no grammar parses masks nothing.

use crate::dedup::tokens::is_literal;
use crate::scan::ast::children;
use crate::scan::lang::Lang;

/// Which node kinds are blanked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opaque {
    Literals,
    LiteralsAndComments,
}

/// The text with every opaque span blanked to spaces, newlines kept.
pub fn blanked(text: &str, lang: Lang, what: Opaque) -> String {
    let mut bytes = text.as_bytes().to_vec();
    for (a, b) in spans(text, lang, what) {
        for c in &mut bytes[a..b] {
            if *c != b'\n' {
                *c = b' ';
            }
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|_| text.to_string())
}

/// The opaque byte spans in tree order, none nested in another.
pub fn spans(text: &str, lang: Lang, what: Opaque) -> Vec<(usize, usize)> {
    let Some(grammar) = lang.grammar() else {
        return Vec::new();
    };
    let spec = crate::scan::spec::spec(lang);
    let mut parser = tree_sitter::Parser::new();
    let tree = parser
        .set_language(&grammar)
        .ok()
        .and_then(|()| parser.parse(text, None));
    let Some(tree) = tree else {
        return Vec::new();
    };
    let comments = what == Opaque::LiteralsAndComments;
    let mut out = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        let kind = node.kind();
        let leaf = node.child_count() == 0;
        let opaque = matches!(kind, "string" | "char")
            || (leaf && is_literal(kind, spec))
            || (comments && spec.comment_kinds.contains(&kind));
        if opaque {
            out.push((node.start_byte(), node.end_byte()));
        } else if !leaf {
            stack.extend(children(node).into_iter().rev());
        }
    }
    out
}

#[cfg(test)]
#[path = "../../tests/unit/scan/opaque.rs"]
mod tests;
