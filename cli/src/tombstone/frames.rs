//! Reading a surface through the vocabulary (vocab.rs): the word cut
//! every reader shares, the windows a name is spelled in, the frame
//! candidates a label yields, and the retrospective-mark test for
//! prose. Nothing here knows what was erased — the hub (mod.rs) joins
//! these readings with names.rs.

use super::vocab::{
    CLOSE, EN_PREFIX, EN_SUFFIX, JOIN_MAX, MARKS_EN, MARKS_ZH, OPEN, ZH_PREFIX, ZH_SUFFIX, entries,
    has,
};

/// One token of a surface: a bracket, an ASCII word (lower-cased; cut
/// at `_` `-` `$`, at any non-alphanumeric, and at a camel rise), or
/// a whole non-ASCII run (a Chinese phrase is one token — the
/// segmentation limit the spec states rather than solves).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Word {
    Open,
    Close,
    Ascii(String),
    Wide(String),
}

impl Word {
    fn ascii(&self) -> Option<&str> {
        match self {
            Word::Ascii(a) => Some(a),
            _ => None,
        }
    }

    /// The spelling of a word that can be part of a name.
    pub fn text(&self) -> Option<&str> {
        match self {
            Word::Ascii(s) | Word::Wide(s) => Some(s),
            _ => None,
        }
    }
}

/// The cut in progress: the ASCII word and the wide run being built,
/// and the raw last ASCII char (the camel test needs its case).
#[derive(Default)]
struct Cut {
    ascii: String,
    wide: String,
    last: Option<char>,
}

/// Push the word a buffer holds, if any, and empty it.
fn take(buf: &mut String, out: &mut Vec<Word>, make: fn(String) -> Word) {
    if !buf.is_empty() {
        out.push(make(std::mem::take(buf)));
    }
}

impl Cut {
    fn ascii(&mut self, raw: char, out: &mut Vec<Word>) {
        take(&mut self.wide, out, Word::Wide);
        let rises = raw.is_ascii_uppercase()
            && self
                .last
                .is_some_and(|p| p.is_ascii_lowercase() || p.is_ascii_digit());
        if rises {
            self.flush_ascii(out);
        }
        self.ascii.push(raw.to_ascii_lowercase());
        self.last = Some(raw);
    }

    fn wide(&mut self, c: char, out: &mut Vec<Word>) {
        self.flush_ascii(out);
        self.wide.push(c);
    }

    fn flush_ascii(&mut self, out: &mut Vec<Word>) {
        take(&mut self.ascii, out, Word::Ascii);
        self.last = None;
    }

    fn flush(&mut self, out: &mut Vec<Word>) {
        self.flush_ascii(out);
        take(&mut self.wide, out, Word::Wide);
    }
}

/// The word cut of a surface, left to right.
pub fn words(s: &str) -> Vec<Word> {
    let mut out = Vec::new();
    let mut cut = Cut::default();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            cut.ascii(c, &mut out);
        } else if c.is_alphanumeric() {
            cut.wide(c, &mut out);
        } else {
            cut.flush(&mut out);
            if OPEN.contains(&c) {
                out.push(Word::Open);
            } else if CLOSE.contains(&c) {
                out.push(Word::Close);
            }
        }
    }
    cut.flush(&mut out);
    out
}

/// One spelling a surface offers: the words `w[at..at + len]`,
/// `_`-joined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub at: usize,
    pub len: usize,
    pub text: String,
}

/// None when the window runs off the end or over a bracket.
fn span(w: &[Word], at: usize, len: usize) -> Option<Span> {
    let parts: Option<Vec<&str>> = (at..at + len).map(|j| w.get(j)?.text()).collect();
    parts.map(|p| Span {
        at,
        len,
        text: p.join("_"),
    })
}

/// Every window of 1..=JOIN_MAX adjacent words; a bracket or the end
/// cuts a window short.
pub fn windows(w: &[Word]) -> Vec<Span> {
    (0..w.len())
        .flat_map(|at| (1..=JOIN_MAX).filter_map(move |len| span(w, at, len)))
        .collect()
}

/// A name an absence frame binds, and whether the frame is
/// parenthesized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub span: Span,
    pub bracketed: bool,
}

/// A word that belongs to a frame, never to a name.
pub fn frame_word(w: &str) -> bool {
    has(EN_PREFIX, w) || has(EN_SUFFIX, w) || has(ZH_PREFIX, w) || has(ZH_SUFFIX, w)
}

