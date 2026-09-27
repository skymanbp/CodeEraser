//! The marked positions of a text (spec §三 M1): where a document
//! DECLARES or STRUCTURES a name — a code line's identifier, a
//! declared unit's name, a heading, a list item's lead — and where it
//! merely MENTIONS one (an inline code span). Only a structural
//! position spells a name; a mention keeps one alive. The third
//! self-replay round said why: a long narrative line rewritten in
//! place drops its own spans and re-mentions them, and nothing was
//! removed — you cannot remove what was never declared, only stop
//! mentioning it.

use super::frames::words;
use crate::docdup::segments;
use crate::fourclass::units;
use crate::graph::ladder::md::head::atx_heading;
use crate::graph::ladder::md::slug::render_text;
use crate::graph::md::{content_lines, merge_code_spans};
use crate::mention::token::runs;
use crate::scan::lang::Lang;
use std::collections::BTreeSet;

/// One marked text of a document: its line, the text, and whether the
/// position is structural (declares) or a mention (keeps alive).
pub(super) struct Marked {
    pub(super) line: usize,
    pub(super) text: String,
    pub(super) structural: bool,
}

pub(super) fn marked(text: &str, lang: Lang) -> Vec<Marked> {
    match lang {
        Lang::Markdown => md_marked(text),
        _ => code_marked(text, lang),
    }
}

/// A code file's marked texts: every identifier on a line no comment
/// or docstring segment covers (a trailing comment surrenders its
/// whole line — the under-counting side) and outside every literal
/// (a string's content declares nothing: the fifth replay round bound
/// `independent` out of a caveat message and `linux` out of a cfg
/// string), plus every declared unit's name.
fn code_marked(text: &str, lang: Lang) -> Vec<Marked> {
    let (segs, _) = segments::extract(text, lang);
    let prose: BTreeSet<usize> = segs
        .iter()
        .flat_map(|s| (s.start_line as usize)..=(s.end_line as usize))
        .collect();
    let structural = |line, text: &str| Marked {
        line,
        text: text.to_string(),
        structural: true,
    };
    let masked = without_literals(text, lang);
    let mut out: Vec<Marked> = masked
        .lines()
        .enumerate()
        .filter(|(i, _)| !prose.contains(&(i + 1)))
        .flat_map(|(i, l)| runs(l).map(move |r| structural(i + 1, r)))
        .collect();
    let units = units::segments(text, lang);
    out.extend(
        units
            .iter()
            .map(|u| structural(u.start_line, name_part(&u.key))),
    );
    out
}

/// The text with every literal blanked to spaces, newlines kept
/// (scan::opaque, the grammar walk the dedup tokenizer makes); a
/// source no grammar parses masks nothing.
fn without_literals(text: &str, lang: Lang) -> String {
    crate::scan::opaque::blanked(text, lang, crate::scan::opaque::Opaque::Literals)
}

/// A unit key's name half (`alpha/2` → `alpha`; a Rust impl key has
/// no arity and stays whole).
pub fn name_part(key: &str) -> &str {
    key.rsplit_once('/').map_or(key, |(name, _)| name)
}

/// A Markdown document's marked texts: each heading as rendered and
/// each list item's lead (structural: these declare), the inside of
/// each inline code span (a mention: it keeps alive). Fenced and
/// indented code never reach content_lines, so an example's tokens
/// are neither.
fn md_marked(text: &str) -> Vec<Marked> {
    let mut out = Vec::new();
    for (line, raw, comment) in content_lines(text) {
        let t = raw.trim_start();
        let lead = atx_heading(t).map(render_text).or_else(|| list_lead(t));
        out.extend(lead.map(|text| Marked {
            line,
            text,
            structural: true,
        }));
        let mut spans = comment.clone();
        merge_code_spans(raw, &mut spans);
        let inside: String = raw
            .char_indices()
            .filter(|(i, _)| spans[*i] && !comment[*i])
            .map(|(_, c)| c)
            .collect();
        if !inside.is_empty() {
            out.push(Marked {
                line,
                text: inside,
                structural: false,
            });
        }
    }
    out
}

/// The first word after a list marker (`- X`, `* X`, `1. X`, `1) X`).
fn list_lead(trimmed: &str) -> Option<String> {
    let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
    let rest = if digits > 0 {
        trimmed[digits..].strip_prefix(['.', ')'])?
    } else {
        trimmed.strip_prefix(['-', '*', '+'])?
    };
    if !rest.starts_with([' ', '\t']) {
        return None;
    }
    let w = words(rest);
    w.iter().find_map(|x| x.text().map(str::to_string))
}
