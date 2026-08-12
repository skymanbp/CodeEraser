//! Markdown reference-site scanner (design brief §4, Markdown row;
//! user decision D3 2026-08-12: link SYNTAX only — bare paths inside
//! inline-code spans are deliberately not sites, the cost is visible
//! in the unresolved ledger rather than inherited silently).
//!
//! Fence and inline-code awareness is a RED condition of sub-
//! milestone 2b: a link-shaped string inside a fenced block or an
//! inline code span must not emit a site. Bare URLs and reference
//! definitions DO emit sites (they later resolve `external` /
//! feed reference links), so their exclusion from the doc graph is
//! ledger-visible, never silent.

use super::sites::RawSite;

/// Scan one Markdown document for reference sites.
pub fn detect(text: &str) -> Vec<RawSite> {
    let mut out = Vec::new();
    let mut fence: Option<char> = None;
    for (i, line) in text.lines().enumerate() {
        let t = line.trim_start();
        if let Some(mark) = fence_marker(t) {
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
        scan_line(line, i + 1, &mut out);
    }
    out
}

/// Three-or-more backticks or tildes open/close a fence.
fn fence_marker(trimmed: &str) -> Option<char> {
    ['`', '~']
        .into_iter()
        .find(|&mark| trimmed.chars().take_while(|&c| c == mark).count() >= 3)
}

/// Sites on one non-fence line: reference definitions first (they
/// own the whole line), then bracket links and autolinks outside
/// inline-code spans.
fn scan_line(line: &str, lineno: usize, out: &mut Vec<RawSite>) {
    if let Some((id, target)) = ref_definition(line) {
        out.push(RawSite::md("ref_def", lineno, format!("{id}: {target}")));
        return;
    }
    let mask = code_span_mask(line);
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

/// `[id]: target` at line start (CommonMark link reference
/// definition). Skips `[id]:` with an empty rest.
fn ref_definition(line: &str) -> Option<(&str, &str)> {
    let t = line.trim_start();
    let inner = t.strip_prefix('[')?;
    let close = inner.find(']')?;
    let rest = inner[close + 1..].strip_prefix(':')?;
    let target = rest.split_whitespace().next()?;
    (!target.is_empty()).then(|| (&inner[..close], target))
}

/// Byte mask of inline-code spans: true = inside backticks.
fn code_span_mask(line: &str) -> Vec<bool> {
    let mut mask = vec![false; line.len()];
    let mut open: Option<usize> = None;
    for (i, b) in line.bytes().enumerate() {
        if b != b'`' {
            continue;
        }
        match open {
            Some(start) => {
                mask[start..=i].fill(true);
                open = None;
            }
            None => open = Some(i),
        }
    }
    mask
}

/// `[text](target)` (or `![alt](target)` = image) and `[text][id]`
/// starting at byte `start`; returns the index scanning resumes at.
fn bracket_site(line: &str, start: usize, lineno: usize, out: &mut Vec<RawSite>) -> usize {
    let bytes = line.as_bytes();
    let image = start > 0 && bytes[start - 1] == b'!';
    let Some(close) = find_from(line, start + 1, b']') else {
        return start + 1;
    };
    match bytes.get(close + 1) {
        Some(b'(') => {
            let Some(end) = find_from(line, close + 2, b')') else {
                return close + 1;
            };
            let target = line[close + 2..end].split_whitespace().next().unwrap_or("");
            if !target.is_empty() {
                let label = if image { "image" } else { "link" };
                out.push(RawSite::md(label, lineno, target.to_string()));
            }
            end + 1
        }
        Some(b'[') => {
            let Some(end) = find_from(line, close + 2, b']') else {
                return close + 1;
            };
            let id = &line[close + 2..end];
            if !id.is_empty() {
                out.push(RawSite::md("ref_link", lineno, id.to_string()));
            }
            end + 1
        }
        _ => close + 1,
    }
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
mod tests {
    use super::detect;

    fn kinds_specs(text: &str) -> Vec<(&'static str, String)> {
        detect(text).into_iter().map(|s| (s.kind, s.spec)).collect()
    }

    /// Sub-milestone 2b RED condition: link-shaped strings inside a
    /// fenced block or an inline code span must not emit a site.
    #[test]
    fn fences_and_code_spans_emit_nothing() {
        let text = "```md\n[fenced](./no.md)\n```\nsee `[coded](./no.md)` too\n~~~\n[tilde](./no.md)\n~~~\n";
        assert_eq!(kinds_specs(text), vec![]);
    }

    /// A ``` inside an open ~~~ fence is content, not a closer.
    #[test]
    fn mismatched_fence_markers_do_not_close() {
        let text = "~~~\n```\n[still fenced](./no.md)\n```\n~~~\n[out](./yes.md)\n";
        assert_eq!(kinds_specs(text), vec![("link", "./yes.md".into())]);
    }

    #[test]
    fn link_kinds_and_specs() {
        let text = "[a](./a.md) ![img](img.png) [b][ref]\n[ref]: ./b.md\n<https://x.example>\n[anchor](#sec)\n";
        assert_eq!(
            kinds_specs(text),
            vec![
                ("link", "./a.md".into()),
                ("image", "img.png".into()),
                ("ref_link", "ref".into()),
                ("ref_def", "ref: ./b.md".into()),
                ("url", "https://x.example".into()),
                ("link", "#sec".into()),
            ]
        );
    }

    /// Anti-invention: every spec is a substring of its source line.
    #[test]
    fn specs_are_line_substrings() {
        let text = "intro [a](./a.md) and <https://x.example>\n[id]: ./target.md \"title\"\n";
        let lines: Vec<&str> = text.lines().collect();
        for site in detect(text) {
            let line = lines[site.line - 1];
            let ok = site.spec.split(' ').all(|part| {
                let part = part.trim_end_matches(':');
                line.contains(part)
            });
            assert!(ok, "spec {:?} not in line {:?}", site.spec, line);
        }
    }
}
