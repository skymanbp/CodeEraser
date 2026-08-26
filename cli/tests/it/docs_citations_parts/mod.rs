//! The citation gate's LEDGER half, plus the shapes both halves
//! share. Split out of docs_citations.rs when the v2.14 label/anchor
//! invariant pushed that file past the 300-line dogfood wall — the
//! same wall common/mod.rs and graph/deadcode/flags.rs were split at.
//!
//! Direction is deliberate: the leaf owns the shared shapes and the
//! parent references DOWNWARD, so there is no `use super::` web to
//! become an import cycle by the graph family's own axis-6 measure
//! (the house convention erase/model.rs states).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Citation {
    pub citing: String,
    pub citing_line: usize,
    /// The human-readable half — `Cost.hs:42-55` in
    /// `[Cost.hs:42-55](…#L42)`. The ledger pins anchor TEXT, so a
    /// label that drifts away from its own anchor was invisible to
    /// this gate until the v2.14 invariant; a reader trusts the
    /// label, and a bless would have accepted the lie.
    pub label: String,
    pub link: String,
    pub target: String,
    pub line: usize,
    pub end: Option<usize>,
    pub index: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LedgerEntry {
    pub target: String,
    pub line: usize,
    pub text: String,
}

pub type Ledger = BTreeMap<String, LedgerEntry>;

/// The trailing `:START` / `:START-END` a citation label carries, or
/// None when the label names no line (a bare file reference).
pub fn label_lines(label: &str) -> Option<(usize, Option<usize>)> {
    let (_, tail) = label.rsplit_once(':')?;
    let (start, end) = tail
        .split_once('-')
        .map_or((tail, None), |(a, b)| (a, Some(b)));
    let start: usize = start.parse().ok()?;
    let end = match end {
        Some(e) => Some(e.parse().ok()?),
        None => None,
    };
    Some((start, end))
}

pub fn target_path(root: &Path, citation: &Citation) -> PathBuf {
    root.join(&citation.citing)
        .parent()
        .expect("citing file parent")
        .join(&citation.target)
}

pub fn lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read target {}: {e}", path.display()))
        .lines()
        .map(str::to_string)
        .collect()
}

pub fn key(citation: &Citation) -> String {
    format!("{}:{}", citation.citing, citation.index)
}

pub fn current_ledger(root: &Path, citations: &[Citation]) -> Ledger {
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

pub fn json(ledger: &Ledger) -> String {
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

pub fn ledger_errors(root: &Path, citations: &[Citation], ledger: &Ledger) -> Vec<String> {
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

pub fn assert_ledger(root: &Path, citations: &[Citation], current: &Ledger) {
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
