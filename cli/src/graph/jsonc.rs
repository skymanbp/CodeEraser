//! JSONC cleaning for resolver configs (split from roots.rs per the
//! E01 300-line cap): tsconfig in the wild carries comments and
//! trailing commas that strict serde_json refuses — `clean` strips
//! both, string-aware and char-based, with zero new dependencies.

/// Comments out, trailing commas out — ready for serde_json.
pub fn clean(text: &str) -> String {
    strip_trailing_commas(&strip_comments(text))
}

type Stream<'a> = std::iter::Peekable<std::str::Chars<'a>>;

/// Remove // and /* */ comments outside string literals
/// (char-based: byte indexing would shred multi-byte UTF-8).
fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                out.push('"');
                copy_string(&mut chars, &mut out);
            }
            '/' if chars.peek() == Some(&'/') => skip_line(&mut chars, &mut out),
            '/' if chars.peek() == Some(&'*') => skip_block(&mut chars),
            _ => out.push(c),
        }
    }
    out
}

/// Copy a string literal verbatim through its closing quote.
fn copy_string(chars: &mut Stream, out: &mut String) {
    while let Some(c) = chars.next() {
        out.push(c);
        match c {
            '\\' => out.extend(chars.next()),
            '"' => return,
            _ => {}
        }
    }
}

/// Skip to end of line, keeping the newline itself.
fn skip_line(chars: &mut Stream, out: &mut String) {
    for n in chars.by_ref() {
        if n == '\n' {
            out.push('\n');
            return;
        }
    }
}

/// Skip a /* */ block comment (the '*' peeked by the caller).
fn skip_block(chars: &mut Stream) {
    chars.next();
    let mut prev = ' ';
    for n in chars.by_ref() {
        if prev == '*' && n == '/' {
            return;
        }
        prev = n;
    }
}

/// Remove commas whose next non-whitespace token closes a scope —
/// string-aware, so a literal `"x, }"` value stays intact.
fn strip_trailing_commas(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_str = false;
    let mut escaped = false;
    for (i, c) in text.char_indices() {
        if in_str {
            in_str = escaped || c != '"';
            escaped = !escaped && c == '\\';
        } else if c == '"' {
            in_str = true;
        } else if c == ',' {
            let next = text[i + 1..].chars().find(|n| !n.is_whitespace());
            if matches!(next, Some('}') | Some(']')) {
                continue;
            }
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::clean;

    /// Comments and trailing commas go; string contents — including
    /// a literal that LOOKS like a trailing comma or comment — stay.
    #[test]
    fn strips_jsonc_but_never_string_contents() {
        let dirty = "{\n  // line comment\n  \"a\": \"x, }\", /* block */\n  \"b\": \"//not-a-comment\",\n  \"c\": [1, 2,],\n}\n";
        let value: serde_json::Value = serde_json::from_str(&clean(dirty)).expect("parses");
        assert_eq!(value["a"], "x, }");
        assert_eq!(value["b"], "//not-a-comment");
        assert_eq!(value["c"], serde_json::json!([1, 2]));
    }
}
