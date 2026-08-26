use crate::common::repo_root;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
struct Row {
    n: usize,
    title: String,
    file: String,
}

fn parse_row(line: &str) -> Row {
    let cells: Vec<_> = line.split('|').collect();
    assert!(
        cells.len() == 5,
        "docs_nav: malformed Contents row: {line:?}"
    );
    let n = cells[1]
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("docs_nav: malformed Contents row number: {line:?}"));
    let link = cells[2].trim();
    let (title, target) = link
        .strip_prefix('[')
        .and_then(|rest| rest.split_once("]("))
        .and_then(|(title, target)| target.strip_suffix(')').map(|t| (title, t)))
        .unwrap_or_else(|| panic!("docs_nav: malformed Contents row link: {line:?}"));
    let file = target
        .strip_prefix("methodology/")
        .filter(|f| !f.is_empty() && !f.contains('/'))
        .unwrap_or_else(|| panic!("docs_nav: malformed Contents row target: {line:?}"));
    assert!(
        !title.is_empty(),
        "docs_nav: malformed Contents row title: {line:?}"
    );
    Row {
        n,
        title: title.to_string(),
        file: file.to_string(),
    }
}

fn contents(root: &Path) -> Vec<Row> {
    let text = fs::read_to_string(root.join("docs/reference/methodology.md"))
        .expect("docs_nav: read methodology index");
    let mut section = false;
    let mut table = false;
    let mut rows = Vec::new();
    for line in text.lines() {
        if !table {
            section |= line == "## Contents";
            table = section && line == "| # | Section | What it judges |";
            continue;
        }
        if line.trim().is_empty() {
            break;
        }
        if line.starts_with("|---") {
            continue;
        }
        assert!(
            line.starts_with('|'),
            "docs_nav: malformed Contents row: {line:?}"
        );
        rows.push(parse_row(line));
    }
    assert!(section, "docs_nav: Contents heading missing");
    assert!(table, "docs_nav: Contents table header missing");
    assert!(!rows.is_empty(), "docs_nav: Contents table has no rows");
    rows
}

fn fmt_nav(prev: Option<&Row>, next: Option<&Row>) -> String {
    let mut parts = vec!["[index](../methodology.md)".to_string()];
    if let Some(row) = prev {
        parts.push(format!("[← {:02} {}]({})", row.n, row.title, row.file));
    }
    if let Some(row) = next {
        parts.push(format!("[→ {:02} {}]({})", row.n, row.title, row.file));
    }
    parts.join(" · ")
}

fn directory_files(dir: &Path) -> BTreeSet<String> {
    fs::read_dir(dir)
        .expect("docs_nav: read methodology booklet directory")
        .map(|entry| {
            let entry = entry.expect("docs_nav: read methodology directory entry");
            assert!(
                entry
                    .file_type()
                    .expect("docs_nav: booklet file type")
                    .is_file(),
                "docs_nav: directory entry is not a file: {}",
                entry.path().display()
            );
            entry.file_name().to_string_lossy().into_owned()
        })
        .collect()
}

fn check_booklet(dir: &Path, row: &Row, prev: Option<&Row>, next: Option<&Row>) {
    let path = dir.join(&row.file);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("docs_nav: {}: cannot read booklet: {e}", row.file));
    let lines: Vec<_> = text.lines().collect();
    let expected = format!("# {}", row.title);
    let actual = lines.first().copied().unwrap_or("<missing>");
    assert_eq!(
        actual, expected,
        "docs_nav: {} line 1 expected {expected:?}, actual {actual:?}",
        row.file
    );
    assert_eq!(
        lines.get(1).copied().unwrap_or("<missing>"),
        "",
        "docs_nav: {} line 2 expected empty, actual {:?}",
        row.file,
        lines.get(1).copied().unwrap_or("<missing>")
    );
    let expected_nav = fmt_nav(prev, next);
    assert_eq!(
        lines.get(2).copied().unwrap_or("<missing>"),
        expected_nav,
        "docs_nav: {} line 3 expected {expected_nav:?}, actual {:?}",
        row.file,
        lines.get(2).copied().unwrap_or("<missing>")
    );
    assert_eq!(
        lines.get(3).copied().unwrap_or("<missing>"),
        "",
        "docs_nav: {} line 4 expected empty, actual {:?}",
        row.file,
        lines.get(3).copied().unwrap_or("<missing>")
    );
}

#[test]
fn methodology_booklets_have_derived_navigation() {
    let root = repo_root();
    let rows = contents(&root);
    let dir = root.join("docs/reference/methodology");
    let expected: BTreeSet<_> = rows.iter().map(|row| row.file.clone()).collect();
    let actual = directory_files(&dir);
    let missing: Vec<_> = expected.difference(&actual).cloned().collect();
    let extra: Vec<_> = actual.difference(&expected).cloned().collect();
    assert!(
        missing.is_empty(),
        "docs_nav: table files missing from directory: {missing:?}"
    );
    assert!(
        extra.is_empty(),
        "docs_nav: directory files absent from table: {extra:?}"
    );
    for (i, row) in rows.iter().enumerate() {
        check_booklet(
            &dir,
            row,
            i.checked_sub(1).map(|j| &rows[j]),
            rows.get(i + 1),
        );
    }
}
