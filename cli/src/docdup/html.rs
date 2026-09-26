//! HTML text segments (plan v2.30 step 5, register D20): a block
//! element's prose — a paragraph, a list item, a heading (BLOCK) — is
//! one segment the way a Markdown paragraph is, and its lines are the
//! element's SOURCE lines under a byte mask that leaves only its text
//! leaves visible. Markup, attributes, entities and comments never
//! reach the wordizer; nor does the text of a nested block element (a
//! segment of its own), inline code (`code`, masked like an md code
//! span) or preformatted and scripted content (`pre`, `textarea`,
//! `script`, `style`). The document's own comments are masked like a
//! Markdown file's, not segmented — they are not prose a reader sees
//! — and that is exactly what lets a `ce:allow(docdup) -- why` marker
//! written in an HTML comment inside the element stay on the raw line
//! for exempt::classify to read. The shed counters (MdShed) count the
//! code and script elements the walk met, whether it masked or
//! skipped them.

use super::segments::{MdShed, RawSeg, SegLine};
use super::spec::KIND_HTML_TEXT;
use crate::scan::ast::{self, children};
use crate::scan::html::tag_in;
use crate::scan::lang::Lang;
use std::cell::Cell;
use tree_sitter::Node;

/// The elements whose text descendants form one segment (booklet §9:
/// the block-level prose containers; a heading is a segment like a
/// paragraph, a table cell like a list item).
const BLOCK: &str =
    "p li dt dd td th h1 h2 h3 h4 h5 h6 blockquote figcaption caption summary label legend title";

/// The elements whose content is code: masked inside a segment, never
/// a segment outside one.
const CODE: &str = "pre code textarea";

/// All text segments of one document in line order, plus the shed
/// counts — the Markdown extractor's contract (segments::extract). The
/// walk never enters script, style or code content, and counts each
/// such element it meets on the counter that element feeds.
pub(super) fn segments(text: &str) -> (Vec<RawSeg>, MdShed) {
    let lines = Lines::new(text);
    let (code, script) = (Cell::new(0), Cell::new(0));
    let segs = ast::with_tree(text, Lang::Html, |tree| {
        let src = text.as_bytes();
        let enter = |node: Node| {
            let shed = match node.kind() {
                "script_element" | "style_element" => &script,
                "element" if tag_in(node, src, CODE) => &code,
                _ => return true,
            };
            shed.set(shed.get() + 1);
            false
        };
        ast::preorder(tree.root_node(), enter, children)
            .into_iter()
            .filter(|&node| node.kind() == "element" && tag_in(node, src, BLOCK))
            .map(|node| block(node, src, &lines))
            .collect()
    });
    let shed = MdShed {
        code: code.get(),
        script: script.get(),
        ..MdShed::default()
    };
    (segs, shed)
}

/// One block element as a segment: its source lines, each masked
/// except for the text leaves that belong to it — not to a nested
/// block (its own segment), nor to code, script or style content.
fn block(element: Node, src: &[u8], lines: &Lines) -> RawSeg {
    let (first, last) = (element.start_position().row, element.end_position().row);
    let mut masks: Vec<Vec<bool>> = (first..=last).map(|r| vec![true; lines.len(r)]).collect();
    for leaf in ast::preorder(element, |node| !opaque(node, src), children) {
        if leaf.kind() == "text" {
            lines.unmask(leaf, first, &mut masks);
        }
    }
    RawSeg {
        kind: KIND_HTML_TEXT,
        start_line: first as i64 + 1,
        end_line: last as i64 + 1,
        lines: (first..=last)
            .zip(masks)
            .map(|(row, mask)| SegLine {
                text: lines.text(row).to_string(),
                mask: Some(mask),
            })
            .collect(),
    }
}

/// Whether a node's text belongs elsewhere: to a nested block (a
/// segment of its own), or to code, script or style content.
fn opaque(node: Node, src: &[u8]) -> bool {
    matches!(node.kind(), "script_element" | "style_element")
        || (node.kind() == "element" && (tag_in(node, src, BLOCK) || tag_in(node, src, CODE)))
}

/// The document's rows as (byte offset, content) pairs — tree-sitter
/// counts rows by `\n`, and a row's content excludes its terminator
/// (`\n`, or `\r\n`), so a mask indexes the bytes of the line as
/// shingle::line_words sees it.
struct Lines<'a> {
    rows: Vec<(usize, &'a str)>,
}

impl<'a> Lines<'a> {
    fn new(text: &'a str) -> Self {
        let mut rows = Vec::new();
        let mut at = 0;
        for piece in text.split_inclusive('\n') {
            let content = piece.trim_end_matches('\n').trim_end_matches('\r');
            rows.push((at, content));
            at += piece.len();
        }
        Lines { rows }
    }

    fn len(&self, row: usize) -> usize {
        self.rows.get(row).map_or(0, |(_, c)| c.len())
    }

    fn text(&self, row: usize) -> &'a str {
        self.rows.get(row).map_or("", |(_, c)| c)
    }

    /// Clear the mask over the bytes of a text leaf, row by row (a
    /// leaf may span lines), inside the block's own row window.
    fn unmask(&self, leaf: Node, first: usize, masks: &mut [Vec<bool>]) {
        let (start, end) = (leaf.start_byte(), leaf.end_byte());
        for row in leaf.start_position().row..=leaf.end_position().row {
            let Some(mask) = row.checked_sub(first).and_then(|i| masks.get_mut(i)) else {
                continue;
            };
            let Some(&(line_start, content)) = self.rows.get(row) else {
                continue;
            };
            let from = start.max(line_start);
            let to = end.min(line_start + content.len());
            for b in from..to {
                mask[b - line_start] = false;
            }
        }
    }
}
