//! Markdown reference-site scanner (design brief §4, Markdown row;
//! user decision D3 2026-08-12: link SYNTAX only — bare unbracketed
//! URLs and bare paths in inline code are deliberately not sites;
//! `<http…>` autolinks, images and reference definitions ARE sites,
//! so their later exclusion from the doc graph stays ledger-visible).
//!
//! Fence, inline-code and HTML-comment awareness implement the 2b
//! RED condition: a link-shaped string inside any of them must not
//! emit a site. Inline code pairs backtick RUNS of equal length
//! (CommonMark), not single backticks — the Opus review caught the
//! first draft masking nothing inside ``double`` spans. Nested
//! brackets are depth-matched, so a badge `[![alt](img)](url)` emits
//! the link (url) AND the image (img) instead of one mislabeled
//! site. Indented code blocks are NOT modeled (block context is
//! list-sensitive); their rare link-shaped content stays a site and
//! surfaces in the audit rather than silently.

use super::sites::RawSite;

/// Scan one Markdown document for reference sites.
pub fn detect(text: &str) -> Vec<RawSite> {
    let mut out = Vec::new();
    for (lineno, line, mask) in content_lines(text) {
        scan_line(line, lineno, mask, &mut out);
    }
    out
}

/// content_lines PLUS the inline-code mask merged in — the full
/// triple mask (fence + HTML comment + inline code) as one additive
/// surface. docdup's segment extractor may ONLY call this (F3): the
/// bare walk masks comments but not `code spans`, and a judge seeing
/// text the detector masks would be the drift bug. No existing call
/// path changes — scan_line still merges its own spans. Public: it
/// IS the contracted docdup masking surface (design vol.2 §5.1).
pub fn masked_content_lines(text: &str) -> Vec<(usize, &str, Vec<bool>)> {
    let mut rows = content_lines(text);
    for (_, line, mask) in &mut rows {
        merge_code_spans(line, mask);
    }
    rows
}

/// Fence-aware, comment-masked walk of a document's content lines,
/// shared with the ladder's heading and definition scans — ONE
/// masking implementation: the ladder resolving through a definition
/// the detector refused to see would be the drift bug.
pub(crate) fn content_lines(text: &str) -> Vec<(usize, &str, Vec<bool>)> {
    let mut fence: Option<char> = None;
    let mut in_comment = false;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let t = line.trim_start();
        if !in_comment && let Some(mark) = fence_marker(t) {
            match fence {
                // only the SAME marker closes the fence (``` inside
                // a ~~~ block is content, and vice versa)
                Some(open) if open == mark => fence = None,
                Some(_) => {}
                None => fence = Some(mark),
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        out.push((i + 1, line, comment_mask(line, &mut in_comment)));
    }
    out
}

/// Three-or-more backticks or tildes open/close a fence.
fn fence_marker(trimmed: &str) -> Option<char> {
    ['`', '~']
        .into_iter()
        .find(|&mark| trimmed.chars().take_while(|&c| c == mark).count() >= 3)
}

/// Sites on one content line: reference definitions first (they own
/// the whole line), then bracket links and autolinks outside
/// inline-code spans and HTML comments.
fn scan_line(line: &str, lineno: usize, mut mask: Vec<bool>, out: &mut Vec<RawSite>) {
    merge_code_spans(line, &mut mask);
    if !mask.first().copied().unwrap_or(false)
        && let Some((_, target)) = ref_definition(line)
    {
        out.push(RawSite::md("ref_def", lineno, target.to_string()));
        return;
    }
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if mask[i] {
            i += 1;
            continue;
        }
        match bytes[i] {
            b'[' => i = bracket_site(line, i, lineno, out),
            b'<' => i = autolink_site(line, i, lineno, out),
            _ => i += 1,
        }
    }
}

