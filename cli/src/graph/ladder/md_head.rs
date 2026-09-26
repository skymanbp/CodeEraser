//! The heading half of the Markdown reader (split from md_slug.rs at
//! the 300 gate — plan v2.30 step 5b, boundary items 20 / 21 / 27 /
//! 35): which lines are headings, what a heading's text is, and the
//! raw-HTML anchors a document offers. One reader serves the anchor
//! set (md_slug.rs) and the section units (fourclass/units.rs) — a
//! section the ladder cannot link to, or an anchor the units do not
//! open, would be the drift bug. Block- and comment-aware through the
//! detector's own walk (graph/md.rs content_lines): a fenced or
//! indented `# x` is no heading, and a commented anchor is no target.

use crate::graph::md::{content_lines, list_item, merge_code_spans};

/// One heading: the row it starts on (a setext heading's first text
/// row) and its text with the markers dropped — the `#` run and the
/// space-separated closing run of an ATX heading, the lines of a
/// setext heading's paragraph joined by one space.
pub(crate) struct Heading {
    pub line: usize,
    pub text: String,
}

/// Every heading the document renders, in order — ATX (CommonMark
/// §4.2: one to six `#`, then a space, a tab or the end of the line)
/// and setext (§4.3: a paragraph followed by a line of `=` or `-`;
/// GitHub slugs both). A setext underline needs a paragraph directly
/// above it: after a blank line, a heading, a list item, a table row,
/// a block quote or an HTML block a `---` is a thematic break, never a
/// heading, and a line masked from its first byte (inside an HTML
/// comment) is neither a heading nor a paragraph.
pub(crate) fn headings(text: &str) -> Vec<Heading> {
    let mut out = Vec::new();
    // the open paragraph's rows, consecutive by construction
    let mut para: Vec<(usize, &str)> = Vec::new();
    for (row, line, mask) in content_lines(text) {
        let trimmed = line.trim_start();
        if mask.first().copied().unwrap_or(false) || trimmed.is_empty() {
            para.clear();
            continue;
        }
        if let Some(head) = atx_heading(trimmed) {
            out.push(Heading {
                line: row,
                text: head.to_string(),
            });
            para.clear();
            continue;
        }
        if para.last().is_none_or(|&(prev, _)| prev + 1 != row) {
            para.clear();
        }
        if !para.is_empty() && setext_underline(line) {
            out.push(setext(&para));
            para.clear();
        } else if paragraph_text(trimmed) {
            para.push((row, line));
        } else {
            para.clear();
        }
    }
    out
}

