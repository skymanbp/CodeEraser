//! The plan's unified diff (erase.md §two-phases): every hunk carries
//! its verdict provenance. The diff re-reads each target and re-verifies
//! the plan's content hash on the way — a rendering that showed bytes the
//! plan did not hash would be a second source of truth. The console
//! lines and the plan document are the core's (erase/document.rs); the
//! core places one diff per file, and the GUI's preview reads the same
//! texts joined (`document::Diffs::unified`).

use crate::erase::model::{Plan, Row};
use anyhow::{Result, ensure};
use std::path::Path;

/// The hunks' context lines (CE.Erase.Document `diffContext`).
const CONTEXT: usize = 3;

/// The diff one file at a time: the place of the file's first eraseable
/// row in the plan, and its hunks (no final newline — the console prints
/// each as one line).
pub fn file_diffs(root: &Path, p: &Plan) -> Result<Vec<(usize, String)>> {
    let mut cache = crate::erase::gather::TextCache::new(root);
    let mut out = Vec::new();
    let mut rows = p
        .rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.eraseable)
        .peekable();
    while let Some((at, first)) = rows.next() {
        let mut file_rows = vec![first];
        while let Some((_, r)) = rows.next_if(|(_, r)| r.path == first.path) {
            file_rows.push(r);
        }
        let text = cache.text(&first.path)?.to_string();
        ensure!(
            crate::dedup::tokens::fnv1a(text.as_bytes()) == first.hash,
            "{}: content changed since planning — re-run ce erase",
            first.path
        );
        let mut hunks = String::new();
        file_diff(&mut hunks, &text, &file_rows);
        hunks.pop();
        out.push((at, hunks));
    }
    Ok(out)
}

/// One file's hunks. Every eraseable row is a span since 9.1.0 (no
/// whole-file row is eraseable: the core's reachable_by_name).
fn file_diff(out: &mut String, text: &str, rows: &[&Row]) {
    let lines: Vec<&str> = text.lines().collect();
    let path = &rows[0].path;
    out.push_str(&format!("--- a/{path}\n+++ b/{path}\n"));
    let mut removed_so_far = 0usize;
    for group in hunks(rows, lines.len()) {
        let (head, tail) = (group.head, group.tail);
        let cut_at = |n: usize| group.spans.iter().any(|&(s, e, _)| (s..=e).contains(&n));
        let old_len = tail - head + 1;
        let cut = (head..=tail).filter(|&n| cut_at(n)).count();
        let tags: Vec<String> = group
            .spans
            .iter()
            .map(|(_, _, r)| format!("## {} {} fnv1a64:{:016x}", r.class, r.provenance, r.hash))
            .collect();
        out.push_str(&format!(
            "@@ -{head},{old_len} +{},{} @@ {}\n",
            head - removed_so_far,
            old_len - cut,
            tags.join(" ")
        ));
        for (i, l) in lines.iter().enumerate().take(tail).skip(head - 1) {
            let mark = if cut_at(i + 1) { '-' } else { ' ' };
            out.push_str(&format!("{mark}{l}\n"));
        }
        removed_so_far += cut;
    }
}

/// One hunk: its first and last shown line and the spans it removes.
struct Hunk<'r> {
    head: usize,
    tail: usize,
    spans: Vec<(usize, usize, &'r Row)>,
}

/// The rows' spans in line order, gathered into hunks whose windows
/// (each span with CONTEXT lines either side) touch or overlap
/// (v2.35 G6, R3-14). One hunk per span printed lines 6–9 twice for
/// spans (5,6) and (9,10) — line 6 `-` in one hunk and context in the
/// next — which no tool applies as a unified diff.
fn hunks<'r>(rows: &[&'r Row], len: usize) -> Vec<Hunk<'r>> {
    let mut spans: Vec<(usize, usize, &Row)> = rows
        .iter()
        .map(|r| {
            let (s, e) = r.span.expect("span rows only");
            (s as usize, (e as usize).min(len), *r)
        })
        .collect();
    spans.sort_by_key(|&(s, e, _)| (s, e));
    let mut out: Vec<Hunk> = Vec::new();
    for (s, e, r) in spans {
        let head = s.saturating_sub(CONTEXT + 1) + 1; // first shown line
        let tail = (e + CONTEXT).min(len);
        match out.last_mut() {
            Some(h) if head <= h.tail + 1 => {
                h.tail = h.tail.max(tail);
                h.spans.push((s, e, r));
            }
            _ => out.push(Hunk {
                head,
                tail,
                spans: vec![(s, e, r)],
            }),
        }
    }
    out
}

#[cfg(test)]
#[path = "../../tests/unit/erase/render.rs"]
mod tests;
