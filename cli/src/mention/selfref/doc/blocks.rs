//! The code blocks of a doc run (split from doc.rs when plan v2.30
//! step 3's indented-block reader pushed it past 300 lines): the
//! forms each language's doc tool renders as code, read from the
//! run's lines with their comment markers already stripped.

use crate::scan::lang::Lang;

/// The code blocks of a doc run, in the forms the language's doc tool
/// renders as code — the safe direction is more mentions, so every
/// form counts. The three Markdown readers (rustdoc, Doxygen, a JDK 23
/// Markdown doc comment) also render an indented block; haddock is no
/// Markdown, so an indented haddock line stays prose. Java reads the
/// Markdown forms on every run: one run may hold both comment forms,
/// and in a `/** */` run they only ever add mentions.
pub(super) fn code_blocks(lang: Lang, lines: &[String]) -> Vec<String> {
    match lang {
        Lang::Java => [javadoc(lines), indented(lines)].concat(),
        Lang::C | Lang::Cpp => [doxygen(lines), indented(lines)].concat(),
        Lang::Rust => [fenced(lines), indented(lines)].concat(),
        _ => fenced(lines),
    }
}

/// Javadoc: the `<pre>` blocks and the `{@code …}` / `{@snippet …}`
/// spans the javadoc tool renders as code, and the fences of a
/// Markdown doc comment (`///`, JDK 23).
fn javadoc(lines: &[String]) -> Vec<String> {
    let doc = lines.join("\n");
    let mut out = pre_blocks(&doc);
    for tag in ["{@code", "{@snippet"] {
        out.extend(inline_tags(&doc, tag));
    }
    out.extend(between(lines, is_fence, is_fence));
    out
}

/// Doxygen: `@code` … `@endcode` blocks under either command prefix,
/// and Markdown fences.
fn doxygen(lines: &[String]) -> Vec<String> {
    let opens = |t: &str| t.starts_with("@code") || t.starts_with("\\code");
    let closes = |t: &str| t.starts_with("@endcode") || t.starts_with("\\endcode");
    let mut out = between(lines, opens, closes);
    out.extend(between(lines, is_fence, is_fence));
    out
}

/// The lines strictly between an opening line and the next closing one
/// (trimmed text tested).
fn between(
    lines: &[String],
    opens: impl Fn(&str) -> bool,
    closes: impl Fn(&str) -> bool,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in lines {
        let t = line.trim();
        if inside && closes(t) {
            inside = false;
        } else if !inside && opens(t) {
            inside = true;
        } else if inside {
            out.push(line.clone());
        }
    }
    out
}

fn is_fence(t: &str) -> bool {
    t.starts_with("```") || t.starts_with("~~~")
}

/// CommonMark's indented code block (spec 0.31.2 §4.4) — rustdoc
/// compiles one as a doctest, Doxygen and a Markdown doc comment render
/// one as code: lines four columns past the run's common indent (the
/// indent rustdoc and JEP 467 strip first), in chunks that start the
/// run or follow a blank line, a heading or a fence. An indented line
/// never interrupts a paragraph, which keeps a wrapped `@param` line
/// prose; a list item's indented paragraph reads as code here, the
/// safe direction. A tab counts four columns, the most a tab stop
/// advances.
fn indented(lines: &[String]) -> Vec<String> {
    let columns = |l: &str| {
        l.chars()
            .map_while(|c| match c {
                ' ' => Some(1),
                '\t' => Some(4),
                _ => None,
            })
            .sum::<usize>()
    };
    let base = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| columns(l))
        .min()
        .unwrap_or(0);
    let mut out = Vec::new();
    let (mut code, mut open) = (false, true);
    for line in lines {
        let t = line.trim();
        if t.is_empty() {
            open = true;
        } else if (code || open) && columns(line) >= base + 4 {
            code = true;
            out.push(line.clone());
        } else {
            code = false;
            open = is_heading(t) || is_fence(t);
        }
    }
    out
}

/// An ATX heading: one to six `#`, then a space, a tab or nothing.
fn is_heading(t: &str) -> bool {
    let rest = t.trim_start_matches('#');
    (1..=6).contains(&(t.len() - rest.len())) && (rest.is_empty() || rest.starts_with([' ', '\t']))
}

/// `<pre …>` … `</pre>` spans, the tags matched ASCII-case-blind as HTML
/// matches them (lower-casing keeps every byte offset).
fn pre_blocks(doc: &str) -> Vec<String> {
    let lower = doc.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(found) = lower[at..].find("<pre") {
        let start = at + found + "<pre".len();
        if !lower[start..].starts_with(['>', ' ', '\t', '\n']) {
            at = start;
            continue;
        }
        let Some(gt) = lower[start..].find('>') else {
            break;
        };
        let body = start + gt + 1;
        let Some(close) = lower[body..].find("</pre>") else {
            break;
        };
        out.push(doc[body..body + close].to_string());
        at = body + close + "</pre>".len();
    }
    out
}

/// The bodies of `{@tag …}` spans, braces balanced (a `{@code}` body
/// may hold braces of its own).
fn inline_tags(doc: &str, tag: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = doc;
    while let Some(found) = rest.find(tag) {
        let body = &rest[found + tag.len()..];
        let mut depth = 1;
        let end = body.char_indices().find_map(|(i, c)| {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
            (depth == 0).then_some(i)
        });
        let Some(end) = end else {
            break;
        };
        out.push(body[..end].to_string());
        rest = &body[end + 1..];
    }
    out
}

/// Rust and Haskell: markdown fences, haddock `@` blocks and `>` bird
/// tracks — the safe direction is more mentions, so all three forms
/// count.
fn fenced(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let (mut fence, mut at) = (false, false);
    for line in lines {
        let t = line.trim();
        if t.starts_with("```") || t.starts_with("~~~") {
            fence = !fence;
        } else if t == "@" {
            at = !at;
        } else if fence || at {
            out.push(line.clone());
        } else if let Some(bird) = t.strip_prefix('>') {
            out.push(bird.to_string());
        }
    }
    out
}
