//! The masking half of the Markdown scanner (split from md.rs when
//! indented code joined the block model — plan v2.17 L round step 8,
//! O57): the block state that drops whole lines (fences, indented
//! code) and the byte masks laid over a kept line (HTML comments,
//! inline code spans). One implementation serves the detector, the
//! ladder's heading walk and docdup's segment extractor — a judge
//! seeing text the detector masks would be the drift bug.

/// Block state across lines. A fence opens on three-or-more
/// backticks or tildes and only the SAME marker in a run AT LEAST AS
/// LONG closes it (``` inside a ~~~ block is content, and so is ```
/// inside a ```` block — CommonMark's run rule, the step-8 review's
/// counterexample). An indented code block (CommonMark: four columns
/// of indent where a paragraph is not open — at the document start,
/// after a blank line or a fence) opens only outside a list context,
/// since a list item's continuation paragraph is indented the same
/// way and is prose; it runs while lines stay indented or blank.
/// Neither opens inside an HTML comment. The conservative side is
/// deliberate: a block this walk does not recognise keeps the
/// reading it had (its link-shaped content stays a site, its `#`
/// lines stay headings), never the reverse.
pub(super) struct Blocks {
    /// The open fence's marker and run length.
    fence: Option<(char, usize)>,
    indented: bool,
    /// No paragraph is open: nothing yet, or the previous line was
    /// blank or a fence.
    open: bool,
    list: bool,
}

impl Default for Blocks {
    fn default() -> Self {
        Blocks {
            fence: None,
            indented: false,
            open: true,
            list: false,
        }
    }
}

impl Blocks {
    /// Whether `line` is outside content: a fence marker, a fenced
    /// line, or an indented-code line.
    pub(super) fn skips(&mut self, line: &str, in_comment: bool) -> bool {
        let trimmed = line.trim_start();
        let blank = trimmed.is_empty();
        let indent = columns(line);
        if self.indented {
            if blank || indent >= 4 {
                return true;
            }
            self.indented = false;
        }
        if !in_comment && let Some((mark, len)) = fence_marker(trimmed) {
            match self.fence {
                Some((open, run)) if open == mark && len >= run => self.fence = None,
                Some(_) => {}
                None => self.fence = Some((mark, len)),
            }
            self.open = true;
            return true;
        }
        if self.fence.is_some() {
            return true;
        }
        if !in_comment && indent >= 4 && self.open && !self.list {
            self.indented = true;
            return true;
        }
        if !blank {
            self.list = list_item(trimmed) || (indent > 0 && self.list);
        }
        self.open = blank;
        false
    }
}

/// Leading indent in columns: a tab advances to the next multiple of
/// four (CommonMark tab stops).
fn columns(line: &str) -> usize {
    let mut col = 0;
    for c in line.chars() {
        match c {
            ' ' => col += 1,
            '\t' => col += 4 - col % 4,
            _ => break,
        }
    }
    col
}

/// A bullet (`-`, `*`, `+`) or ordered (`1.`, `1)`) list marker
/// followed by whitespace or the end of the line.
fn list_item(trimmed: &str) -> bool {
    let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
    let marker = if digits > 0 {
        trimmed[digits..]
            .starts_with(['.', ')'])
            .then_some(digits + 1)
    } else {
        trimmed.starts_with(['-', '*', '+']).then_some(1)
    };
    marker.is_some_and(|n| trimmed.len() == n || trimmed[n..].starts_with([' ', '\t']))
}

/// Three-or-more backticks or tildes open/close a fence: the marker
/// and its run length (a closer must match both, module doc).
fn fence_marker(trimmed: &str) -> Option<(char, usize)> {
    ['`', '~']
        .into_iter()
        .map(|mark| (mark, trimmed.chars().take_while(|&c| c == mark).count()))
        .find(|&(_, run)| run >= 3)
}

/// Byte mask of `<!-- … -->` spans, stateful across lines.
pub(super) fn comment_mask(line: &str, in_comment: &mut bool) -> Vec<bool> {
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
/// pub(crate): the ladder's anchor reader masks spans too (md_slug.rs).
pub(crate) fn merge_code_spans(line: &str, mask: &mut [bool]) {
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
