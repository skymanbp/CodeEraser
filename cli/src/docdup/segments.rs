//! Segment extraction — the three document-text kinds (design vol.2
//! §5.1), geometry and lines ONLY: exemption policy lives in
//! exempt.rs, wordization in shingle.rs. md_para may come from
//! nothing but `md::masked_content_lines` (F3/RM8): a judge seeing
//! text the detector masks — fence bodies, HTML comments, inline
//! code — would be the drift bug, so the mask rides along into
//! wordization instead of being re-derived. The tree walk is docdup's
//! own additive parse (RM7): sharing the tokenizer's tree would
//! couple TOKENIZER_REV to comment semantics.

use super::spec::{self, KIND_COMMENT, KIND_DOCSTRING, KIND_MD_PARA};
use crate::graph::md;
use crate::scan::lang::Lang;
use crate::scan::{ast, spec as scan_spec};
use tree_sitter::Node;

/// One extracted line: owned text plus the byte mask the md walk
/// carried (None for tree-node lines — they have no masked spans).
pub struct SegLine {
    pub text: String,
    pub mask: Option<Vec<bool>>,
}

/// One raw segment before exemption and admission.
pub struct RawSeg {
    pub kind: i64,
    pub start_line: i64,
    pub end_line: i64,
    pub lines: Vec<SegLine>,
}

/// The line counts md extraction sheds without segments: indented
/// code (NOT modeled — md.rs:14-16) and HTML-markup lines (the
/// 2026-08-14 attainment-line-B amendment, ccm #842) — the ledger
/// keeps both visible instead of silent.
#[derive(Default)]
pub struct MdShed {
    pub indented: u64,
    pub html: u64,
}

/// All raw segments of one file in line order, plus the md shed
/// counts.
pub fn extract(text: &str, lang: Lang) -> (Vec<RawSeg>, MdShed) {
    match lang {
        Lang::Markdown => md_paragraphs(text),
        _ => (tree_segments(text, lang), MdShed::default()),
    }
}

/// Paragraphs = maximal runs of ADJACENT paragraph lines. A fence or
/// heading line never reaches us / is refused, so it breaks
/// adjacency and therefore the paragraph — no state beyond the last
/// line number is needed.
fn md_paragraphs(text: &str) -> (Vec<RawSeg>, MdShed) {
    let (mut segs, mut cur) = (Vec::new(), Vec::<(usize, SegLine)>::new());
    let (mut shed, mut last) = (MdShed::default(), 0usize);
    for (no, line, mask) in md::masked_content_lines(text) {
        let vis = visible(line, &mask);
        if !paragraph_line(&vis) {
            shed.html += u64::from(html_line(&vis));
            flush(&mut cur, &mut segs);
            continue;
        }
        if !cur.is_empty() && no != last + 1 {
            flush(&mut cur, &mut segs);
        }
        if line.starts_with("    ") || line.starts_with('\t') {
            shed.indented += 1;
        }
        let seg_line = SegLine {
            text: line.to_string(),
            mask: Some(mask),
        };
        cur.push((no, seg_line));
        last = no;
    }
    flush(&mut cur, &mut segs);
    (segs, shed)
}

/// The unmasked chars of a line, trimmed — classification must not
/// see HTML-comment or inline-code bytes.
fn visible(line: &str, mask: &[bool]) -> String {
    line.char_indices()
        .filter(|(i, _)| !mask[*i])
        .map(|(_, c)| c)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Paragraph content excludes ATX headings, table rows, lines that
/// are ONLY a list marker (design §5.1; a list item WITH text is
/// content) and HTML-markup lines (2026-08-14 amendment: sponsor
/// tables, badge strips and template headers are structure the
/// lexical judge cannot tell from prose at J ≈ 1.0).
fn paragraph_line(vis: &str) -> bool {
    !vis.is_empty()
        && !vis.starts_with('#')
        && !vis.starts_with('|')
        && !bare_marker(vis)
        && !html_line(vis)
}

/// The CommonMark HTML-block start condition, line-level: `<` then an
/// ASCII letter (open tag) or `/` (close tag). `<!--` never reaches
/// here — the detector masks HTML comments (F3).
fn html_line(vis: &str) -> bool {
    let mut chars = vis.chars();
    chars.next() == Some('<')
        && chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '/')
}

