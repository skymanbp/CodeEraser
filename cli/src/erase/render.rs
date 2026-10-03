//! The plan's unified diff (erase.md §two-phases): every hunk carries
//! its verdict provenance. The diff re-reads each target and re-verifies
//! the plan's content hash on the way — a rendering that showed bytes the
//! plan did not hash would be a second source of truth. The console
//! lines and the plan document are the core's (erase/document.rs); the
//! core places one diff per file. `report_json` is the GUI preview's
//! document until that face reads the core's (it holds no core path).

use crate::erase::model::{Plan, Row, family_command};
use anyhow::{Result, ensure};
use std::path::Path;

/// The hunks' context lines (CE.Erase.Document `diffContext`).
const CONTEXT: usize = 3;

/// The plan document. `families` (0.3.0) names the family command
/// behind every out-of-class kind — the same table the console
/// sentence reads, so a GUI chip and a console line never disagree.
pub fn report_json(p: &Plan) -> serde_json::Value {
    let families: serde_json::Map<String, serde_json::Value> = p
        .counts
        .out_of_class
        .keys()
        .filter_map(|k| family_command(k).map(|c| ((*k).to_string(), c.into())))
        .collect();
    serde_json::json!({
        "schema": crate::erase::model::SCHEMA_ID,
        "rows": p.rows,
        "counts": p.counts,
        "families": families,
    })
}

/// Unified diff over every eraseable row, grouped per file, hunks in
/// plan order with cumulative new-side offsets. Provenance rides as
/// a `##` trailer on each hunk header — verdict family, evidence,
/// and the plan's file hash (fnv1a64).
pub fn diff(root: &Path, p: &Plan) -> Result<String> {
    Ok(file_diffs(root, p)?
        .into_iter()
        .map(|(_, d)| d + "\n")
        .collect())
}

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

fn file_diff(out: &mut String, text: &str, rows: &[&Row]) {
    let lines: Vec<&str> = text.lines().collect();
    let path = &rows[0].path;
    if rows.iter().any(|r| r.span.is_none()) {
        let r = rows.iter().find(|r| r.span.is_none()).expect("checked");
        out.push_str(&format!("--- a/{path}\n+++ /dev/null\n"));
        out.push_str(&format!(
            "@@ -1,{} +0,0 @@ ## {} {} fnv1a64:{:016x}\n",
            lines.len(),
            r.class,
            r.provenance,
            r.hash
        ));
        for l in &lines {
            out.push_str(&format!("-{l}\n"));
        }
        return;
    }
    out.push_str(&format!("--- a/{path}\n+++ b/{path}\n"));
    let mut removed_so_far = 0usize;
    for r in rows {
        let (s, e) = r.span.expect("span rows only");
        let (s, e) = (s as usize, (e as usize).min(lines.len()));
        let head = s.saturating_sub(CONTEXT + 1) + 1; // first shown line
        let tail = (e + CONTEXT).min(lines.len());
        let old_len = tail - head + 1;
        let cut = e - s + 1;
        out.push_str(&format!(
            "@@ -{head},{old_len} +{},{} @@ ## {} {} fnv1a64:{:016x}\n",
            head - removed_so_far,
            old_len - cut,
            r.class,
            r.provenance,
            r.hash
        ));
        for (i, l) in lines.iter().enumerate().take(tail).skip(head - 1) {
            let mark = if (s..=e).contains(&(i + 1)) { '-' } else { ' ' };
            out.push_str(&format!("{mark}{l}\n"));
        }
        removed_so_far += cut;
    }
}

#[cfg(test)]
#[path = "../../tests/unit/erase/render.rs"]
mod tests;
