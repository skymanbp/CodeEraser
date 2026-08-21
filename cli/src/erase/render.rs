//! The plan's two faces (erase.md §two-phases): a unified diff whose
//! every hunk carries its verdict provenance, and the machine JSON
//! rows. The diff re-reads each target and re-verifies the plan's
//! content hash on the way — a rendering that showed bytes the plan
//! did not hash would be a second source of truth.

use crate::erase::model::{Plan, Row};
use anyhow::{Result, ensure};
use std::path::Path;

const CONTEXT: usize = 3;

pub fn report_json(p: &Plan) -> serde_json::Value {
    serde_json::json!({
        "schema": crate::erase::model::SCHEMA_ID,
        "rows": p.rows,
        "counts": p.counts,
    })
}

/// The whole console face: hunks for eraseable rows, named advisory
/// rows, the aggregate out-of-class tail, one summary line.
pub fn print(root: &Path, p: &Plan, as_json: bool) -> Result<()> {
    if as_json {
        println!("{}", report_json(p));
        return Ok(());
    }
    print!("{}", diff(root, p)?);
    for r in p.rows.iter().filter(|r| !r.eraseable) {
        println!(
            "{}",
            crate::i18n::line(
                "advisory {} {}{}: {} ({})",
                "仅建议 {} {}{}：{}（{}）",
                &[
                    &r.class,
                    &r.path,
                    &span_str(r),
                    &r.reason,
                    &r.provenance.as_str()
                ],
            )
        );
    }
    for (k, n) in &p.counts.out_of_class {
        println!(
            "{}",
            crate::i18n::line(
                "advisory {}: {} finding(s) — no deterministic-safe erase; see the family command",
                "仅建议 {}：{} 条——无确定性安全擦除；见对应家族命令",
                &[k, n],
            )
        );
    }
    println!(
        "{}",
        crate::i18n::line(
            "erase plan: {} eraseable, {} advisory (dry-run; --apply to act)",
            "擦除计划：可擦 {}，仅建议 {}（演练；--apply 才动手）",
            &[&p.counts.eraseable, &p.counts.advisory],
        )
    );
    Ok(())
}

/// Unified diff over every eraseable row, grouped per file, hunks in
/// plan order with cumulative new-side offsets. Provenance rides as
/// a `##` trailer on each hunk header — verdict family, evidence,
/// and the plan's file hash (fnv1a64).
pub fn diff(root: &Path, p: &Plan) -> Result<String> {
    let mut cache = crate::erase::gather::TextCache::new(root);
    let mut out = String::new();
    let mut rows = p.eraseable().peekable();
    while let Some(first) = rows.next() {
        let mut file_rows = vec![first];
        while rows.peek().is_some_and(|r| r.path == first.path) {
            file_rows.push(rows.next().expect("peeked"));
        }
        let text = cache.text(&first.path)?.to_string();
        ensure!(
            crate::dedup::tokens::fnv1a(text.as_bytes()) == first.hash,
            "{}: content changed since planning — re-run ce erase",
            first.path
        );
        file_diff(&mut out, &text, &file_rows);
    }
    Ok(out)
}

fn span_str(r: &Row) -> String {
    match r.span {
        Some((s, e)) => format!(":{s}-{e}"),
        None => String::new(),
    }
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