/// Where the X slot of a prefix frame at `j` starts: the next word,
/// or the one after for `no more`; None when `w[j]` opens no frame.
/// A Chinese prefix counts here only as a word of its own (`无 cache`,
/// `无cache`): inside one run (`无东坡肉`) it is wide_frames' job.
fn slot_after(w: &[Word], j: usize) -> Option<usize> {
    let opens = match &w[j] {
        Word::Ascii(a) => has(EN_PREFIX, a),
        Word::Wide(r) => has(ZH_PREFIX, r),
        _ => false,
    };
    if !opens {
        return None;
    }
    let more = w[j].ascii() == Some("no") && w.get(j + 1).and_then(Word::ascii) == Some("more");
    Some(if more { j + 2 } else { j + 1 })
}

fn closes(w: &Word) -> bool {
    w.ascii().is_some_and(|a| has(EN_SUFFIX, a))
}

/// Every name a label's absence frames bind: `(no Dongpo Pork)`
/// yields `dongpo` and `dongpo_pork` (bracketed), `cook_without_dongpo`
/// yields `dongpo` (bare), `番茄炒蛋（无东坡肉）` yields `东坡肉`
/// (bracketed). A prefix binds the 1..=JOIN_MAX words after it (`no
/// more` counts as one prefix), a suffix the words before it, and a
/// Chinese prefix or suffix form binds the rest of its own run. The
/// candidates are spellings; the hub keys them (names::key) and asks
/// the erased set — a candidate that names nothing erased is nothing.
pub fn label_candidates(w: &[Word]) -> Vec<Candidate> {
    let mut out = Vec::new();
    for j in 0..w.len() {
        let bracketed = inside_brackets(w, j);
        let mut push = |s: Option<Span>| out.extend(s.map(|span| Candidate { span, bracketed }));
        if let Some(from) = slot_after(w, j) {
            (1..=JOIN_MAX).for_each(|len| push(span(w, from, len)));
        }
        if closes(&w[j]) {
            (1..=JOIN_MAX).for_each(|len| push(j.checked_sub(len).and_then(|at| span(w, at, len))));
        }
        if let Word::Wide(r) = &w[j] {
            let own = |text| {
                Some(Span {
                    at: j,
                    len: 1,
                    text,
                })
            };
            wide_frames(r).into_iter().for_each(|text| push(own(text)));
        }
    }
    out
}

/// Inside a bracket pair: the nearest bracket before `i` opens and
/// the nearest after `i` closes.
fn inside_brackets(w: &[Word], i: usize) -> bool {
    let bracket = |x: &&Word| matches!(x, Word::Open | Word::Close);
    let before = w[..i].iter().rev().find(bracket);
    let after = w[i + 1..].iter().find(bracket);
    matches!(before, Some(Word::Open)) && matches!(after, Some(Word::Close))
}

/// A wide run as a frame: a Chinese prefix form opens it or a suffix
/// form closes it, and the rest of the run is the name.
fn wide_frames(r: &str) -> Vec<String> {
    let heads = entries(ZH_PREFIX).filter_map(|p| r.strip_prefix(p));
    let tails = entries(ZH_SUFFIX).filter_map(|s| r.strip_suffix(s));
    heads
        .chain(tails)
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .collect()
}

/// The sentences of a prose text: cut after `.`, `!`, `?`, `;` when
/// whitespace or the end follows (so `ce.toml` and `a.rs` stay whole)
/// and after any full-width `。！？；`. The prose conjunction is read
/// per sentence — the fourth self-replay round bound a name mentioned
/// 3,000 characters away from the mark on one 5,000-character line.
pub fn sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut it = text.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        let end = i + c.len_utf8();
        let next = it.peek().map(|(_, n)| *n);
        let cut = matches!(c, '。' | '！' | '？' | '；')
            || (matches!(c, '.' | '!' | '?' | ';') && next.is_none_or(char::is_whitespace));
        if cut {
            out.push(&text[start..end]);
            start = end;
        }
    }
    out.push(&text[start..]);
    out.into_iter()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

/// Whether a prose segment carries a retrospective mark.
pub fn has_mark(text: &str) -> bool {
    let lower = text.to_lowercase();
    entries(MARKS_EN).any(|m| phrase_in(&lower, m)) || entries(MARKS_ZH).any(|m| text.contains(m))
}

/// `needle` (ASCII) in `hay` at word boundaries — `previously` must
/// not match inside a `previously_seen` identifier. Each occurrence
/// is tried; the needle starts with an ASCII byte, so `start + 1` is
/// always a char boundary.
fn phrase_in(hay: &str, needle: &str) -> bool {
    let mut from = 0;
    while let Some(pos) = hay[from..].find(needle) {
        let (start, end) = (from + pos, from + pos + needle.len());
        let clean = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric() && c != '_');
        if clean(hay[..start].chars().next_back()) && clean(hay[end..].chars().next()) {
            return true;
        }
        from = start + 1;
    }
    false
}

#[cfg(test)]
#[path = "../../tests/unit/tombstone/frames.rs"]
mod tests;
