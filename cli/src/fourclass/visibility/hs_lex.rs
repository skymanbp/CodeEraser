//! Haskell 2010 §2.3 lexing of a module header's export list (plan
//! v2.17 L round piece (2), criterion H5; every rule below was ruled by
//! GHC 9.10.3 on a compiled fixture, not read off the report alone).
//!
//! Comments come off in ONE left-to-right scan with three states, and
//! the states are not two passes: stripping line comments first eats
//! `-} , bar ) where` out of `( foo {- -- -} , bar )`, stripping block
//! comments first opens an unterminated block on the `{-` inside
//! `( foo -- {-⏎ , bar )` — each order hides a really-exported name.
//!   - code: `{-` opens a block; a run of dashes opens a line comment
//!     under the rule below; whichever comes first wins.
//!   - block: only `{-` / `-}` count, nested (Haskell block comments
//!     nest, and a `{-# … #-}` pragma is swallowed by the same
//!     counter); `--` means nothing here.
//!   - line: nothing means anything until the newline; `{-` included.
//!     A CPP directive line (`#ifdef` at the start of a line) enters
//!     this state too — see `CPP`.
//!
//! The dashes rule is maximal munch. A MAXIMAL run of two or more `-`
//! opens a line comment iff the character right after the run is not
//! a symbol character (or there is none) AND the character right
//! before the run is not a symbol character (or there is none). The
//! first half keeps `-->` an operator; the second is the Haskell 2010
//! lexeme rule read backwards — a dash run preceded by a symbol
//! character is the tail of one varsym or consym lexeme (`<--`,
//! `|--`, `-->--`, `:--`), never a comment — and without it `(<--)`
//! would read as "`(` then a comment" and swallow `, bar ) where`.
//! Because the second half rejects every position inside a run, the
//! scan may advance one character at a time; where it resumes after a
//! rejected run carries no weight.
//!
//! Symbol characters are Haskell 2010 `symbol`: the twenty
//! `ascSymbol` below, spelled as characters — never as a string
//! literal (`\^` does not escape) and never as a regex class (`|-~`
//! reads as a range and drops `-`) — plus every `uniSymbol`, Unicode
//! Symbol or Punctuation outside `special`, `_`, `"`, `'`. `(--⊕)` is
//! a legal operator export, and an ASCII-only table would cut the
//! list off there and hide every name after it.

use unicode_properties::general_category::{GeneralCategoryGroup, UnicodeGeneralCategory};

/// Haskell 2010 `ascSymbol`, the twenty of them.
pub(super) const ASC_SYMBOL: [char; 20] = [
    '!', '#', '$', '%', '&', '*', '+', '.', '/', '<', '=', '>', '?', '@', '\\', '^', '|', '-', '~',
    ':',
];

/// Haskell 2010 `symbol` = `ascSymbol | uniSymbol<special | _ | " | '>`.
/// Every excluded character is ASCII, so the Unicode arm needs no
/// exclusion list of its own.
pub(super) fn is_symbol(c: char) -> bool {
    if c.is_ascii() {
        return ASC_SYMBOL.contains(&c);
    }
    matches!(
        c.general_category_group(),
        GeneralCategoryGroup::Symbol | GeneralCategoryGroup::Punctuation
    )
}

/// The text with every comment removed, newlines kept so a line
/// comment still ends where the source line does.
pub(super) fn strip_comments(text: &str) -> String {
    let ch: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let (mut i, mut depth, mut line) = (0usize, 0usize, false);
    while i < ch.len() {
        let c = ch[i];
        if line {
            if c == '\n' {
                line = false;
                out.push('\n');
            }
            i += 1;
        } else if depth > 0 {
            if opens_block(&ch, i) {
                depth += 1;
                i += 2;
            } else if closes_block(&ch, i) {
                depth -= 1;
                i += 2;
            } else {
                if c == '\n' {
                    out.push('\n');
                }
                i += 1;
            }
        } else if opens_block(&ch, i) {
            depth = 1;
            i += 2;
        } else if let Some(end) = dashes_open_comment(&ch, i) {
            line = true;
            i = end;
        } else if cpp_directive(&ch, i) {
            line = true;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

fn opens_block(ch: &[char], i: usize) -> bool {
    ch[i] == '{' && ch.get(i + 1) == Some(&'-')
}

fn closes_block(ch: &[char], i: usize) -> bool {
    ch[i] == '-' && ch.get(i + 1) == Some(&'}')
}

/// The index past the dash run when the run at `i` opens a line
/// comment under the maximal-munch rule (module doc), else None.
fn dashes_open_comment(ch: &[char], i: usize) -> Option<usize> {
    if ch[i] != '-' {
        return None;
    }
    let end = i + ch[i..].iter().take_while(|&&c| c == '-').count();
    let after_ok = ch.get(end).is_none_or(|&c| !is_symbol(c));
    let before_ok = i == 0 || !is_symbol(ch[i - 1]);
    (end - i >= 2 && after_ok && before_ok).then_some(end)
}

/// The directives the C preprocessor honours under `{-# LANGUAGE CPP
/// #-}`. CPP runs BEFORE the Haskell lexer, so a directive line inside
/// an export list is not Haskell at all; reading it as a line comment
/// yields the union of every conditional branch, which is the safe
/// side (a name exported under some flag keeps its public guard). A
/// MagicHash name never begins with `#`, so the check cannot eat an
/// operator entry.
const CPP: [&str; 13] = [
    "if", "ifdef", "ifndef", "elif", "else", "endif", "define", "undef", "include", "error",
    "warning", "line", "pragma",
];

/// A CPP directive at `i`: the line's first non-blank character is `#`
/// followed by one of the directive names.
fn cpp_directive(ch: &[char], i: usize) -> bool {
    if i != 0 && ch[i - 1] != '\n' {
        return false;
    }
    let hash = i + ch[i..]
        .iter()
        .take_while(|&&c| c == ' ' || c == '\t')
        .count();
    if ch.get(hash) != Some(&'#') {
        return false;
    }
    let word: String = ch[hash + 1..]
        .iter()
        .take_while(|c| c.is_ascii_lowercase())
        .collect();
    CPP.contains(&word.as_str())
}

/// The comment-free list text split into its entries: the outer
/// parentheses come off, then a comma at parenthesis depth zero
/// separates entries (`T(A, B)` stays one entry).
pub(super) fn entries(list: &str) -> Vec<String> {
    let inner = list
        .trim()
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(list);
    let (mut out, mut cur, mut depth) = (Vec::new(), String::new(), 0i32);
    for c in inner.chars() {
        match c {
            ',' if depth == 0 => out.push(std::mem::take(&mut cur)),
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth -= 1;
                cur.push(c);
            }
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out.into_iter()
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect()
}