/// The setext heading a paragraph and its underline make.
fn setext(para: &[(usize, &str)]) -> Heading {
    Heading {
        line: para[0].0,
        text: para
            .iter()
            .map(|(_, l)| l.trim())
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// ATX heading text: 1-6 leading #, then a space, a tab or the end;
/// a closing `#` run strips only where a space or a tab precedes it
/// (CommonMark §4.2 — `# C#` is the heading `C#`, `## Two ##` is
/// `Two`, `# #` is empty).
pub(crate) fn atx_heading(trimmed: &str) -> Option<&str> {
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &trimmed[hashes..];
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }
    let body = rest.trim();
    let kept = body.trim_end_matches('#');
    let closing = kept.len() < body.len() && (kept.is_empty() || kept.ends_with([' ', '\t']));
    Some(if closing { kept.trim_end() } else { body })
}

/// A setext underline: at most three spaces of indent, then a run of
/// `=` or of `-` and trailing whitespace only (CommonMark §4.3).
fn setext_underline(line: &str) -> bool {
    let indent = line.len() - line.trim_start_matches(' ').len();
    let run = line.trim();
    indent <= 3
        && !line.starts_with('\t')
        && !run.is_empty()
        && (run.bytes().all(|b| b == b'=') || run.bytes().all(|b| b == b'-'))
}

/// Whether a non-blank, unmasked, non-heading line is paragraph text a
/// setext underline may close: not a list item, a table row, a block
/// quote, an HTML block opener or an underline-shaped line itself.
fn paragraph_text(trimmed: &str) -> bool {
    let html = trimmed.strip_prefix('<').is_some_and(|rest| {
        rest.starts_with(|c: char| c.is_ascii_alphabetic() || matches!(c, '/' | '!' | '?'))
    });
    list_item(trimmed).is_none()
        && !trimmed.starts_with(['|', '>'])
        && !html
        && !setext_underline(trimmed)
}

/// Raw-HTML anchors as `(row, id)` in document order: `<a name=…>`,
/// `<a id=…>` and `<h1..6 id=…>` (name too), read from the tag's `<`
/// to its `>` across lines where the tag runs over several, the
/// attribute spelled with or without spaces around `=`, its value
/// quoted either way or bare (plan v2.30 step 5b: the one-line,
/// one-word reader missed each of those). Masked bytes — an HTML
/// comment, a code span — never open or continue a tag, and the row
/// is the one the tag opens on.
pub(crate) fn anchors(text: &str) -> Vec<(usize, String)> {
    let (flat, rows) = live_text(text);
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(p) = flat[i..].find('<') {
        let open = i + p;
        i = open + 1;
        let Some(end) = flat[open..].find('>') else {
            break;
        };
        let body = flat[open + 1..open + end].trim_start();
        let name_len = body.find(char::is_whitespace).unwrap_or(body.len());
        let tag = body[..name_len].to_ascii_lowercase();
        let heading =
            tag.len() == 2 && tag.starts_with('h') && tag.ends_with(['1', '2', '3', '4', '5', '6']);
        if tag == "a" || heading {
            out.extend(
                attr_ids(&body[name_len..])
                    .into_iter()
                    .map(|id| (rows[open], id)),
            );
        }
    }
    out
}

/// The document's live text as one string — masked bytes blanked,
/// non-content lines absent, rows joined by newlines — beside the row
/// each byte came from.
fn live_text(text: &str) -> (String, Vec<usize>) {
    let (mut flat, mut rows) = (String::new(), Vec::new());
    for (row, line, mut mask) in content_lines(text) {
        merge_code_spans(line, &mut mask);
        for (at, c) in line.char_indices() {
            flat.push(if mask[at] { ' ' } else { c });
            rows.extend(std::iter::repeat_n(row, c.len_utf8()));
        }
        flat.push('\n');
        rows.push(row);
    }
    (flat, rows)
}

/// The `id` / `name` attribute values of a tag body, in order: an
/// attribute is a name, optional whitespace, `=`, optional whitespace
/// and a value quoted with `"` or `'` or bare up to whitespace or a
/// self-closing `/`; a valueless word is skipped.
fn attr_ids(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut s = body.trim_start();
    while !s.is_empty() {
        let stop = s
            .find(|c: char| c.is_whitespace() || matches!(c, '=' | '/'))
            .unwrap_or(s.len())
            .max(1);
        let (name, tail) = s.split_at(stop);
        let (value, next) = match tail.trim_start().strip_prefix('=') {
            Some(v) => attr_value(v.trim_start()),
            None => (None, tail),
        };
        if matches!(name.to_ascii_lowercase().as_str(), "id" | "name")
            && let Some(v) = value.filter(|v| !v.is_empty())
        {
            out.push(v.to_string());
        }
        s = next.trim_start();
    }
    out
}

/// One attribute value and the rest of the body after it.
fn attr_value(s: &str) -> (Option<&str>, &str) {
    match s.chars().next() {
        Some(q @ ('"' | '\'')) => match s[1..].find(q) {
            Some(end) => (Some(&s[1..1 + end]), &s[2 + end..]),
            None => (Some(&s[1..]), ""),
        },
        _ => {
            let end = s
                .find(|c: char| c.is_whitespace() || c == '/')
                .unwrap_or(s.len());
            (Some(&s[..end]), &s[end..])
        }
    }
}
