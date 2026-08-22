//! Machine-resolvable file:line citations for the methodology family.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Citation {
    citing: String,
    citing_line: usize,
    link: String,
    target: String,
    line: usize,
    end: Option<usize>,
    index: usize,
}

#[derive(Debug, Deserialize, Serialize)]
struct LedgerEntry {
    target: String,
    line: usize,
    text: String,
}

type Ledger = BTreeMap<String, LedgerEntry>;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .to_path_buf()
}

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
            if let Some((target, target_line, end)) = parse_anchor(link) {
                index += 1;
                out.push(Citation {
                    citing: citing.clone(),
                    citing_line: line_no + 1,
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

fn target_path(root: &Path, citation: &Citation) -> PathBuf {
    root.join(&citation.citing)
        .parent()
        .expect("citing file parent")
        .join(&citation.target)
}

fn lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read target {}: {e}", path.display()))
        .lines()
        .map(str::to_string)
        .collect()
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

fn key(citation: &Citation) -> String {
    format!("{}:{}", citation.citing, citation.index)
}

fn current_ledger(root: &Path, citations: &[Citation]) -> Ledger {
    citations
        .iter()
        .filter_map(|citation| {
            let path = target_path(root, citation);
            let target_lines = lines(&path);
            target_lines.get(citation.line - 1).map(|text| {
                (
                    key(citation),
                    LedgerEntry {
                        target: citation.target.clone(),
                        line: citation.line,
                        text: text.trim().to_string(),
                    },
                )
            })
        })
        .collect()
}

fn json(ledger: &Ledger) -> String {
    format!(
        "{}\n",
        serde_json::to_string_pretty(ledger).expect("serialize citation ledger")
    )
}

fn unique_line(target: &[String], text: &str) -> Option<usize> {
    let matches: Vec<_> = target
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim() == text)
        .map(|(i, _)| i + 1)
        .collect();
    // then, not then_some: then_some evaluates matches[0] eagerly and
    // panics on an empty match set — the vanished path needs the None
    (matches.len() == 1).then(|| matches[0])
}

fn ledger_errors(root: &Path, citations: &[Citation], ledger: &Ledger) -> Vec<String> {
    let mut errors = Vec::new();
    let expected: BTreeSet<_> = citations.iter().map(key).collect();
    let actual: BTreeSet<_> = ledger.keys().cloned().collect();
    for missing in expected.difference(&actual) {
        errors.push(format!("{missing}: missing ledger entry"));
    }
    for extra in actual.difference(&expected) {
        errors.push(format!("{extra}: extra ledger entry"));
    }
    for citation in citations {
        let Some(entry) = ledger.get(&key(citation)) else {
            continue;
        };
        let path = target_path(root, citation);
        if entry.target != citation.target {
            errors.push(format!(
                "{}: target changed: {} -> {}",
                key(citation),
                entry.target,
                citation.target
            ));
            continue;
        }
        let target_lines = lines(&path);
        let current = target_lines.get(citation.line - 1).map(|s| s.trim());
        if current == Some(entry.text.as_str()) && citation.line == entry.line {
            continue;
        }
        match unique_line(&target_lines, &entry.text) {
            Some(line) if line != entry.line => {
                errors.push(format!("{}: moved: now at L{line}", key(citation)));
            }
            _ => errors.push(format!(
                "{}: vanished: semantic change needs a human",
                key(citation)
            )),
        }
    }
    errors
}

fn assert_ledger(root: &Path, citations: &[Citation], current: &Ledger) {
    let path = root.join("contracts/docs-citations.json");
    if std::env::var("CE_BLESS").as_deref() == Ok("1") {
        let rendered = json(current);
        let status = match fs::read_to_string(&path) {
            Ok(old) if old.replace("\r\n", "\n") == rendered => "unchanged",
            Ok(_) => "changed",
            Err(_) => "created",
        };
        fs::write(&path, rendered).expect("bless citation ledger");
        println!(
            "blessed {} citations at {} ({status})",
            current.len(),
            path.display()
        );
        return;
    }
    let saved = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "missing citation ledger {} ({e}); CE_BLESS=1 to create",
            path.display()
        )
    });
    let ledger: Ledger = serde_json::from_str(&saved).expect("parse citation ledger");
    let errors = ledger_errors(root, citations, &ledger);
    assert!(
        errors.is_empty(),
        "docs citation ledger errors:\n{}",
        errors.join("\n")
    );
    assert_eq!(
        saved.replace("\r\n", "\n"),
        json(current),
        "docs citation ledger drifted — CE_BLESS=1 to regenerate"
    );
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