fn bare_marker(vis: &str) -> bool {
    if matches!(vis, "-" | "*" | "+") {
        return true;
    }
    let digits = vis.chars().take_while(char::is_ascii_digit).count();
    digits > 0 && matches!(&vis[digits..], "." | ")")
}

fn flush(cur: &mut Vec<(usize, SegLine)>, segs: &mut Vec<RawSeg>) {
    if cur.is_empty() {
        return;
    }
    let (start, end) = (cur[0].0 as i64, cur[cur.len() - 1].0 as i64);
    segs.push(RawSeg {
        kind: KIND_MD_PARA,
        start_line: start,
        end_line: end,
        lines: cur.drain(..).map(|(_, l)| l).collect(),
    });
}

/// Comment blocks and docstrings off ONE parse of docdup's own.
fn tree_segments(text: &str, lang: Lang) -> Vec<RawSeg> {
    ast::with_tree(text, lang, |tree| {
        let comment_kinds = scan_spec::spec(lang).comment_kinds;
        let hosts = spec::doc_spec(lang).docstring_hosts;
        let (mut comments, mut segs) = (Vec::new(), Vec::new());
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if comment_kinds.contains(&node.kind()) {
                comments.push(node);
                continue;
            }
            if hosts.contains(&node.kind())
                && let Some(s) = docstring_node(node)
            {
                segs.push(node_seg(KIND_DOCSTRING, s, text));
            }
            stack.extend(ast::children(node).into_iter().rev());
        }
        comments.sort_by_key(|n| n.start_byte());
        segs.extend(merge_comments(&comments, text));
        segs.sort_by_key(|s| (s.start_line, s.kind));
        segs
    })
}

/// The docstring string of a host node: the body's first named child
/// is a bare-string expression statement (module bodies ARE the
/// node; function/class bodies sit in the `body` field).
fn docstring_node(host: Node<'_>) -> Option<Node<'_>> {
    let body = match host.kind() {
        "module" => host,
        _ => host.child_by_field_name("body")?,
    };
    let stmt = body.named_child(0)?;
    let s = stmt.named_child(0)?;
    (stmt.kind() == "expression_statement" && s.kind() == "string").then_some(s)
}

/// Contiguous same-column comment nodes merge into one block (design
/// §5.1: a `//` run is one comment per line lexically but one
/// paragraph semantically).
fn merge_comments(nodes: &[Node<'_>], text: &str) -> Vec<RawSeg> {
    let mut out: Vec<RawSeg> = Vec::new();
    let mut last: Option<(usize, usize)> = None; // (end row, start col)
    for n in nodes {
        let (row, col) = (n.start_position().row, n.start_position().column);
        let mergeable = last == Some((row.wrapping_sub(1), col));
        if let (true, Some(prev)) = (mergeable, out.last_mut()) {
            let seg = node_seg(KIND_COMMENT, *n, text);
            prev.end_line = seg.end_line;
            prev.lines.extend(seg.lines);
        } else {
            out.push(node_seg(KIND_COMMENT, *n, text));
        }
        last = Some((n.end_position().row, col));
    }
    out
}

fn node_seg(kind: i64, node: Node<'_>, text: &str) -> RawSeg {
    let lines = text[node.byte_range()]
        .lines()
        .map(|l| SegLine {
            text: l.to_string(),
            mask: None,
        })
        .collect();
    RawSeg {
        kind,
        start_line: node.start_position().row as i64 + 1,
        end_line: node.end_position().row as i64 + 1,
        lines,
    }
}

#[cfg(test)]
#[path = "segments_tests.rs"]
mod tests;
