//! Machine-resolvable file:line citations for the methodology family.
//! This half harvests citations and checks them structurally; the
//! ledger half and the shapes both use live in docs_citations_parts.

mod common;
use common::repo_root;

mod docs_citations_parts;

use docs_citations_parts::{
    Citation, assert_ledger, current_ledger, label_lines, lines, target_path,
};
use std::fs;
use std::path::{Path, PathBuf};

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("methodology path under repo root")
        .to_string_lossy()
        .replace('\\', "/")
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![root.join("docs/reference/methodology.md")];
    let dir = root.join("docs/reference/methodology");
    files.extend(
        fs::read_dir(dir)
            .expect("methodology directory")
            .map(|entry| entry.expect("methodology entry").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "md")),
    );
    files.sort();
    files
}

fn parse_anchor(link: &str) -> Option<(&str, usize, Option<usize>)> {
    let hash = link.find("#L")?;
    let target = &link[..hash];
    let numbers = &link[hash + 2..];
    let (start, end) = numbers
        .split_once("-L")
        .map_or((numbers, None), |(a, b)| (a, Some(b)));
    if start.is_empty() || !start.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let line = start.parse().ok()?;
    let end = end.map(str::parse).transpose().ok()?;
    Some((target, line, end))
}

fn citations_in_file(root: &Path, path: &Path) -> Vec<Citation> {
    let text = fs::read_to_string(path).expect("read methodology markdown");
    let citing = display_path(root, path);
    let mut out = Vec::new();
    let mut index = 0;
    for (line_no, line) in text.lines().enumerate() {
        let mut rest = line;
        while let Some(open) = rest.find("](") {
            let after = &rest[open + 2..];
            let Some(close) = after.find(')') else { break };
            let link = &after[..close];
            // the label is the bracketed half immediately before
            // "](": scan back to its opening bracket
            let head = &rest[..open];
            let label = head
                .rfind('[')
                .map(|b| head[b + 1..].to_string())
                .unwrap_or_default();
            if let Some((target, target_line, end)) = parse_anchor(link) {
                index += 1;
                out.push(Citation {
                    citing: citing.clone(),
                    citing_line: line_no + 1,
                    label,
                    link: link.to_string(),
                    target: target.to_string(),
                    line: target_line,
                    end,
                    index,
                });
            }
            rest = &after[close + 1..];
        }
    }
    out
}

fn all_citations(root: &Path) -> Vec<Citation> {
    markdown_files(root)
        .iter()
        .flat_map(|path| citations_in_file(root, path))
        .collect()
}

/// The label must agree with the anchor it links to (v2.14). The
/// ledger pins the anchor's TEXT, so a stale label sailed through
/// every bless — 28 of them had, across the corpus, each one telling
/// a reader a line number the link does not go to. A range label may
/// anchor at its start alone (the house convention), but any number
/// it does state must match.
fn label_errors(citation: &Citation, prefix: &str, errors: &mut Vec<String>) {
    let Some((label_start, label_end)) = label_lines(&citation.label) else {
        return;
    };
    if label_start != citation.line {
        errors.push(format!(
            "{prefix} -> label says L{label_start}, anchor goes to L{}",
            citation.line
        ));
    }
    if let (Some(le), Some(ae)) = (label_end, citation.end)
        && le != ae
    {
        errors.push(format!(
            "{prefix} -> label range ends L{le}, anchor ends L{ae}"
        ));
    }
}

fn structural_errors(root: &Path, citations: &[Citation]) -> Vec<String> {
    let mut errors = Vec::new();
    for citation in citations {
        let path = target_path(root, citation);
        let prefix = format!(
            "{}:{}: {}",
            citation.citing, citation.citing_line, citation.link
        );
        if !path.is_file() {
            errors.push(format!("{prefix} -> target missing"));
            continue;
        }
        let count = lines(&path).len();
        if citation.line < 1 || citation.line > count {
            errors.push(format!(
                "{prefix} -> line L{} past EOF (target has {} lines)",
                citation.line, count
            ));
        }
        label_errors(citation, &prefix, &mut errors);
        if let Some(end) = citation.end {
            if end < citation.line {
                errors.push(format!(
                    "{prefix} -> range end L{end} before start L{}",
                    citation.line
                ));
            } else if end > count {
                errors.push(format!(
                    "{prefix} -> range end L{end} past EOF (target has {} lines)",
                    count
                ));
            }
        }
    }
    errors
}

#[test]
fn methodology_citations_are_resolvable() {
    let root = repo_root();
    let citations = all_citations(&root);
    // the harvest is real (the i18n gate's own guard): a broken
    // parser must not pass vacuously over an empty citation set —
    // the family carries ~1200 anchors, so 500 is a safe floor
    assert!(
        citations.len() > 500,
        "only {} citations harvested — the parser is broken",
        citations.len()
    );
    let structural = structural_errors(&root, &citations);
    assert!(
        structural.is_empty(),
        "docs citation structural errors:\n{}",
        structural.join("\n")
    );
    let current = current_ledger(&root, &citations);
    assert_ledger(&root, &citations, &current);
}
