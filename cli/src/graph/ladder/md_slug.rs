//! The anchor-set half of the Markdown ladder (split from md.rs at
//! the 300 gate — plan v2.17 L round step 8, O57): the slugs a
//! document offers and the hash coupling them to the resolve key. The
//! step closed four stated limits: a heading is slugged from its
//! RENDERED text (`render_text` — GitHub slugs what the heading
//! shows, not its source), indented code offers no heading
//! (md_mask.rs), raw-HTML anchors — `<h1..6 id=…>`, `<a name=…>`,
//! `<a id=…>` — enter the set verbatim (they are GitHub targets, the
//! audited FAQ.md row's shape), and a fragment or path is
//! percent-decoded before the lookup. Every remaining approximation
//! still degrades an anchor to file level, never invents a section.

use crate::graph::md::{content_lines, matching_close, merge_code_spans};
use std::collections::BTreeMap;

/// The slug set folded to one resolve_key input (module header):
/// order-sensitive on purpose — duplicate -N suffixes shift with
/// order, and the set IS what anchor() consults.
pub fn slug_hash(text: &str) -> u64 {
    let mut buf = Vec::new();
    for slug in slug_set(text) {
        buf.extend_from_slice(slug.as_bytes());
        buf.push(b'\n');
    }
    crate::dedup::tokens::fnv1a(&buf)
}

/// GitHub-slugged ATX headings in document order, -N suffixes for
/// duplicates, plus every raw-HTML anchor id verbatim (an explicit id
/// sits outside the heading counter; two equal ids simply match
/// twice and degrade); block- and comment-aware via the detector's
/// walk, and an anchor tag inside an inline code span is text, not a
/// target (the step-8 review: the comment-only mask let it in). A
/// heading line can carry an anchor tag too — its own slug drops the
/// tag, the id still enters.
pub(super) fn slug_set(text: &str) -> Vec<String> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut out = Vec::new();
    for (_, line, mask) in content_lines(text) {
        if mask.first().copied().unwrap_or(false) {
            continue;
        }
        if let Some(head) = atx_heading(line.trim_start()) {
            // the rendered text is trimmed like GitHub trims it — a
            // dropped trailing tag must not leave a hyphen behind
            let base = slugify(render_text(head).trim());
            let n = seen.entry(base.clone()).or_insert(0usize);
            out.push(if *n == 0 { base } else { format!("{base}-{n}") });
            *n += 1;
        }
        let mut spans = mask;
        merge_code_spans(line, &mut spans);
        out.extend(html_anchors(line, &spans));
    }
    out
}

/// ATX heading text: 1-6 leading #, then a space, a tab or the end;
/// the space-separated closing sequence strips (CommonMark).
fn atx_heading(trimmed: &str) -> Option<&str> {
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &trimmed[hashes..];
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }
    Some(rest.trim().trim_end_matches('#').trim_end())
}

/// The heading as rendered: a backslash escape unescapes, a code
/// span keeps its content, `[text](dest)` / `[text][id]` keep the
/// text, `![alt](dest)` keeps the alt, an inline HTML tag or comment
/// drops, `*` drops always (a delimiter, or a literal the slug drops
/// anyway) and `_` drops only where it PAIRS as emphasis
/// (`emphasis_underscores`): `snake_case` and an unpaired
/// `_private_helper()` keep their underscores, `_em_` does not.
fn render_text(head: &str) -> String {
    let bytes = head.as_bytes();
    let emphasis = emphasis_underscores(bytes);
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let step = match bytes[i] {
            b'\\' if bytes.get(i + 1).is_some_and(u8::is_ascii_punctuation) => {
                out.push(bytes[i + 1] as char);
                2
            }
            b'`' => code_span(head, i, &mut out),
            b'<' => tag(head, i, &mut out),
            b'[' => bracket(head, i, &mut out),
            b'!' if bytes.get(i + 1) == Some(&b'[') => bracket(head, i + 1, &mut out) + 1,
            b'*' => 1,
            b'_' if emphasis[i] => 1,
            _ => {
                let c = head[i..].chars().next().expect("in bounds");
                out.push(c);
                c.len_utf8()
            }
        };
        i += step;
    }
    out
}

/// A backtick run of N closes against the next run of exactly N; an
/// unclosed run renders literally. Returns the bytes consumed.
fn code_span(head: &str, i: usize, out: &mut String) -> usize {
    let n = head[i..].bytes().take_while(|&b| b == b'`').count();
    let body = i + n;
    let mut j = body;
    while let Some(p) = head[j..].find('`') {
        let at = j + p;
        let m = head[at..].bytes().take_while(|&b| b == b'`').count();
        if m == n {
            out.push_str(&head[body..at]);
            return at + m - i;
        }
        j = at + m;
    }
    out.push_str(&head[i..body]);
    n
}