/// Byte mask of `<!-- … -->` spans, stateful across lines.
fn comment_mask(line: &str, in_comment: &mut bool) -> Vec<bool> {
    let mut mask = vec![false; line.len()];
    let mut i = 0;
    while i < line.len() {
        if *in_comment {
            let end = line[i..].find("-->").map(|p| i + p + 3);
            let stop = end.unwrap_or(line.len());
            mask[i..stop].fill(true);
            if end.is_some() {
                *in_comment = false;
            }
            i = stop;
        } else {
            match line[i..].find("<!--") {
                Some(p) => {
                    *in_comment = true;
                    i += p;
                }
                None => break,
            }
        }
    }
    mask
}

/// Inline-code spans pair backtick RUNS of equal length (CommonMark:
/// a run of N backticks closes only against another run of N).
fn merge_code_spans(line: &str, mask: &mut [bool]) {
    let runs = backtick_runs(line);
    let mut i = 0;
    while i < runs.len() {
        let (start, len) = runs[i];
        match runs[i + 1..].iter().position(|&(_, l)| l == len) {
            Some(offset) => {
                let (close, _) = runs[i + 1 + offset];
                mask[start..close + len].fill(true);
                i += offset + 2;
            }
            None => i += 1,
        }
    }
}

/// (byte start, run length) of every maximal backtick run.
fn backtick_runs(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut runs = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let start = i;
            while i < bytes.len() && bytes[i] == b'`' {
                i += 1;
            }
            runs.push((start, i - start));
        } else {
            i += 1;
        }
    }
    runs
}

/// `[id]: target` at line start (CommonMark link reference
/// definition), as (label, target). The site spec is the TARGET
/// alone — a verbatim substring of the source line (2b exit
/// criterion; the first draft synthesized "id: target" and had to
/// weaken its own test to pass); the label feeds the ladder's R3
/// definition table.
pub(crate) fn ref_definition(line: &str) -> Option<(&str, &str)> {
    let t = line.trim_start();
    let inner = t.strip_prefix('[')?;
    let close = inner.find(']')?;
    let rest = inner[close + 1..].strip_prefix(':')?;
    let target = rest.split_whitespace().next()?;
    (!target.is_empty()).then_some((&inner[..close], target))
}

/// Byte index of the ']' matching the '[' at `open`, depth-aware.
fn matching_close(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, b) in bytes.iter().enumerate().skip(open) {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// `[label](target)` / `![alt](target)` / `[label][id]` starting at
/// byte `start`. Returns start + 1 so nested constructs inside the
/// label (badge images) get their own scan pass.
fn bracket_site(line: &str, start: usize, lineno: usize, out: &mut Vec<RawSite>) -> usize {
    let bytes = line.as_bytes();
    let image = start > 0 && bytes[start - 1] == b'!';
    let Some(close) = matching_close(bytes, start) else {
        return start + 1;
    };
    match bytes.get(close + 1) {
        Some(b'(') => {
            if let Some(end) = find_from(line, close + 2, b')') {
                let target = line[close + 2..end].split_whitespace().next().unwrap_or("");
                if !target.is_empty() {
                    let label = if image { "image" } else { "link" };
                    out.push(RawSite::md(label, lineno, target.to_string()));
                }
            }
        }
        Some(b'[') => {
            if let Some(end) = find_from(line, close + 2, b']') {
                let id = &line[close + 2..end];
                if !id.is_empty() {
                    out.push(RawSite::md("ref_link", lineno, id.to_string()));
                }
            }
        }
        _ => {}
    }
    start + 1
}

/// `<http://…>` autolink at byte `start`.
fn autolink_site(line: &str, start: usize, lineno: usize, out: &mut Vec<RawSite>) -> usize {
    let Some(end) = find_from(line, start + 1, b'>') else {
        return start + 1;
    };
    let inner = &line[start + 1..end];
    if inner.starts_with("http://") || inner.starts_with("https://") || inner.starts_with("mailto:")
    {
        out.push(RawSite::md("url", lineno, inner.to_string()));
    }
    end + 1
}

fn find_from(line: &str, from: usize, needle: u8) -> Option<usize> {
    line.as_bytes()[from..]
        .iter()
        .position(|b| *b == needle)
        .map(|p| from + p)
}

#[cfg(test)]
#[path = "md_tests.rs"]
mod tests;
