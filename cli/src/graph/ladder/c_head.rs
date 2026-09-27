//! C-family include spellings in document order, without a tree-sitter parse.
//! The walk hands these to the ladder for translation-unit attribution, as with
//! java_header.rs; graph/spec.rs INCLUDE reads the same preproc_include path:
//! quoted names lose quotes, angle names keep brackets, macros keep their spelling.
//! https://eel.is/c++draft/lex.phases, phases 2-3: splice backslash-newline first,
//! then read comments as whitespace. A continued line comment therefore continues.
//! https://eel.is/c++draft/cpp.pre, directive grammar: # begins a logical line
//! after whitespace. https://eel.is/c++draft/lex.header, header-name: backslashes
//! are literal, not string escapes. Only include is a site, never include_next
//! or import; conditional evaluation is absent, so #if 0 includes remain.
//! https://eel.is/c++draft/lex.string, raw-string and d-char-sequence: R/u8R/uR/UR/LR
//! prefixes and at most sixteen delimiter characters. Strings, characters, raw
//! strings and comments are opaque to directive recognition.

/// Include specifiers as the site detector spells them, retaining document order.
/// A directive's `#` is the first token of its logical line (C17 §6.10.1);
/// on well-formed source the detector's grammar reads the same sites (the
/// unit twin leg), while a mid-line `#include` — ill-formed, which the
/// detector's error recovery may still read — is outside that contract.
pub fn read(text: &str) -> Vec<String> {
    let text = splice(text.strip_prefix('\u{feff}').unwrap_or(text));
    let mut lex = Lexer {
        rest: &text,
        blank: true,
    };
    let mut out = Vec::new();
    while !lex.rest.is_empty() {
        if let Some((tail, newline)) = trivia(lex.rest) {
            lex.rest = tail;
            lex.blank |= newline;
            continue;
        }
        if lex.blank
            && let Some(rest) = lex.rest.strip_prefix('#')
            && let Some((spec, tail)) = directive(rest)
        {
            out.push(spec);
            lex.rest = tail;
            lex.blank = false;
            continue;
        }
        lex.token();
    }
    out
}

/// Delete original backslash-LF or backslash-CRLF pairs in one pass.
fn splice(text: &str) -> String {
    let mut rest = text;
    let mut out = String::with_capacity(text.len());
    while let Some(c) = rest.chars().next() {
        rest = &rest[c.len_utf8()..];
        if c == '\\'
            && let Some(tail) = rest
                .strip_prefix("\r\n")
                .or_else(|| rest.strip_prefix('\n'))
        {
            rest = tail;
        } else {
            out.push(c);
        }
    }
    out
}

/// Unread logical source and whether its current line has only trivia so far.
struct Lexer<'a> {
    /// Text following the last consumed token or trivia run.
    rest: &'a str,
    /// A directive may start here when no preceding token occupies this line.
    blank: bool,
}

/// Ordinary tokens are opaque, even when a literal contains logical newlines.
impl Lexer<'_> {
    /// Consume a literal, identifier, or single punctuation character.
    fn token(&mut self) {
        self.blank = false;
        if let Some(tail) = raw_tail(self.rest) {
            self.rest = tail;
            return;
        }
        let mut chars = self.rest.chars();
        let Some(c) = chars.next() else { return };
        self.rest = match c {
            '"' | '\'' => literal_tail(chars.as_str(), c),
            c if ident(c) => chars.as_str().trim_start_matches(ident),
            _ => chars.as_str(),
        };
    }
}

/// One whitespace/comment run, retaining whether it crossed a logical newline.
fn trivia(text: &str) -> Option<(&str, bool)> {
    let trimmed = text.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let end = if trimmed.len() != text.len() {
        text.len() - trimmed.len()
    } else if text.starts_with("//") {
        text.find('\n').map_or(text.len(), |at| at + 1)
    } else if let Some(rest) = text.strip_prefix("/*") {
        rest.find("*/").map_or(text.len(), |at| at + 4)
    } else {
        return None;
    };
    Some((&text[end..], text[..end].contains('\n')))
}

/// Spaces and comments between directive tokens cannot cross a logical newline.
fn blanks(mut text: &str) -> &str {
    while let Some((tail, newline)) = trivia(text) {
        if newline {
            return text;
        }
        text = tail;
    }
    text
}

/// A directive past #, including its operand's end so header escapes stay literal.
fn directive(text: &str) -> Option<(String, &str)> {
    let rest = blanks(text).strip_prefix("include")?;
    if rest.starts_with(ident) {
        return None;
    }
    let rest = blanks(rest);
    let first = rest.chars().next()?;
    let line = rest.split('\n').next().unwrap_or_default();
    let (name, end) = match first {
        '"' => {
            let end = line[1..].find('"')? + 1;
            (rest[1..end].to_string(), end + 1)
        }
        '<' => {
            let end = line.find('>')? + 1;
            (rest[..end].to_string(), end)
        }
        c if c == '_' || c.is_alphabetic() => {
            // a macro spelling keeps its argument list, as the
            // detector's preproc_call_expression node does
            let mut end = rest.find(|c| !ident(c)).unwrap_or(rest.len());
            if line[end..].starts_with('(') {
                end += balanced(&line[end..])?;
            }
            (rest[..end].to_string(), end)
        }
        _ => return None,
    };
    Some((name, &rest[end..]))
}

/// The length of the parenthesised list opening a line, None when the
/// line never closes it.
fn balanced(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (at, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(at + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// Identifier characters delimit both the include keyword and macro spelling.
fn ident(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// An ordinary string or character literal, past its opening quote.
fn literal_tail(text: &str, quote: char) -> &str {
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            chars.next();
        } else if c == quote {
            break;
        }
    }
    chars.as_str()
}

/// A valid raw-string opener consumes through its matching terminator, or EOF.
fn raw_tail(text: &str) -> Option<&str> {
    let rest = ["R\"", "u8R\"", "uR\"", "UR\"", "LR\""]
        .into_iter()
        .find_map(|prefix| text.strip_prefix(prefix))?;
    let (delimiter, body) = rest.split_once('(')?;
    if delimiter.chars().count() > 16
        || delimiter.contains(|c: char| c.is_whitespace() || matches!(c, '\\' | ')'))
    {
        return None;
    }
    let closing = format!("){delimiter}\"");
    Some(body.split_once(&closing).map_or("", |(_, tail)| tail))
}

/// Mounted lexical include-reader unit tests.
#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/c_head.rs"]
mod tests;
