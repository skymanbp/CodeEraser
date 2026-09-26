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
/// counterexample). An indented code block (CommonMark §4.4: four
/// columns of indent where a paragraph is not open — at the document
/// start, after a blank line or a fence) opens four columns past the
/// content column of the innermost open list item (§5.2: an item's
/// content starts after its marker and the one to four spaces
/// following it, so its continuation paragraph, indented to that
/// column, is prose and a line four columns deeper is code — plan
/// v2.30 step 5b; the block used to open outside a list context
/// only); it runs while lines stay at that indent or blank. A
/// non-blank line indented short of an item's content column after a
/// blank line closes the item (a lazy continuation of an open
/// paragraph does not). Neither block opens inside an HTML comment.
/// The conservative side is deliberate: a block this walk does not
/// recognise keeps the reading it had (its link-shaped content stays
/// a site, its `#` lines stay headings), never the reverse.
pub(super) struct Blocks {
    /// The open fence's marker and run length.
    fence: Option<(char, usize)>,
    /// The open indented block's column: the code column it opened at.
    indented: Option<usize>,
    /// No paragraph is open: nothing yet, or the previous line was
    /// blank or a fence.
    open: bool,
    /// Content columns of the open list items, innermost last.
    lists: Vec<usize>,
}

impl Default for Blocks {
    fn default() -> Self {
        Blocks {
            fence: None,
            indented: None,
            open: true,
            lists: Vec::new(),
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
        if let Some(col) = self.indented {
            if blank || indent >= col {
                return true;
            }
            self.indented = None;
        }
        if !in_comment && let Some((mark, len)) = fence_marker(trimmed) {
            self.toggle_fence(mark, len);
            return true;
        }
        if self.fence.is_some() {
            return true;
        }
        if blank {
            self.open = true;
            return false;
        }
        let code = self.code_column(indent, trimmed);
        if !in_comment && self.open && indent >= code {
            self.indented = Some(code);
            return true;
        }
        if let Some(width) = list_item(trimmed) {
            self.lists.push(indent + width);
        }
        self.open = false;
        false
    }

    /// A fence marker line: it closes the open fence when it matches the
    /// opening run (same character, at least as long), is text inside a
    /// fence of the other character, and opens a fence otherwise; either
    /// way no paragraph is open after it.
    fn toggle_fence(&mut self, mark: char, len: usize) {
        match self.fence {
            Some((open, run)) if open == mark && len >= run => self.fence = None,
            Some(_) => {}
            None => self.fence = Some((mark, len)),
        }
        self.open = true;
    }

    /// The column where indented code starts for a non-blank line at
    /// `indent`, after closing the list items the line leaves: every
    /// item whose content column it falls short of, when no paragraph
    /// is open or the line is itself a list marker (a marker at an
    /// outer level interrupts the paragraph; any other short line is a
    /// lazy continuation and closes nothing).
    fn code_column(&mut self, indent: usize, trimmed: &str) -> usize {
        if self.open || list_item(trimmed).is_some() {
            while self.lists.last().is_some_and(|&col| indent < col) {
                self.lists.pop();
            }
        }
        self.lists.last().map_or(0, |col| *col) + 4
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
/// followed by whitespace or the end of the line: the width of the
/// item's marker plus the spaces its content sits after (CommonMark
/// §5.2: one to four; five or more, or the end of the line, read as
/// one, the rest being the item's own indented code or nothing).
/// pub(crate): the heading reader asks whether a line is a list item
/// (ladder/md_head.rs).
pub(crate) fn list_item(trimmed: &str) -> Option<usize> {
    let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
    let marker = if digits > 0 {
        trimmed[digits..]
            .starts_with(['.', ')'])
            .then_some(digits + 1)
    } else {
        trimmed.starts_with(['-', '*', '+']).then_some(1)
    }?;
    let rest = &trimmed[marker..];
    if rest.is_empty() {
        return Some(marker + 1);
    }
    if !rest.starts_with([' ', '\t']) {
        return None;
    }
    let gap = columns(rest);
    Some(marker + if gap > 4 { 1 } else { gap })
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
