//! The same-file exceptions of the mention veto (sealed criterion §2,
//! "其他文件" rule; plan v2.17 L round piece (6)). A mention counts
//! only from ANOTHER file — except where the declaring file itself
//! hands the name to a second interpreter, which nothing else can
//! see: a Go `text/template` action inside a string, a TS string or
//! template literal (minus the code inside `${…}`, plus the strings
//! nested in that code), a Python doctest line, a Rust
//! `macro_rules!` body or a fenced or indented block in a run of doc
//! comments (rustdoc compiles both as doctests), a fenced or
//! bird-tracked block in a run of Haskell haddock, a C / C++ string
//! literal (the `dlsym` argument, the method-table entry, the Lua or
//! Python registration name — TS's reason) or a Doxygen code block
//! (`@code`, a fence, an indented block), a Java string (reflection's
//! `getMethod("name")`, the same reason) or a Javadoc code span or
//! Markdown code block. Plain comments and prose
//! of the same file never count (X-5/X-6: a language-neutral rule
//! revived dead code from docstrings in one corpus and killed live
//! code in another).
//!
//! Read lazily and once per file: the token set is built the first
//! time a declaration of the file survives the other-file checks,
//! from the same bytes the allow claim reads (conv/name.rs).

mod doc;

use super::conv::text;
use super::token::{emit, whole_run_only};
use crate::scan::ast;
use crate::scan::lang::Lang;
use doc::Runs;
use std::collections::BTreeSet;
use std::path::Path;
use tree_sitter::Node;

/// One judged file's text and, on demand, its self-mention tokens.
pub struct SelfText {
    rel: String,
    text: String,
    tokens: Option<BTreeSet<String>>,
}

impl SelfText {
    /// The file as it is on disk now, decoded the way its `symbols`
    /// rows were extracted (lossily, dedup/index.rs) — a stray byte
    /// must not void the file's exception regions or its allow claim
    /// while its declarations stand; a vanished file reads as empty,
    /// claims nothing and exempts nothing.
    pub fn read(root: &Path, rel: &str) -> SelfText {
        SelfText {
            rel: rel.to_string(),
            text: std::fs::read(root.join(rel))
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .unwrap_or_default(),
            tokens: None,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether the file's own exception regions spell `name` as one
    /// token of the file's tokenizer arm.
    pub fn mentions(&mut self, name: &str) -> bool {
        if self.tokens.is_none() {
            let mut set = BTreeSet::new();
            let arm = whole_run_only(&self.rel);
            for region in regions(&self.rel, &self.text) {
                emit(&region, arm, &mut |t| {
                    set.insert(t.to_string());
                });
            }
            self.tokens = Some(set);
        }
        self.tokens.as_ref().is_some_and(|s| s.contains(name))
    }
}

/// The text regions the language hands to a second interpreter.
fn regions(rel: &str, source: &str) -> Vec<String> {
    let Some(lang) = Lang::judged_path(Path::new(rel)) else {
        return Vec::new();
    };
    let Some(tree) = ast::parse_lang(source, lang) else {
        return Vec::new();
    };
    let src = source.as_bytes();
    let mut out = Vec::new();
    let mut runs = Runs::new(lang);
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        // the node kinds each language hands to a second interpreter,
        // dispatched HERE so no extractor repeats the guard
        match (lang, node.kind()) {
            (Lang::Go, "interpreted_string_literal" | "raw_string_literal") => {
                go_actions(node, src, &mut out);
            }
            (Lang::TypeScript | Lang::Tsx, "string" | "template_string") => {
                ts_strings(node, src, &mut out);
            }
            (Lang::Python, "string") => py_doctest(node, src, &mut out),
            // a `macro_rules!` body is a second grammar (§2 (b))
            (Lang::Rust, "macro_definition") => out.push(text(node, src).to_string()),
            // every C-family string, raw strings included (a
            // concatenated string's pieces are string nodes of their
            // own and collect themselves)
            (Lang::C | Lang::Cpp, "string_literal" | "raw_string_literal") => {
                out.push(text(node, src).to_string());
            }
            // every Java string, a text block included (one kind)
            (Lang::Java, "string_literal") => out.push(text(node, src).to_string()),
            _ => {}
        }
        // a run of doc comments is a fact about siblings in document
        // order; the stack yields them reversed, so push reversed
        let kids = ast::children(node);
        for child in kids.iter().rev() {
            stack.push(*child);
        }
        for child in kids {
            runs.feed(child, src, &mut out);
        }
    }
    runs.flush(&mut out);
    out
}

/// Go: every `{{ … }}` action inside a string literal.
fn go_actions(node: Node<'_>, src: &[u8], out: &mut Vec<String>) {
    let mut rest = text(node, src);
    while let Some((_, after)) = rest.split_once("{{") {
        let Some((action, tail)) = after.split_once("}}") else {
            break;
        };
        out.push(action.to_string());
        rest = tail;
    }
}

/// TS: `collect(n)` = the bytes of a string or template minus its
/// DIRECT `template_substitution` children; the strings nested inside
/// those substitutions are string nodes of their own and collect
/// themselves when the walk reaches them (§2 (a), L5-F10).
fn ts_strings(node: Node<'_>, src: &[u8], out: &mut Vec<String>) {
    let mut at = node.start_byte();
    for sub in ast::children(node)
        .into_iter()
        .filter(|c| c.kind() == "template_substitution")
    {
        out.push(String::from_utf8_lossy(&src[at..sub.start_byte()]).into_owned());
        at = sub.end_byte();
    }
    out.push(String::from_utf8_lossy(&src[at..node.end_byte()]).into_owned());
}

/// Python: `>>> ` lines of any string, and `... ` lines whose indent
/// (blank = space or tab, compared byte for byte) already opened a
/// `>>> ` line earlier in the SAME string node (§2 (a), L4-F6/L5-F8).
fn py_doctest(node: Node<'_>, src: &[u8], out: &mut Vec<String>) {
    let mut opened: BTreeSet<&str> = BTreeSet::new();
    for content in ast::children(node)
        .into_iter()
        .filter(|c| c.kind() == "string_content")
    {
        for line in text(content, src).lines() {
            let body = line.trim_start_matches([' ', '\t']);
            let indent = &line[..line.len() - body.len()];
            if let Some(rest) = body.strip_prefix(">>> ") {
                opened.insert(indent);
                out.push(rest.to_string());
            } else if let Some(rest) = body.strip_prefix("... ")
                && opened.contains(indent)
            {
                out.push(rest.to_string());
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/mention/selfref_tests.rs"]
mod tests;