/// `<tag …>` / `</tag>` and a closed `<!-- … -->` drop whole; a `<`
/// that opens neither is text.
fn tag(head: &str, i: usize, out: &mut String) -> usize {
    if let Some(comment) = head[i..].strip_prefix("<!--")
        && let Some(end) = comment.find("-->")
    {
        return end + 7;
    }
    let opens = head
        .as_bytes()
        .get(i + 1)
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'/');
    match (opens, head[i..].find('>')) {
        (true, Some(end)) => end + 1,
        _ => {
            out.push('<');
            1
        }
    }
}

/// `[text]` keeps its rendered text and swallows an immediately
/// following `(dest)` or `[id]`; an unmatched `[` is text.
fn bracket(head: &str, open: usize, out: &mut String) -> usize {
    let bytes = head.as_bytes();
    let Some(close) = matching_close(bytes, open) else {
        out.push('[');
        return 1;
    };
    out.push_str(&render_text(&head[open + 1..close]));
    let tail = match bytes.get(close + 1) {
        Some(b'(') => head[close + 1..].find(')'),
        Some(b'[') => head[close + 1..].find(']'),
        _ => None,
    };
    tail.map_or(close + 1, |end| close + end + 2) - open
}

/// The underscores that delimit emphasis, PAIRED: CommonMark's
/// left/right-flanking classes with the intraword rule decide which
/// can open and which can close, and an opener drops only against a
/// closer after it — `_em_` and `__x__` drop, `_private_helper()`
/// and `foo_` render their underscore (the step-8 review: a flanking
/// but unpaired `_` used to vanish), `snake_case` never opens.
fn emphasis_underscores(bytes: &[u8]) -> Vec<bool> {
    // 0 = boundary or whitespace, 1 = punctuation, 2 = word
    let class = |b: Option<&u8>| match b {
        None => 0,
        Some(b) if b.is_ascii_whitespace() => 0,
        Some(b) if b.is_ascii_punctuation() => 1,
        Some(_) => 2,
    };
    let mut paired = vec![false; bytes.len()];
    let mut openers: Vec<usize> = Vec::new();
    for (i, _) in bytes.iter().enumerate().filter(|(_, b)| **b == b'_') {
        let (prev, next) = (class(bytes.get(i.wrapping_sub(1))), class(bytes.get(i + 1)));
        let left = next != 0 && (next != 1 || prev != 2);
        let right = prev != 0 && (prev != 1 || next != 2);
        let closes = right && (!left || next == 1);
        if closes && let Some(open) = openers.pop() {
            paired[open] = true;
            paired[i] = true;
        } else if left && (!right || prev == 1) {
            openers.push(i);
        }
    }
    paired
}

/// `<a name="x">`, `<a id="x">`, `<h1..6 id="x">` (name too) on one
/// line, unmasked bytes only: the tag body up to `>` is read for an
/// `id=`/`name=` attribute, quoted either way or bare.
fn html_anchors(line: &str, mask: &[bool]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(p) = line[i..].find('<') {
        let open = i + p;
        i = open + 1;
        if mask[open] {
            continue;
        }
        let Some(end) = line[open..].find('>') else {
            break;
        };
        let mut words = line[open + 1..open + end].split_whitespace();
        let tag = words.next().unwrap_or("").to_ascii_lowercase();
        let heading =
            tag.len() == 2 && tag.starts_with('h') && tag.ends_with(['1', '2', '3', '4', '5', '6']);
        if tag == "a" || heading {
            out.extend(words.filter_map(attr_id));
        }
    }
    out
}

/// `id=…` / `name=…` → the value, a self-closing slash and one quote
/// pair stripped.
fn attr_id(word: &str) -> Option<String> {
    let (key, value) = word.split_once('=')?;
    if !matches!(key.to_ascii_lowercase().as_str(), "id" | "name") {
        return None;
    }
    let value = value.trim_end_matches('/').trim_matches(['"', '\'']);
    (!value.is_empty()).then(|| value.to_string())
}

/// GitHub slug: lowercase, keep letters/digits/_/-, spaces become
/// hyphens, everything else drops (verified against the audited
/// Hangul, backtick and parenthesis ground-truth rows).
fn slugify(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            out.extend(c.to_lowercase());
        } else if c == ' ' {
            out.push('-');
        }
    }
    out
}

/// `%XX` escapes decoded — a destination is percent-encoded in the
/// source and plain in the tree; an escape that is not two hex digits
/// stays as written, and a result that is not UTF-8 leaves the whole
/// text untouched — never a guess.
pub(super) fn percent_decode(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = (bytes[i] == b'%')
            .then(|| bytes.get(i + 1..i + 3))
            .flatten()
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match hex {
            Some(b) => {
                out.push(b);
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}
